# RFC-048 Footer Search / Settings Task Breakdown / PR Plan

## PR 1 - Footer Affordance Implementation

Goal: make Search pointer-discoverable and keep Language available through
Settings.

Tasks:

- Extend `StatusBar` props with explicit handlers/state:
  - open/toggle Search;
  - know whether Search is available;
  - read/write locale through the existing `locale` signal.
- Replace the direct footer locale `select` with:
  - a Search icon/text button;
  - a Settings icon/text button.
- Wire Search to the existing `search_open` state in `App`.
- In Settings, render the existing locale choices.
- Omit Help/About for M10 unless a minimal functional About dialog is included.
- Add or reuse i18n keys for `Search`, `Settings`, and any menu labels.
- Add CSS for compact footer utility buttons and the settings menu.

Acceptance:

- A pointer user can open Search from the footer when a document is open.
- Search still opens with `Ctrl+f` and Quick Actions.
- A pointer or keyboard user can change Language from Settings.
- No visible Help/About item is a no-op.
- The Document Map footer plain-file-text toggle is unchanged.

Suggested checks:

- `cargo check -p omriss`
- targeted component/session tests if available

## PR 2 - Keyboard, Focus, and Menu Dismissal

Goal: make the new footer menu behave like a trustworthy shell control.

Tasks:

- Ensure footer Search and Settings buttons are reachable by keyboard.
- Ensure the Settings menu closes on Escape.
- Ensure the Settings menu closes on outside click.
- Ensure focus styling is visible for the buttons and menu controls.
- Ensure tab order remains predictable in the footer.
- If Search is unavailable on the welcome screen, hide it or disable it with a
  plain accessible explanation.

Acceptance:

- Keyboard users can open Settings, change Language, and close the menu.
- Escape closes Settings before affecting editor focus.
- Clicking outside Settings closes it.
- The Search affordance does not open an empty/useless panel when no document is
  open.

Suggested checks:

- `cargo check -p omriss`
- `cargo test -p omriss-ui`

## PR 3 - QA Record and Review Package

Goal: finish the controlled M10 follow-up cleanly.

Tasks:

- Manually retest `RFC048-QA-009`.
- Update `.git-exclude/qa/rfc-048-m10/issue-log.md`.
- Update `.git-exclude/qa/rfc-048-m10/manual-end-to-end.md` if the search
  operation text changes from keyboard-only to footer Search.
- Create an implementation review request package for the footer change.
- Record command evidence in the review package.

Acceptance:

- `RFC048-QA-009` is either closed or has a clear owner-accepted residual.
- Implementation review package exists at the review point.
- Required commands are observed after the final implementation change.

Suggested checks:

- `cargo fmt --check`
- `cargo check -p omriss`
- `cargo test -p omriss-ui`
- `git diff --check`

## PR Ordering Notes

- PR 1 and PR 2 may be combined if the diff remains small.
- PR 3 must happen after implementation and verification.
- Do not implement Help/About as a visible item unless it performs a real
  action.
