# RFC-073 — Acceptance and QA Checklist

Related RFC: `../../done/073-complete-localization-and-locale-persistence.md`

Lifecycle status: inherited from RFC-073.

---

## Gates (each PR)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `cargo build -p omriss-components --target wasm32-unknown-unknown`
- [ ] `git diff -- crates/core`: only §8.6

## Proofs

- [ ] The list of `Err` sources and the variant each became (or why none)
- [ ] No English cause left: the command and its output
- [ ] Exhaustive maps, the one commented `ErrorKind` wildcard
- [ ] Level 2: the open-failure dialog per variant, en and ja
- [ ] Save failure: the status message per reason, en and ja
- [ ] GUI: `chmod 000`, both locales, AT-SPI description has no OS text
- [ ] `from_tag("ja.UTF-8")` is Japanese
- [ ] JSON list labels in Japanese in the five places; RFC-077's search tests
      unmodified

## Report

Gates, the `Err` list, the proofs with where each lives, the GUI result, and
what was not run.
