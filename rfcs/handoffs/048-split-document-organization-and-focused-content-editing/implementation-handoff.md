# RFC-048 Implementation Handoff

## Summary

Implement the M10 UI role separation for Markdown first:

- left side: `Document Map`, for structure navigation and organization;
- right side: Markdown `Writing Area`, internally `FocusedContentPanel`, for
  focused content editing, preview, save/status, and raw-source escape hatch.

The developer must implement against the accepted M10 bundle shape, not RFC-048
alone. Load-bearing behavior is split across:

- RFC-048: product decision and layout roles;
- RFC-049: Document Map structure actions and row/menu model;
- RFC-050: Writing Area behavior and apply-on-navigation draft lifecycle;
- RFC-051: migration sequencing, release gates, and acceptance criteria;
- RFC-053 type core: canonical type names for node kinds, capabilities,
  structure commands, movement, and draft state.

The first implementation target is Markdown. Do not implement JSON, TOML, YAML,
or the full adapter trait in this handoff.

## Scope followed

In scope:

- Rename user-facing `Outline` wording to `Document Map` where it refers to the
  left navigation/organization panel.
- Keep Markdown's right-side label as `Writing Area`; use
  `FocusedContentPanel` or equivalent for internal component naming.
- Remove visible structural editing controls from the Writing Area.
- Place visible structure operations in the Document Map.
- Use row menus, command routing, or equivalent UI affordances for add, rename,
  delete, move, join, and show-plain-file-text operations.
- Use apply-on-navigation: no primary `Done` button or `Commit -> Done`
  workflow.
- Ensure Ctrl+S applies a valid pending focused draft before save.
- Keep Markdown source text canonical and preserve unrelated source bytes.
- Update i18n labels, docs, keyboard reference, and release notes.
- Add or update tests and manual QA coverage from
  `acceptance-qa-checklist.md`.

Out of scope:

- JSON/TOML/YAML parsing or editing.
- Full RFC-053 adapter implementation beyond the pulled-forward type concepts
  needed by the UI boundary.
- Drag-and-drop structure movement unless already stable and covered.
- Rich text, collaboration, plugin, AI, mobile layout, or schema features.
- Any lifecycle move from `proposed/` to `done/`.

## Files changed

This handoff package was generated in:

- `rfcs/handoffs/048-split-document-organization-and-focused-content-editing/README.md`
- `rfcs/handoffs/048-split-document-organization-and-focused-content-editing/implementation-handoff.md`
- `rfcs/handoffs/048-split-document-organization-and-focused-content-editing/task-breakdown-pr-plan.md`
- `rfcs/handoffs/048-split-document-organization-and-focused-content-editing/acceptance-qa-checklist.md`

Likely implementation files:

- `crates/app/src/components/document_map_pane.rs`
- `crates/app/src/components/focused_content_pane.rs`
- `crates/app/src/components/command_palette.rs`
- `crates/app/src/components/raw_source.rs`
- `crates/app/src/input/keyboard.rs`
- `crates/app/src/shell/app.rs`
- `crates/app/src/shell/dispatch.rs`
- `crates/ui/src/i18n/en.rs`
- `crates/ui/src/i18n/ja.rs`
- `crates/ui/src/interface/commands.rs`
- `crates/ui/src/session.rs`
- `docs/src/*.md`
- `README.md`
- `CHANGELOG.md`

Existing worktree context:

- There are already dirty implementation edits related to this area. Treat them
  as an exploratory spike to audit, keep, amend, or replace according to the
  RFCs. They are not design authority.
- `.gitignore` also has a pre-existing dirty change for `/.git-exclude/`.
  Do not revert it unless the owner explicitly asks.

## Design decisions and assumptions

- The two-zone split is the durable product decision.
- Markdown remains first-class and must not regress.
- Source text remains canonical. The Document Map is derived structure.
- The Writing Area must not expose visible structure-changing controls.
- `Done` is not a primary action. Drafts apply on navigation, save, preview,
  search, or blur.
- The right panel's user-facing name is `Writing Area` for Markdown only.
  Structured formats should title the selected item instead of inheriting that
  panel name.
- RFC-053 owns canonical public type names. Do not introduce a local
  `FocusedNodeKind`.
- Future format readiness means compatible boundaries, not parser work.
- Destructive structure operations require confirmation with the safest default.
- Errors and status messages must not expose parser internals, byte ranges, node
  IDs, or system details.

Assumptions to confirm before coding:

- The owner accepts using the existing dirty implementation as a spike rather
  than reverting first.
- M10 may pull forward the RFC-053 type core without implementing the full M11
  adapter architecture.
- If implementation is shipped before RFC lifecycle completion, it must be
  release-gated or held from release per RFC-051.

## Tests and gates run

Observed while creating/amending the design/handoff package in this working
session:

- `bash scripts/check-rfcs.sh` passed.
- `git diff --check` passed.

Earlier local observations from this thread, not durable release evidence:

- `cargo fmt` completed.
- `cargo test -p omriss -p omriss-ui` passed.
- `cargo check -p omriss-app` passed.
- `mdbook build docs` passed, and the generated `docs/book` artifact was
  removed afterward.
- `bash scripts/check-rfcs.sh` passed.
- `git diff --check` passed.

Required for the implementation PR:

- Rerun every command above after the final implementation changes.
- Record durable evidence in CI logs, committed evidence notes, or the PR
  description before treating gates as release evidence.
- Add any missing focused UI/component/manual QA evidence from
  `acceptance-qa-checklist.md`.
- Do not claim release readiness until the full checklist is satisfied.

## Generated artifacts

Generated handoff package:

- `README.md`
- `implementation-handoff.md`
- `task-breakdown-pr-plan.md`
- `acceptance-qa-checklist.md`

No release archive, commit, tag, or push was created.

## Known limitations

- This handoff is based on RFCs that still live under `rfcs/proposed/`.
- It does not accept, implement, or ship RFC-048.
- It does not replace the RFC-049/050/051 sibling handoffs; those must exist
  before PR 1 or any implementation PR opens.
- Current dirty implementation files may already satisfy parts of the handoff,
  but they still require audit against the amended RFC text.
- Non-technical-user QA cannot be completed by code review alone.

## Recommended next step

Use `task-breakdown-pr-plan.md` to split the work into small PRs. The first PR
should be a reconciliation PR: compare the current dirty implementation against
the amended RFCs, remove any stale `Done`/`FocusedNodeKind` assumptions, and
prove that the right panel has no visible structural controls.
