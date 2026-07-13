# RFC-048 Footer Search / Settings Implementation Handoff

## Summary

Implement the accepted M10 footer amendment from `RFC048-QA-009`:

- replace the direct footer language selector with compact Search and Settings
  controls;
- make Search pointer-discoverable from the persistent shell footer;
- move Language into Settings;
- do not show Help/About in M10 unless it has real behavior.

This is a small shell affordance improvement. It must not disturb Document Map
structure controls or the Document Map footer plain-file-text toggle.

## Scope followed

In scope:

- Add a footer Search button that opens the existing `SearchPanel`.
- Add a footer Settings button that opens a small menu.
- Move the existing locale switching behavior into the Settings menu.
- Hide or disable Search intentionally when no document is open.
- Close the Settings menu on Escape and outside click.
- Preserve keyboard traversal and focus-visible styling.
- Keep English and Japanese labels in i18n.
- Record manual QA evidence against `RFC048-QA-009`.

Out of scope:

- A no-op Help/About menu item.
- A full preferences/settings screen.
- Reworking SearchPanel behavior, search indexing, or result navigation.
- Moving Document Map actions or changing structure-edit semantics.
- Adding a new global shell state model.

## Files changed

This handoff package was generated in:

- `rfcs/handoffs/048-footer-search-settings-shell-affordances/README.md`
- `rfcs/handoffs/048-footer-search-settings-shell-affordances/implementation-handoff.md`
- `rfcs/handoffs/048-footer-search-settings-shell-affordances/task-breakdown-pr-plan.md`
- `rfcs/handoffs/048-footer-search-settings-shell-affordances/acceptance-qa-checklist.md`

Likely implementation files:

- `crates/app/src/components/status_bar.rs`
- `crates/app/src/shell/app.rs`
- `crates/app/assets/style.css`
- `crates/ui/src/i18n/en.rs`
- `crates/ui/src/i18n/ja.rs`
- component or shell tests if available for StatusBar/menu behavior
- `.git-exclude/qa/rfc-048-m10/issue-log.md`
- `.git-exclude/qa/rfc-048-m10/manual-end-to-end.md`

## Design decisions and assumptions

- Search is a normal editing/navigation utility and should be discoverable
  without knowing `Ctrl+f` or Quick Actions.
- The footer right side is the appropriate shell utility location for Search
  and Settings.
- Language remains supported, but it is lower-frequency than Search and should
  live under Settings.
- M10 should defer Help/About rather than adding a visible dead affordance.
- The implementation should extend `StatusBar` with explicit callbacks from
  `App`; do not introduce broad app-state restructuring for this change.
- Icon controls may be dependency-free for M10 if the app has no existing icon
  library. Accessible labels and titles are required.

## Tests and gates run

While creating this handoff package, no build or test command was run.

Required after implementation:

- `cargo check -p omriss-app`
- `cargo test -p omriss-ui`
- `cargo fmt --check`
- `git diff --check`

Recommended if docs or RFC text change:

- `bash scripts/check-rfcs.sh`
- `mdbook build docs`

## Generated artifacts

Generated handoff package:

- `README.md`
- `implementation-handoff.md`
- `task-breakdown-pr-plan.md`
- `acceptance-qa-checklist.md`

No commit, tag, push, release archive, or deployment was created.

## Known limitations

- The current accepted M10 decision omits Help/About unless implemented.
- The package assumes the existing `SearchPanel` remains the search UI.
- Pointer discoverability improves only after implementation; current app still
  requires `Ctrl+f` or Quick Actions.

## Recommended next step

Implement PR 1 from `task-breakdown-pr-plan.md`, then run the focused QA
checklist and create an implementation review request package before treating
`RFC048-QA-009` as closed.
