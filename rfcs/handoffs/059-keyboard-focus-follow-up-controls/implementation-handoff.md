# RFC-059 — Implementation Handoff (§6.2, §6.2a, §6.2b, F-4)

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Lifecycle status: inherited from RFC-059.

---

## 1. The one cause

Every shortcut is caught by a single handler: `onkeydown` on the root
`div.app` (`crates/app/src/shell/app.rs:235-236`, `tabindex: 0`). A key reaches
it only while focus is **inside** `div.app`.

Focus leaves it in two ways, and the three symptoms are those two ways:

| Symptom | How focus leaves |
|---|---|
| §6.2a, after Ctrl+S or Esc | a re-render replaces the focused element; the WebView moves focus to `<body>`, outside `div.app` |
| F-4, after a dialog or Quick Actions closes | the focused element was inside the overlay, the overlay is removed, nothing is told where to go — `<body>` again |
| §6.2b, Ctrl+F | nothing ever moves focus *in*: `autofocus` is not honoured by the WebView (Task 019), and `.search-overlay` has no programmatic focus |

So this is not three bugs. It is the absence of a rule for **where focus
goes**, in three places that need one.

## 2. What exists today

Focus is handled in four places, by three different techniques:

| Where | Technique |
|---|---|
| dialogs — `modal_focus.rs` | initial focus by `eval` after two `requestAnimationFrame`s; Tab/Shift+Tab/arrows trapped in Rust (`trap_modal_tab`: `prevent_default`, then an `eval` that moves focus) |
| Quick Actions — `command_palette.rs` | initial focus by `eval`; Tab left native, with two `palette-sentinel` spans (`tabindex: 0`) that bounce focus back. The sentinels carry the hard-coded English text "Quick Actions focus boundary" — untranslated, and readable by a screen reader |
| section-title dialog — `app.rs` `on_structure_dialog` | its own `eval` of `.split-dialog-input` focus, on top of `focus_active_modal()` |
| Document Map row menu — `document_map_pane/menu_focus.rs` | its own copy of the same script shape |

The focusable-element selector and the visibility filter are copied into
every script in `modal_focus.rs` (four times) and `command_palette.rs` (twice).
**No code anywhere restores focus when an overlay closes.** RFC-059 §8 already
requires focus trapping to be centralised; this is how far it is from that.

## 3. The recommended design

One module, `crates/components/src/focus.rs`, owns every programmatic focus
move. Three rules.

**Rule 1 — scopes.** An overlay that takes the keyboard marks its root with
one attribute (`data-focus-scope`, say). Dialogs, Quick Actions and the search
panel all get it. One parameterised script does initial focus (`[autofocus]`
first, then the first focusable) and Tab/Shift+Tab containment, for whatever
scope is topmost. `modal_focus.rs`'s four scripts, the palette's sentinels,
and the section-title dialog's extra `eval` are deleted.

The search panel is **not** modal — it is `role="search"`, a panel beside the
document — so it gets initial focus and restoration, but **not** containment.
Phase A should confirm that is the right call (§4, question 4).

**Rule 2 — restoration.** When a scope opens, the element that had focus is
remembered. When it closes, focus goes back to that element if it is still in
the document; otherwise to the view's primary surface (the focused section's
editor, or the active Document Map node); otherwise to `div.app`. One rule for
every overlay, so no per-dialog code.

The fallback order is a pure function of state the Rust side already has —
view mode, focused section, active node — so put **that** in Rust and test it
at level 1. Only "focus this element" is JavaScript.

**Rule 3 — re-render.** After a render, focus must not be on `<body>` while
the window is the active one. Two ways to get there; Phase A decides:

- **(a) don't replace the element.** Find out *which* element Ctrl+S replaces
  and why. Dioxus patches templates in place, so a focused `<textarea>` being
  rebuilt points at something specific — a keyed or conditional change high in
  the tree, for instance. If a stable render keeps the element, the symptom is
  gone at its cause and nothing else is needed for that path.
- **(b) a guard.** One listener in `focus.rs` that notices focus on `<body>`
  and applies Rule 2's fallback. A safety net for transitions (a) cannot fix.

Expect both: (a) for save, which is common and should not even flicker; (b)
for view changes such as zoom, which genuinely remove the focused element.

### Rejected: catching keys at the document instead

Moving the shortcut handler to `document` would make focus irrelevant to
shortcuts. It does not fit this framework:

