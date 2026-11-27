# RFC-060 — Task Breakdown and PR Plan (P-1 to P-4)

Related RFC: `../../proposed/060-accessibility-screen-reader-validation-plan.md`

Lifecycle status: inherited from RFC-060.

The design is RFC-060 §10.

## PR 1 — names that are true

1. P-1: first measure `document::Title` (§10.1). If it names the document and
   the window, use it through one title function. Otherwise stop and report.
2. P-2: landmark names in every view; the level-2 landmark test and its
   deliberate break.
3. P-4: the toolbar's own key.
4. P-3: the measurement, and the markup fix only if omriss's markup is the
   cause.

Constraints:
- Both locales, through the catalog.
- No change to `omriss-core` or `omriss-ui`, except catalog keys and the title
  function if it belongs in `omriss-ui`.
- No new dependency.
- `omriss-components` still builds for `wasm32-unknown-unknown`.
- `docs/src/` and `known-limitations.md` change with anything they describe.

Gates: fmt, clippy `-D warnings`, `cargo test --workspace` and `cargo test`
with both counts, `check-rfcs`, `mdbook build docs`, the wasm32 build.

Report: gates, the P-1 measurement, the landmark table per view, P-3's
finding, the AT-SPI output in both locales, and what was not run.
