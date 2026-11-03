# RFC-075 — Acceptance and QA Checklist

Related RFC: `../../done/075-component-crate-extraction-and-renderer-portability.md`

Lifecycle status: inherited from RFC-075.

---

## 1. Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace` — **the count is unchanged**, or every
      difference is explained test by test
- [ ] `bash scripts/check-rfcs.sh`
- [ ] `mdbook build docs`

## 2. The boundary, demonstrated

- [ ] **`cargo build -p omriss-components --target wasm32-unknown-unknown`
      succeeds.** The acceptance test for the whole RFC: if it fails, a
      platform dependency survived and the boundary is not where RFC-075 says
      it is
- [ ] `cargo test` **without** `--workspace` now builds and tests the
      components, which it could not before — give the before and after
- [ ] `omriss-ui` still has **exactly one dependency**
- [ ] No file in `omriss-components` imports `crate::` anything outside its own
      crate — shown as a grep and its output, not asserted

## 3. The move, demonstrated

- [ ] The diff is moves and import paths. State it as a command and its output
      — `git diff --stat` and `git log --follow` on a sample file showing the
      rename was tracked
- [ ] No logic change. If one was unavoidable, name the file and the reason;
      do not bury it
- [ ] `crates/components/Cargo.toml` carries `publish = false` **with the
      reason in a comment**, and the description from the handoff §2 verbatim

## 4. The desktop application is unchanged

- [ ] It builds and runs
- [ ] Open a Markdown file, edit a section, save, undo — unchanged behaviour
- [ ] Open a JSON file, edit a value, save, undo — unchanged behaviour
- [ ] One dialog opens and closes correctly (the components moved; the modal
      focus script moved with them)

**No reduced smoke run is required by this work** — it changes no behaviour.
The release's own run at the end covers it, and if this move broke something
user-visible then §3 was not a move.

## 5. The question answered by construction

- [ ] Handoff §5.1 — `input/keyboard.rs`: moved or kept, **with the reasoning**
      and with every caller enumerated rather than sampled

## 6. Report content

1. The diff statistics and the move evidence (§3).
2. The wasm build output (§2).
3. The before and after of `cargo test` without `--workspace` (§2).
4. The §5.1 answer.
5. Gate output.
6. What was not run, or not proven.
