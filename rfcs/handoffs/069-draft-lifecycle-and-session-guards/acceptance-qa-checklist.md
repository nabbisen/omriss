# RFC-069 — Acceptance and QA Checklist

Related RFC: `../../proposed/069-draft-lifecycle-and-session-guards.md`

Lifecycle status: inherited from RFC-069.

---

## Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `git diff -- crates/core` is empty
- [ ] `omriss-ui` still has exactly one dependency
- [ ] `cargo build -p omriss-components --target wasm32-unknown-unknown`

## §2.3

- [ ] Every case in the PR 1 table, as a test
- [ ] No `DefaultHasher` left in `session.rs`

## §2.4

- [ ] `on_navigate` contains no inline commit
- [ ] The guard test exists, runs in plain `cargo test`, and bites (show it)

## §2.5

- [ ] No `"struct.*.pending"` string anywhere — grep output in the report
- [ ] No signal written in a render body for these requests
- [ ] The five actions each open their modal; Cancel leaves the file's sha256
      unchanged — through `scripts/gui/`

## Records

- [ ] RFC-069's Status says what landed and that §2.2 remains blocked upstream
- [ ] RFC-016's header note on the close guard is unchanged — the gap is still
      real

## Report

Gate output, the test tables as results, the guard's deliberate-break output,
the grep, the GUI evidence, and what was not run.
