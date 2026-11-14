# RFC-059 — Phase A design output

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Phase A of `task-breakdown-pr-plan.md`. **No product code changed**: the probes
were throwaway builds (a patched `app.rs`, reverted before every commit, binary
kept under `.git-exclude/tmp/`), and the only committed code is the harness
helper `gui_focused*` with its self-test. Everything below was measured on
**WebKitGTK 2.54.2 (`webkit2gtk-4.1`) under GTK 3.24, X11 backend, default window size, English
locale**, through `scripts/gui/` with isolated state and one instance at a time.
Windows is the owner's (§5); nothing here guesses its result.

## 0. The short version

- The cause in the handoff §1 holds, with one correction in emphasis: on
  WebKitGTK **focus is not lost by re-rendering — it is lost by removal**.
  Ctrl+S keeps the very same `<textarea>`. Removing a focused element — closing
  Quick Actions or a dialog, switching to the plain file text — leaves focus on
  `<body>`, **without firing any `blur` or `focusout`**, and the next shortcut
  does nothing. That is deterministic on Linux, not "sometimes".
- So a guard must trigger on **DOM removal** (`MutationObserver`), not on focus
  events. A prototype of Rule 3(b) built that way fixed both cases tried.
- Accept §3's three rules. Changes: Rule 3's two ways are not alternatives for
  Linux — (a) is already true for Ctrl+S, (b) is what fixes everything else;
  search panel gets **no containment** (§1, Q4); one more focus site exists
  (`preview_pane.rs`); and the owner's Windows result decides whether (b) is
  enough there.

## 1. Answers to §4.1–§4.5

### Q1 — after Ctrl+S from the section editor, which element is focused; is it the same node?

**The same node, still focused.** The script tagged `document.activeElement`
on the Ctrl+S keydown and read it back after the save:

| Focus owner at Ctrl+S | After the save |
|---|---|
| section `<textarea class="body-editor">` | same node, `isConnected`, still `activeElement`; AT-SPI: `[entry] 'Edit section text'` |
| Document Map button (`Add top-level section`) | same node, still `activeElement`; AT-SPI: that button |

Ten Ctrl+S in a row from the editor: the tagged node was the active, connected
node after **all ten**.

What the save *does* remove: `span.dirty-indicator` in the toolbar and
`span.local-dirty` in the section title — nothing that holds focus. So on
WebKitGTK there is no replaced element to find; Rule 3(a) has nothing to fix
for save here, and the owner's Windows symptom is **not reproduced on Linux**,
consistent with task 015's check D. The next shortcut works immediately
(Ctrl+S then Ctrl+F in a single `xdotool` call opened the search panel).

Esc out of a top-level section also kept the same textarea (zoom-out lands on
the document root, which has an editor too).

### Q2 — when a focused element is removed, does the WebView fire `focusout` / `blur`?

**No.** Capture-phase listeners for `blur`, `focusout`, `focus`, `focusin` on
`window`, plus a `MutationObserver`, recorded:

