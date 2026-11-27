# RFC-085 — Acceptance and QA Checklist

Related RFC: `../../done/085-format-aware-panes.md`

Lifecycle status: inherited from RFC-085.

---

## Gates (each PR)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `cargo build -p omriss-components --target wasm32-unknown-unknown`
- [ ] `git diff -- crates/core` empty (PR 6: only the adapter method, RFC-085 §8.2)

## Proofs

- [ ] F2: the test's result on the old code, quoted, then green
- [ ] The open-a-node path: one call, every surface; the draft guard (new or
      existing, named)
- [ ] The structure guard, and its deliberate break
- [ ] Level 2, per surface × Markdown / valid JSON / empty JSON / unreadable
      `.json`: cards, crumbs, highlighted row, empty-state text, opened view
- [ ] Search and statistics on a JSON fixture: result paths, targets, counts,
      both locales
- [ ] Markdown tests unmodified — `git diff` of Markdown test files empty, or
      each change explained
- [ ] GUI, `en_US`, once (`scripts/gui/`): a valid `.json` opens with the
      highlighted row equal to the Writing Area's node; Esc climbs one level;
      an unreadable `.json` opens in plain file text with the map's message
- [ ] PR 4's number, with how it was measured

## Report

Gates, F2's result first, the guards with their breaks, the level-2 table, the
GUI result, the measurement, and what was not run.
