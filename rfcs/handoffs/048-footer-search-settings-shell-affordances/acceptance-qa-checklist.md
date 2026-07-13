# RFC-048 Footer Search / Settings Acceptance / QA Checklist

Use this checklist for the `RFC048-QA-009` implementation.

## Footer Search

- [ ] Footer shows a persistent Search control when a document is open.
- [ ] Activating footer Search opens the existing Search panel.
- [ ] Search panel focus lands in the search input.
- [ ] Search result navigation still applies a valid pending draft first.
- [ ] `Ctrl+f` still opens Search.
- [ ] Quick Actions `search.open` still opens Search.
- [ ] Search is hidden or intentionally disabled when no document is open.

## Footer Settings

- [ ] Footer shows a persistent Settings control.
- [ ] Activating Settings opens a menu.
- [ ] Settings menu contains Language controls.
- [ ] English to Japanese switching works from Settings.
- [ ] Japanese to English switching works from Settings.
- [ ] Settings closes on Escape.
- [ ] Settings closes on outside click.
- [ ] Settings controls are keyboard reachable.
- [ ] No visible Help/About item is a no-op.

## Layout and Accessibility

- [ ] Footer status text remains readable.
- [ ] Dirty state and file name remain visible.
- [ ] Footer controls do not overlap on narrow desktop widths.
- [ ] Search and Settings controls have accessible labels.
- [ ] Search and Settings controls have visible focus states.
- [ ] Tab order remains predictable.
- [ ] Document Map footer `Show plain file text` / `Back to editor` toggle is
      unchanged.

## Regression Checks

- [ ] Open a Markdown file, select a section, write, save.
- [ ] Type an unsaved change, open Search from the footer, navigate through a
      search result, and confirm the draft is preserved.
- [ ] Type an unsaved change, open Settings, change Language, and confirm the
      draft is preserved.
- [ ] Open plain file text and return to editor.
- [ ] Close/replace-current-document with unsaved changes and cancel.

## Required Commands

Run after final implementation changes:

```text
cargo check -p omriss-app
cargo test -p omriss-ui
cargo fmt --check
git diff --check
```

If docs or RFC text change, also run:

```text
bash scripts/check-rfcs.sh
mdbook build docs
```