- Dioxus desktop delivers events to Rust through a synchronous request, so
  `prevent_default()` inside a Rust handler works. A `document` listener must
  be installed through `document::eval`, whose channel to Rust is
  asynchronous: by the time Rust decides a key is a shortcut, the WebView has
  already acted on it (Ctrl+F, Ctrl+P, Ctrl+S each have a WebView default).
- Deciding in JavaScript instead means a second copy of the keymap beside
  `keyboard::interpret`. Two keymaps drift.
- It treats the symptom. Focus on `<body>` is also what a screen reader
  reports — F-4 is an accessibility defect whether or not a shortcut is
  pressed.

If Phase A finds a way around the first point, say so with the evidence. Do
not build it without the architect's agreement.

## 4. What Phase A must measure

Each answer is a measurement on the real WebView, not a reading of
documentation. Use `scripts/gui/` on Linux (WebKitGTK); §4.6 is the owner's.

1. **After Ctrl+S from the section editor:** which element is focused, and
   is it the same DOM node as before (tag it with a property before the save
   and read it back after)? If it is not, which change in the render
   replaced it?
2. **When a focused element is removed:** does the WebView fire `focusout` /
   `blur`? Browsers differ on this. If WebKitGTK does not, a guard built on
   those events cannot work, and it needs another trigger.
3. **Timing:** does a restoration done by `eval` land before the next key the
   user presses? Ctrl+S followed at once by Ctrl+F is the case.
4. **Search panel containment:** with the panel open, should Tab leave it for
   the document? Recommend one answer, with the reason.
5. **The accessible tree:** confirm "no node is focused" is what AT-SPI
   reports in each F-4 case, so it can be the assertion in PR 2's test.
6. **WebView2 — the owner's.** Windows is the platform that fails §6.2a, and
   it ships through the Microsoft Store. Write a short checklist the owner can
   run on Windows for questions 1 and 3 — keys pressed, what to look at, what
   each outcome means. The owner runs it; do not guess its result.

## 5. `design-output.md` — what to hand back

- The answers to §4.1–§4.5, each with its evidence.
- The design: accept §3 or say what changes and why.
- The restoration table: every transition in the acceptance checklist, with
  its target.
- The owner's Windows checklist (§4.6).
- Anything found that changes scope — say so, and stop there.

Short. The design note is input to a review, not documentation.

## 6. Constraints

1. `omriss-core` and `omriss-ui` do not change, except a pure fallback
   function if it belongs in `omriss-ui`.
2. No new crate dependency. `omriss-components` still builds for
   `wasm32-unknown-unknown`.
3. No new shortcut and no new command — §6.5 is out of scope.
4. All JavaScript for focus lives in `focus.rs`. A source-scanning test, in
   the style of the commit-path guard (`commit_path_tests.rs`), fails if
   `.focus()` appears in any other file under `crates/components/src` or
   `crates/app/src`.
5. Any visible text a scope needs is translated. The palette's English
   sentinel text goes away with the sentinels.
6. `docs/src/known-limitations.md` changes in the same PR as the fix it
   describes — never ahead of it. Two sections are affected: "Quick Actions
   Reverse Tab Traversal Is Deferred" and "Keyboard Shortcuts Stop Working
   After a View Change or a Save". Each is deleted, or reduced to what still
   holds, once the evidence says so.

## 7. Evidence plan

Per the GUI rules' Test Optimization Rules.

| Question | Level | Evidence |
|---|---|---|
| Is the restoration target right for every view-mode, overlay and section-state combination? | 1 — unit tests on the fallback function | test table |
| Is focus code in one place only? | 1 — source-scanning test | the guard, and its deliberate break |
| Does every overlay root carry the scope marker? | 2 — Dioxus runtime test rendering each overlay | test results |
| After each transition, is exactly one node focused, the expected one? | 3 — `scripts/gui/` on X11/WebKitGTK | AT-SPI focused node, per transition |
| After Ctrl+S, does the next shortcut work at once? | 3 — `scripts/gui/` | the next shortcut's effect in the tree (Ctrl+F → search panel present) |
| The same on Windows | 3 — owner, WebView2 | the owner's checklist result |

`scripts/gui/` has no helper that reads *which* node is focused, or asserts
that none is. Add one to `lib.sh` (`gui_focused`, printing the focused node's
role and name, empty when none) and cover it in `selftest.sh`. Extend the
harness; do not script around it.

**Captures that remain: none.** A focus ring that is not visible while AT-SPI
reports focus would be the exception — capture that one case if it happens.
