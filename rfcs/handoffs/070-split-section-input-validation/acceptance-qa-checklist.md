# RFC-070 — Acceptance and QA Checklist

Related RFC: `../../done/070-split-section-input-validation.md`

Lifecycle status: inherited from RFC-070.

---

## Gates (each PR)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `cargo test -p omriss-core --test properties` — count, and **0 ignored**
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`

## Proofs

- [ ] Every title `rename_section` refuses, `split_section` refuses
- [ ] `rename_section` accepts and refuses the same titles as before
- [ ] Offsets refused: inside an ATX child heading, between a setext title and
      its underline, past the body, inside a character, between CR and LF
- [ ] An offset legally in the body still splits; `structural_ops*` unmodified
- [ ] The split property un-ignored and green; its break with the old bound
- [ ] P4 for rename and join; its break with a §2.8 guard reverted
- [ ] `grep -rn '#\[ignore' crates/` — no output

## Report

Gates, the proofs with where each lives, the two shrunk cases from the breaks,
and what was not run.
