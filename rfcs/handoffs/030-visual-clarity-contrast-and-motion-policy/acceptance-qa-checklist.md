# RFC-030 — Acceptance and QA Checklist (contrast defects)

Related RFC: `../../done/030-visual-clarity-contrast-and-motion-policy.md`

Lifecycle status: inherited from RFC-030.

---

## Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`

## Proofs

- [ ] The guard's table of every declared pair, both themes, as test output
- [ ] Both deliberate breaks fail it (output in the report)
- [ ] `grep -n "color: *#" crates/app/assets/style.css` — no output
- [ ] One look at the welcome screen in the **light** theme — the only capture:
      the owner's report was visual, so the fix is confirmed the same way once.
      Through `scripts/gui/`, `gui_shot`
      If the harness cannot pin the light theme for its own instance, say so
      and skip it — never change the desktop's theme setting.

## Report

Gate output, the pair table, the break output, the grep, the one capture, and
what was not run.
