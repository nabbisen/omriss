# RFC-059 — Acceptance and QA Checklist

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Lifecycle status: inherited from RFC-059.

---

## Phase A

- [ ] `design-output.md` answers the implementation handoff §4.1–§4.5, each
      with its evidence
- [ ] The restoration table covers every transition below
- [ ] The owner's Windows checklist is written, and runnable without reading
      the code
- [ ] `gui_focused` is in `lib.sh` and covered by `selftest.sh`, which passes
- [ ] No product code changed

## Gates (each PR)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`, and `cargo test` — state both counts
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] `git diff -- crates/core` is empty
- [ ] `cargo build -p omriss-components --target wasm32-unknown-unknown`
- [ ] `bash scripts/gui/selftest.sh` passes before the GUI run

## Structure

- [ ] All focus JavaScript is in `focus.rs`; the guard test runs in plain
      `cargo test` and bites (show it)
- [ ] `palette-sentinel` and "Quick Actions focus boundary" appear nowhere —
      grep output in the report
- [ ] Every overlay root renders the scope marker — level-2 test

## Transitions — exactly one node focused afterwards, the expected one

Through `scripts/gui/`, on X11/WebKitGTK, both `en_US` and `ja_JP` once.

- [ ] Ctrl+S from the section editor — focus stays in the editor, and the
      next shortcut works at once
- [ ] Ctrl+S from the Document Map
- [ ] Esc out of a section
- [ ] Zoom in with Enter, zoom out with Esc
- [ ] Ctrl+\` there and back
- [ ] Ctrl+F — the search field is focused; Esc or Close — back to the opener
- [ ] Ctrl+P — the palette input; Esc — back to the opener; Enter on a command
      — that command's own focus (Search → the search field)
- [ ] Quick Actions: Tab and Shift+Tab both cycle inside, at both ends
- [ ] Each dialog: open → its preferred control; Cancel or Esc → the opener;
      confirm → the table's target (delete especially: the opener is gone)

## Windows (owner)

- [ ] The owner's checklist, run against PR 4's build — result recorded as the
      owner reported it

## Records

- [ ] RFC-059 records the accepted design, and which sections landed; §6.1,
      §6.3, §6.4 and §6.5 remain open
- [ ] `docs/src/known-limitations.md` matches what the evidence shows, changed
      in the PR that fixed each part

## Report

Gate output, test tables as results, the guard's deliberate-break output, the
grep, the transition results, the Windows result, and what was not run.
