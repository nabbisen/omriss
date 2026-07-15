# RFC-049 Implementation Handoff

## Summary

Implement the Document Map as the single visible place for Markdown structure
navigation and organization. The Document Map owns add, rename, move, join,
delete, and show-plain-file-text actions. The Writing Area must not render
visible structure controls.

## Scope followed

In scope:

- Render a Document Map tree from the current Markdown structure.
- Show current section selection clearly.
- Route visible structure actions through Document Map row actions or menus.
- Use RFC-053 canonical movement vocabulary: `Up`, `Down`, `InsidePrevious`,
  and `OutOneLevel`.
- Derive available actions from capabilities or an equivalent view model.
- Hide unsupported actions, or disable them with plain-language reasons.
- Confirm destructive operations with cancel as the safest path.
- Preserve existing Markdown source-preserving core operations.

Out of scope:

- JSON/TOML/YAML adapters.
- Full RFC-053 adapter implementation.
- Drag-and-drop unless already stable and test-covered.
- Schema editing or parser internals in UI.

## Files changed

Generated handoff files:

- `rfcs/handoffs/049-document-map-structural-editing-model/README.md`
- `rfcs/handoffs/049-document-map-structural-editing-model/implementation-handoff.md`
- `rfcs/handoffs/049-document-map-structural-editing-model/task-breakdown-pr-plan.md`
- `rfcs/handoffs/049-document-map-structural-editing-model/acceptance-qa-checklist.md`

Likely implementation files:

- `crates/app/src/components/document_map_pane.rs`
- `crates/app/src/shell/dispatch.rs`
- `crates/ui/src/interface/commands.rs`
- `crates/ui/src/session.rs`
- `crates/ui/src/i18n/en.rs`
- `crates/ui/src/i18n/ja.rs`

## Design decisions and assumptions

- The Document Map is where users shape the document.
- Normal UI must not expose heading levels, byte ranges, parser terms, or node
  IDs.
- Markdown remains the only required runtime format for this handoff.
- Future structured formats use the same broad action vocabulary, but are not
  implemented here.
- Existing spike code is implementation context only.

## Tests and gates run

No implementation gates were run for this handoff beyond repository-level RFC
checks recorded by the parent task. Implementation PRs must rerun the required
commands after code changes.

Required later:

- `cargo test -p omriss-core`
- `cargo test -p omriss-ui`
- `cargo check -p omriss`
- `bash scripts/check-rfcs.sh`
- `git diff --check`

## Generated artifacts

This handoff package only. No release archive, commit, tag, or push.

## Known limitations

- The RFC remains proposed.
- This package does not authorize implementation without RFC-048, RFC-050, and
  RFC-051 handoffs.
- Component test coverage may need to be added where none exists today.

## Recommended next step

Implement after PR 0 reconciliation. Start by making the row action model
explicit, then move visible structure affordances into the Document Map without
changing core Markdown edit semantics.
