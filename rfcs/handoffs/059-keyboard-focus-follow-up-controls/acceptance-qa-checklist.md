# RFC-059 — Acceptance and QA Checklist

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Lifecycle status: inherited from RFC-059.

---

## Phase A — done

Investigation reported in `design-output.md`; design decided in RFC-059 §12.

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
- [ ] Every overlay root renders its `data-focus-scope` — level-2 test
- [ ] Every view renders at most one `data-focus-primary`, on §12.3's element —
      level-2 test
- [ ] No component calls `document::eval` to move focus — the same guard, or
      a grep in the report

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
      confirm → §12.2 step 2's fallback (delete especially: the opener is gone)
- [ ] A dialog opened from a row menu: Cancel → the row's "Section actions"
      button (`data-focus-return`)
- [ ] A search result chosen → that section's primary surface
- [ ] Launch, with a file and without → primary surface or `div.app`, and the
      first shortcut works
- [ ] Read-only plain file text: the `<pre>` takes focus and is announced by
      its translated name
- [ ] Unchanged: dialog arrow keys, row-menu keys, preview Back

## Windows (owner)

- [ ] The owner's checklist (`design-output.md` §5), run against PR 2's build —
      result recorded as the owner reported it

## Records

- [ ] RFC-059's Status says which parts of §12 landed; §6.1, §6.3, §6.4 and
      §6.5 remain open
- [ ] `docs/src/known-limitations.md` matches what the evidence shows, changed
      in the PR that fixed each part

## Report

Gate output, test tables as results, the guard's deliberate-break output, the
grep, the transition results, the Windows result, and what was not run.