| Removal | blur / focusout | MutationObserver | `document.activeElement` | AT-SPI focused |
|---|---|---|---|---|
| Esc closes Quick Actions (input removed) | **none** | `removed DIV.palette-overlay` | `BODY` | nothing |
| Esc closes the confirm-delete dialog | **none** | `removed DIV.modal-overlay` | `BODY` | nothing |
| Cancel (AT-SPI press) closes it | **none** | `removed DIV.modal-overlay` | `BODY` | nothing |
| Ctrl+\` — `main.focused-content-pane` (editor) removed | **none** | `removed MAIN.focused-content-pane` | `BODY` | nothing |

(When focus *moves* between live elements the events do fire normally —
Ctrl+P: `blur`/`focusout` on the textarea, `focus`/`focusin` on the palette
input.) A guard built on `focusout` therefore cannot work. The trigger is the
`MutationObserver`, with the `activeElement === body` check deferred one
`requestAnimationFrame` so the DOM has settled.

### Q3 — does a restoration done by `eval` land before the next key?

**Yes, with margin.** Over ten saves, from the page's `keydown` to the
`eval` the Rust handler issued afterwards running in the page:
**1–17 ms** (`5 11 17 7 13 4 10 1 9 12`), and that includes the save itself.
The next key in the worst case tried — Ctrl+S and Ctrl+F in one `xdotool` call —
arrived 24 ms after `s`; a person is slower. A restoration issued when a scope
closes sits in that window comfortably. (The guard prototype adds at most one
frame.) No synchronous path is needed.

### Q4 — should Tab leave the search panel for the document?

**Yes. No containment for search.** Reasons:

1. It is `role="search"` — a landmark beside the document, not a dialog and not
   `aria-modal`. Containment is the contract of a dialog; applying it to a
   landmark teaches screen-reader users the wrong thing about what it is.
2. The panel's job is to find something *in the document*. Trapping Tab makes
   the document unreachable while the panel is open, and the panel stays open
   until closed (it does not close on a click elsewhere).
3. Esc already closes it, and the Close button is in it: the user is never
   stranded, so there is no trap to contain.

What it does get: **initial focus** on the field when opened, and **restoration**
when closed — with one exception in the table (§3): choosing a result navigates,
so focus goes to the navigated section's editor, not the opener.

### Q5 — does AT-SPI report "no node is focused" in each F-4 case?

**Yes — an empty result, not a stale node.** `atspi_focused.py` walks the whole
tree; after each of Esc-from-Quick-Actions, Cancel-from-dialog,
Esc-from-dialog and Ctrl+\`, **no node reports `focused`**. So the PR 2 test can
assert exactly that, and its fix is "exactly one node, the expected one".

Two facts the assertion needs, both learned the hard way:

- **WebKit reports a focused element's ancestors as focused too.** With the
  editor focused the tree says `scroll pane`, `document web` *and* `entry` are
  all `focused`. "Which node has focus" is the deepest. `gui_focused` prints only
  the deepest; `--all` prints every one. (An assertion counting focused nodes
  would otherwise see three.)
- A fresh launch has nothing focused either — the harness self-test now asserts
  that.

Also measured, because the acceptance list needs it: **Enter on a command in
Quick Actions** (Search): the palette closes, the search panel opens, and AT-SPI
reports *nothing focused* — worse than §6.2b's "focus stays on the opener",
because the opener is gone too. And **Ctrl+F** leaves focus where it was
(`[entry] 'Edit section text'`), confirming the field is never focused.

## 2. The design

**Accept §3's three rules**, with these changes:

1. **Rule 1 (scopes) — as written.** Plus: `preview_pane.rs:25` is a **fifth**
   place that moves focus by script (the handoff counted four). It gets folded in
   with the others in PR 1; the guard test catches it either way.
2. **Rule 2 (restoration) — as written**, with the fallback function pure and
   level-1 tested. Its input is the view (outline / focus on Markdown / focus on
   a structured node / plain file text read-only / editing) and whether the
   preview is open; its output is a target, never "the opener" (the opener is
   tried first, in JS, because only the page knows whether it is still
   connected). It belongs in `omriss-components` rather than `omriss-ui`:
   `preview_open` is a host signal `omriss-ui` does not know.
3. **Rule 3 (re-render) — (b) is the mechanism; (a) is already true for save.**
   - (a): measured above; nothing to change for Ctrl+S on WebKitGTK. If the
     owner's Windows result shows the node *is* replaced there, PR 4 starts
     with finding what replaced it (§5 tells which outcome means that).
   - (b): one `MutationObserver` in `focus.rs`; on a batch of mutations,
     after one `requestAnimationFrame`, **if** `document.hasFocus()` and
     `activeElement` is `body` (or null), focus Rule 2's fallback. **Not**
     `focusout`: WebKitGTK sends none (Q2).
   - **Prototype evidence** (throwaway, same script run against the baseline and
     the prototype binary):

     | Step | Baseline | With the guard |
     |---|---|---|
     | Ctrl+P, Esc, then Ctrl+F | focused: nothing; search panel **not** opened | focused: editor; search panel opened |
     | Ctrl+\`, then Ctrl+\` again | focused: nothing; **still in plain file text** | left it; focus on `div.app` |

     The guard fired 4 times in that run, always with `activeElement = BODY`.
   - Two behaviours to decide knowingly rather than discover: **the guard also
     fires at launch** (measured: nothing is focused at launch, and Ctrl+F
     pressed before any click does **not** open the search panel — the same
     defect, fixed too; the self-test's "nothing is focused on a fresh launch" becomes
     "the app root is focused"), and **`document.hasFocus()` is required** so it
     never steals focus from another window.
   - Order, so explicit restoration and the guard never fight: a scope's close
     restores in the same frame; the guard's check runs a frame later, finds a
     non-`body` `activeElement`, and does nothing.

**Rejected alternative stands** (key handler on `document`): the handoff's
three reasons are unchanged and nothing measured gives a way around the first.

## 3. The restoration table

"Now" is measured in Phase A unless marked *not measured*. "Target" is what
PR 2–4 must produce; after every transition exactly one node is focused, and it
is the target. *Opener* = the element focused when the overlay opened, if still
in the document. *Primary surface* = Rule 2's fallback: Markdown focus → the
section `textarea`; structured focus → its editor control; plain file text →
the raw editor when editing, the raw `<pre>` (made `tabindex="0"`, so keyboard
users can scroll it) when read-only; outline → `div.app`.

| Transition | Now | Target |
|---|---|---|
| Ctrl+S from the editor | stays on the editor | unchanged |
| Ctrl+S from a Document Map control | stays on that control | unchanged |
| Esc out of a section | stays on the (root's) editor | unchanged — primary surface |
| Zoom in with Enter / zoom out with Esc | *not measured* (Enter in the editor changed nothing; outline mode is reached through the breadcrumb) — PR 4 measures it | primary surface of the new view |
| Ctrl+\` into plain file text | **nothing focused** | primary surface (read-only: the `<pre>`) |
| Ctrl+\` back | *unreachable*: the key is ignored | primary surface of the view returned to |
| Ctrl+F | stays on the opener | the search field |
| Esc / Close on search | *not measured* | opener |
| a search result chosen | *not measured* | **not** the opener: the navigated section's editor |
| Ctrl+P | the palette input | unchanged |
| Esc from Quick Actions | **nothing focused** | opener |
| Enter on a command | **nothing focused** | the command's own focus (Search → its field; Plain file text → its surface); a command that takes none → primary surface. Restoration runs first, the command's focus second |
| Tab / Shift+Tab in Quick Actions | Shift+Tab from the input stays put (sentinel bounce); Tab walks the command buttons | both cycle inside, at both ends, by the scope script — no sentinels |
| a dialog opens | its preferred control (Cancel) | unchanged |
| a dialog: Cancel / Esc | **nothing focused** | opener; if gone, primary surface |
| confirm-delete → Delete | the opener (a row-menu item) is gone | primary surface |
| section-title dialog → Add / Rename | the opener is gone | primary surface (Add: the new section's editor, which the host focuses) |
| unsaved / external-modified dialogs → Save, Discard, Overwrite | *not measured* | the continuing action's own focus, else primary surface |
| preview toggled on and off | *not measured* — `preview_pane.rs` has its own focus script | the preview's own control on, primary surface off |

**One real gap in "opener":** a dialog launched from a Document Map row menu has
a *menu item* as its opener, and the menu closes when the dialog opens — so on
Cancel the opener is gone and the fallback fires, although the row's
`Section actions` button is the right place. Recommend the menu items carry the
trigger they belong to (`data-focus-return`), which Rule 2 consults before the
fallback. It costs one attribute per menu; it avoids a rule that is right
only when the user happened to use the keyboard.

## 4. The self-test and the harness

Committed (`scripts/gui/`): `atspi_focused.py`, and in `lib.sh` `gui_focused`,
`gui_focused_is ROLE NAME`, `gui_focused_none`, documented in the README and
covered in `selftest.sh` (fresh launch: none; editor focused: exactly the editor,
and not its ancestors). `selftest.sh` passes.

## 5. The owner's Windows checklist (§4.6)

Run on the Windows build. Use a copy of any `.md` file with a few headings.
Each step says what to press, what to look at, and what the outcome means. Write
down what you saw, not what you expected.

**W1 — does saving cost the keyboard?**
1. Open the file. Click inside the text of a section so the text cursor
   blinks there. Type `x`.
2. Press **Ctrl+S**. The status bar should say *Saved*. (If it does not, stop —
   report that.)
3. **Without touching the mouse**, type `y`.
   - `y` appears in the text → focus stayed in the text area. Go to W3.
   - `y` does not appear → focus left the text area. Continue to step 4.
4. Press **Tab** once and look at which control shows the focus ring.
   - A toolbar button (probably **Open**) is highlighted → focus was on the
     page's background. **Meaning:** the page still has the keyboard, only no
     element holds it; a page-side fix will work.
   - Nothing is highlighted and further keys do nothing at all → the window's
     web view has lost the keyboard altogether. **Meaning:** a page-side fix
     cannot work here; the host has to give the web view focus back. Report
     this one first.
5. Click the text with the mouse, press Tab again, and note whether you can
   tell the difference (this is the recovery you reported).

**W2 — what removing the focused element does.**
1. Click in a section's text. Press **Ctrl+P** (Quick Actions), then **Esc**.
2. Without clicking, press **Ctrl+F**.
   - The search panel opens → focus was fine. Report it.
   - Nothing happens → same state as W1 step 3's "does not appear". Press **Tab**
     and report the same outcome as W1 step 4.
3. Repeat with **Ctrl+\`** (plain file text), then **Ctrl+\`** again: does it
   leave the plain file text?

**W3 — a quick second shortcut.**
1. Click in the text. Press **Ctrl+S** and **Ctrl+F** as fast as you can
   (hold Ctrl, tap S then F).
2. Does the search panel open? Try five times; report how many of five.
   (Linux: always.)

**Optional, only if your build has the developer tools** (a debug build
does — right-click → *Inspect*, then the *Console*): paste this once, then
press Ctrl+S in the text area. It prints whether the text area is the same
element, whether it is still on the page, what holds focus, and whether the
window has it:

```js
addEventListener('keydown',e=>{if(e.ctrlKey&&e.key==='s'){const el=document.activeElement;setTimeout(()=>console.log('after Ctrl+S: same node',el===document.activeElement,'| connected',el.isConnected,'| active',document.activeElement.tagName,'| window has focus',document.hasFocus()),300)}},true)
```

How to read it: `same node false` and `connected false` → the render **replaced**
the text area (find what replaced it; Rule 3(a)). `same node true` → the
element is kept and the cause is not a re-render. `active BODY` with
`window has focus true` → what Linux shows after a removal (the guard fixes it).
`window has focus false` → the host owns the keyboard (W1 step 4's second
outcome).

The same checklist is re-run against PR 4's build.

## 6. Things found

None changes the PR plan; they sharpen it.

1. **The Linux failure is deterministic for removal**, not "sometimes": after
   Quick Actions or a dialog closes, or after Ctrl+\`, the next shortcut is
   ignored every time. `known-limitations.md` is therefore right about Linux
   too; it is only wrong about *saving*, which on Linux does not trigger it.
2. **Plain file text is the same bug** — Ctrl+\` there leaves nothing focused and
   Ctrl+\` back does nothing. It is in the acceptance list; it needs no extra work
   beyond Rule 3.
3. **A fifth focus site** (`preview_pane.rs`), above.
4. **The Document Map's rows are not focusable elements** (the tree shows plain
   `[panel]` nodes; only their menu and create buttons are focusable). The
   checklist's fallback "the active Document Map node" therefore cannot be a focus
   target today; outline mode falls back to `div.app`. Making rows focusable is
   RFC-059 §6.5, out of scope; the fallback function takes it as a one-line
   change when §6.5 lands.
5. **Conditional, pending the owner:** if W1 step 4 reports the web view lost the
   keyboard altogether, Rule 3(b) is not enough on Windows and a host-side focus
   call is needed (the desktop host owns the window). That would be a design
   change to bring back to the architect, not something to build inside PR 4.

## 7. Evidence

Throwaway probes (not committed): `.git-exclude/tmp/t059-probe.js`,
`t059-probe-guard.js` and `t059-m1.sh` … `t059-m8.sh`. Questions, level, result:
Q1/Q3 — in-page events and timing, 10 saves; Q2 — in-page `blur`/`focusout` and
`MutationObserver` records per removal; Q5 — AT-SPI focused node after each
transition; the guard — the same script against both binaries. **No captures.**
Repeat limit: the baseline/prototype pair was run twice (my script closed the
search panel with a helper that aborts the run instead of returning; fixed and
re-run once); `m3` aborted at Ctrl+\` back, which was the finding, and its
remaining steps were run separately as `m4`.

Not run: Windows; the Japanese locale (no text here depends on it); the zoom
in/out and preview transitions marked *not measured* above.
