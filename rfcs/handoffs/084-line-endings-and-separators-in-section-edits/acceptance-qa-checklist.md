# RFC-084 — Acceptance and QA Checklist

Related RFC: `../../proposed/084-line-endings-and-separators-in-section-edits.md`

Lifecycle status: inherited from RFC-084.

---

## Gates (each PR)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `git diff -- crates/core` empty, or explained (handoff §3)

## Proofs

- [ ] The property: passes now, and fails on the old path (or why not)
- [ ] The source guard, and its deliberate break
- [ ] Task 027 cases 1–5, LF and CRLF, byte-exact — test output
- [ ] A CRLF JSON file: a raw-text edit keeps untouched endings
- [ ] GUI: middle section, type at the end, Ctrl+S — one document newline
      added, caret after the typed text (`gui_bytes`, the page's selection)
- [ ] Owner, Windows: RFC-084 §12's three steps with a CRLF file
- [ ] CHANGELOG entry tells affected users, per RFC-084 §5

## Report

Gate output, the property and guard results with their breaks, the case
table, the GUI result, and what was not run.
