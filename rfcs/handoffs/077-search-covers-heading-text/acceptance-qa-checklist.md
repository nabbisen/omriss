# RFC-077 — Acceptance and QA Checklist

Related RFC: `../../done/077-search-covers-heading-text.md`

Lifecycle status: inherited from RFC-077.

---

## Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `cargo build -p omriss-components --target wasm32-unknown-unknown`
- [ ] `git diff -- crates/core`: the one doc comment

## Proofs

- [ ] A visible heading's title is found; it opens that section (no offset)
- [ ] Heading and body in one section: two results, title first
- [ ] `---` and `===` find no setext underline
- [ ] JSON: a key is found; "Item" and an index find nothing; key + value:
      two results
- [ ] "Current Section" includes the focused node's title and those below
- [ ] The result-kind word in both locales (Level 2)
- [ ] Existing Markdown search tests unmodified
- [ ] The measurement on RFC-085's two fixtures, before and after
- [ ] GUI run (task breakdown)

## Report

Gates, the proofs with where each lives, the measurement, the GUI result, and
what was not run.
