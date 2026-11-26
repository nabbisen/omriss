# RFC-059 — Task Breakdown and PR Plan

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Lifecycle status: inherited from RFC-059.

The design is RFC-059 §12. PR 1, PR 2 and PR 3 in order; PR 4 only if the
owner's Windows run requires it.

---

## Phase A — investigation (done)

Findings in `design-output.md`; the `gui_focused*` helpers in `scripts/gui/`.
Decisions taken from it: RFC-059 §12.

## PR 1 — `focus.rs`: scopes and containment

- The installed script (§12.2 steps 1 and 3) and the markup of §12.1.
- `data-focus-scope` on every dialog, Quick Actions, the Document Map row
  menu (`contain`), and the search panel and the preview (`panel`);
  `data-focus-arrows` where arrow keys move focus today.
- Deleted: `modal_focus.rs`'s scripts and `trap_modal_tab`,
  `focus_active_modal`, `focus_active_palette`, the palette sentinels, the
  section-title dialog's extra `eval`, `menu_focus.rs`'s scripts, and
  `preview_pane.rs`'s focus `eval`.
- The source-scanning guard (no `.focus()` outside `focus.rs`).

Behaviour changes, and only these: Shift+Tab no longer escapes Quick Actions
(`RFC048-QA-019`); Ctrl+F focuses the search field (§6.2b). Everything else
that worked must work the same — dialog initial focus, dialog arrow keys, row
menu keys, preview Back.

Docs in this PR: the limitations section on Quick Actions reverse Tab, and the
"search panel is the exception" paragraph.

Show the guard bites: put a `.focus()` back into `app.rs`, watch the test fail,
restore.

**Landed** (`481092f`, corrections `ab96585`). The corrections, from review:
§12.2's editable-field, IME and type-to-field rules; the script enters scopes
already present when it installs; the section-title dialog stops propagation
of the Enter and Escape it handles.

## PR 2 — restoration (F-4, plain file text, launch)

- §12.2 step 2: the observer's removal check and the three-step fallback.
- `data-focus-primary` per §12.3; the read-only `<pre>`'s `tabindex` and
  accessible name, translated in both locales.
- `data-focus-return` on the row-menu items.
- The one explicit call, for choosing a search result.

Docs in this PR: the limitations section on shortcuts after a view change or a
save — deleted for Linux, reduced to what the owner's Windows run still shows.

Carried from PR 1: **Rename → Esc leaves the view as it was** — one run,
English, the baseline taken *after* the dialog opens (the heading and whether
the editor is present). Inside this PR's GUI run, not a separate round.

Not in this PR: the Document Map tree's double-handled keys (RFC-059 §12,
owned by §6.5).

**Landed** (`64fc36c`, harness `634bbea`).

## PR 3 — overlays are contained (§12.3a)

From the owner's Windows check, 2026-11-26.

- `.search-overlay`: `data-focus-scope="contain"`. Remove `focus_primary` and
  its call in `app.rs`'s search `on_navigate`; choosing a result keeps focus
  on the result.
- The settings popover (`toolbar.rs`): `data-focus-scope="contain"`; remove
  its `role="menu"`, `tabindex: 0` on the container if nothing needs it, and
  the gear button's `aria-haspopup="menu"`. Its own Esc handler stays.
- §12.2 step 2: focus left on `<body>` while a `contain` scope is open goes to
  that scope's first control (§12.3a). Choosing a result may re-render the
  list; this is what keeps focus in the panel then.
- The level-2 scope test expects `contain` for both, and covers the settings
  popover if it does not yet. No new guard: the rule is RFC-059 §12.3a.

GUI, `en_US`, once: from the search field, 20 Tabs and 20 Shift+Tabs never
leave the panel; choose a result → focus stays on it; Esc → the editor of the
section shown. The same containment for the settings popover; Esc → the gear
button.

## PR 4 — Windows, only if needed

Opened only if the owner's checklist, run against PR 2's build, shows the
symptom remains on WebView2. If it shows the web view itself losing the
keyboard (`design-output.md` §5, W1 step 4, second outcome), stop: that is a
design change for the architect (RFC-059 §12.4), not this PR.
