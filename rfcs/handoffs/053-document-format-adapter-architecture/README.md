# RFC-053 Handoff

Companion execution package for:

- RFC: `rfcs/proposed/053-document-format-adapter-architecture.md`
- Milestone: M11 — Format Adapter Foundation

This package replaces the M10-era stub, whose own checklist ended with
"This stub is replaced by a full handoff before RFC-053 implementation."

## Contents

- `implementation-handoff.md` — purpose, slices, non-change scope, risks,
  required evidence
- `task-breakdown-pr-plan.md` — S1–S5 detail and sequencing
- `acceptance-qa-checklist.md` — per-slice verification, including the two
  manual checks that no test can cover

## What this work is

Introduce the format adapter boundary in `omriss-core`, and reconcile the
capability types that M10 shipped early into `omriss-ui`.

## What this work is not

**No new user-facing format ships here.** JSON, TOML, and YAML are RFC-054,
RFC-055, and RFC-056. When this work is done, Markdown behavior must be
byte-for-byte identical and the app must look and behave exactly as it does
today.

Two slices carry most of the risk:

- **S3** must produce zero user-visible change while moving types between
  crates;
- **S4** must wrap the shipped Markdown structural operations, never
  reimplement them.

## Prerequisite

Dev-team task 002 (file split + clippy gate) should land first. It touches
`document_map_pane.rs` and `structural.rs`, which S3 and S4 also touch.
