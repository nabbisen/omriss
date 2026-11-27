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

**PR 1 landed** (`112c280`): P-2, P-4; P-3 measured; P-1 held.

## PR 2 — P-1 and P-3's note (RFC-060 §11)

1. The title function, the product-name constant, and one component that
   applies the title to the window and the page.
2. The start-up name from the same constant (`custom_head`, `main.rs`).
3. The guard of §11.3, with a deliberate break.
4. `known-limitations.md`: the section-actions button is announced as a combo
   box on Linux, with the reason and what it does.

Proof: AT-SPI, `en_US` and `ja_JP`. Read the `[document web]` name as early
as the harness can after launch: "Omriss". Then with a file open:
"notes.md — Omriss". Then after closing or switching the file. Check the
window title in each state too.

**PR 2 landed** (`79d1942`).

## PR 3 — constant scripts (RFC-060 §12)

1. The title reaches the page as data through the eval channel; `js_string` is
   deleted.
2. The start-up script is a constant from `PRODUCT_NAME`.
3. The guard: every `document::eval(` argument is a literal or a `const`. It has
   a deliberate break.

Proof: the title tests and the poller run once, `en_US`, including a file
whose name contains `"`, `\` and `'`. The page's name is that file name
exactly.

**PR 3 landed** (`c8a0ac0`). P-1 to P-4 are closed.
