# RFC-086 — Task Breakdown and PR Plan

Related RFC: `../../done/086-document-map-idle-loop-and-row-lookup.md`

Lifecycle status: inherited from RFC-086.

## PR 1 — RFC-086 §3

1. `peek()` for the effect's `item_tree` reads, and the comment made true.
2. The settle test (§3.2) over every pane, Markdown and JSON, with its
   deliberate break. Search every `use_effect` in `crates/components` and
   `crates/app` for a read-then-write of one signal. List each one you find,
   with what you did.
3. The row index (§3.3).
4. Rebuild the tree only on a structure change (§3.4).
5. CHANGELOG (§4).

Constraints: `omriss-core` and `omriss-ui` unchanged, and no change to
`dioxus-swdir-tree`. If any of those is needed, stop and report. The map's
existing tests pass unmodified. No new dependency.

Gates: fmt, clippy `-D warnings`, `cargo test --workspace` and `cargo test`
with both counts, `check-rfcs`, `mdbook build docs`, the wasm32 build.

Proofs:
- The settle test and its break.
- The effect search results.
- The measurement in Task 029's way, before and after (RFC-086 §6): idle CPU
  for Markdown and for the 8,000-item JSON file, and click to paint.
- One GUI check that expand/collapse, selection following focus, and search
  navigation still work in the map.
