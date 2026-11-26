# RFC-013 — Acceptance and QA Checklist (breadcrumb layout)

Related RFC: `../../done/013-breadcrumb-and-hierarchical-navigation.md`

Lifecycle status: inherited from RFC-013.

---

## Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `git diff -- crates/core` is empty

## Proofs

- [ ] The level-2 test, and that it fails with the wrapper removed
- [ ] Through `scripts/gui/`, a document four levels deep whose headings are
      long enough to be cut: AT-SPI extents show the path on one line,
      starting at the left, and the up button at the right
- [ ] One capture of that case — the hardest one — since the owner's report
      was visual. Japanese, if its titles are the longer
- [ ] The full title is the accessible name of a cut crumb (AT-SPI)

## Report

Gate output, the test and its break, the extents, the capture, and what was
not run.
