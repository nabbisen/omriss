# RFC-050 Implementation Handoff

## Summary

Implement the Markdown Writing Area as a calm focused-content editing surface.
It edits, previews, validates, and saves focused content. It does not rearrange
document structure and it does not expose a primary Done action.

## Scope followed

In scope:

- Keep the Markdown right panel labelled `Writing Area`.
- Use an internal `FocusedContentPanel` boundary or equivalent.
- Render selected item title, editor, preview, save/status, and optional
  read-only smaller-section navigation.
- Remove visible structure controls from the right panel.
- Implement apply-on-navigation for focused Markdown drafts.
- Ensure Ctrl+S applies a valid pending draft before save.
- Use plain save and validation messages.
- Preserve keyboard and screen-reader usability.

Out of scope:

- Rich text.
- AI writing features.
- JSON/TOML/YAML editing.
- Full adapter implementation.
- Making preview the default mode.

## Files changed

Generated handoff files:

- `rfcs/handoffs/050-writing-area-simplification-and-guided-editing/README.md`
- `rfcs/handoffs/050-writing-area-simplification-and-guided-editing/implementation-handoff.md`
- `rfcs/handoffs/050-writing-area-simplification-and-guided-editing/task-breakdown-pr-plan.md`
- `rfcs/handoffs/050-writing-area-simplification-and-guided-editing/acceptance-qa-checklist.md`

Likely implementation files:

- `crates/app/src/components/focused_content_pane.rs`
- `crates/app/src/components/raw_source.rs`
- `crates/app/src/input/keyboard.rs`
- `crates/app/src/shell/app.rs`
- `crates/app/src/shell/dispatch.rs`
- `crates/ui/src/session.rs`
- `crates/ui/src/i18n/en.rs`
- `crates/ui/src/i18n/ja.rs`

## Design decisions and assumptions

- The right side is where users work on the selected item, not where they
  rearrange the document.
- Markdown drafts are valid text and apply on navigation, save, preview, search,
  or blur.
- Future structured invalid-draft behavior is represented by RFC-053
  `DraftState`, but not implemented for Markdown-only M10.
- Optional smaller-section links are read-only navigation only.

## Tests and gates run

No implementation gates were run for this handoff beyond repository-level RFC
checks recorded by the parent task. Implementation PRs must rerun the required
commands after code changes.

Required later:

- `cargo test -p omriss`
- `cargo test -p omriss-ui`
- `cargo check -p omriss-app`
- `git diff --check`

## Generated artifacts

This handoff package only. No release archive, commit, tag, or push.

## Known limitations

- The RFC remains proposed.
- Invalid structured draft handling is future adapter work.
- Manual QA is required to prove no content is lost during navigation.

## Recommended next step

After Document Map action routing is settled, implement the focused draft
lifecycle and prove that Save/Ctrl+S captures the latest typed Markdown text.
