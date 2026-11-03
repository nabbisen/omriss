# RFC-076 — Acceptance and QA Checklist

Related RFC: `../../proposed/076-editable-plain-file-text.md`

Lifecycle status: inherited from RFC-076.

---

## 1. Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace` — state the count and name every added test
- [ ] `cargo test` (default members)
- [ ] `bash scripts/check-rfcs.sh`
- [ ] `mdbook build docs`
- [ ] **`git diff -- crates/core` is empty** (handoff §2.1)
- [ ] `omriss-ui` still has exactly one dependency
- [ ] `cargo build -p omriss-components --target wasm32-unknown-unknown`

## 2. The guarantees that make this acceptable

- [ ] **One commit, one undo step.** A user who rewrites a document and presses
      Ctrl+Z once gets it back — shown as a test and in the GUI
- [ ] **Undo is byte-exact.** The source after undo equals the source before,
      byte for byte, including line endings and the trailing newline
- [ ] A stale revision is refused and the source is unchanged
- [ ] A commit that changes nothing records no history entry

## 3. The warning (RFC-071)

- [ ] RFC-071's coverage test passes **unmodified**, and the raw commit appears
      in its enumeration
- [ ] A raw commit that removes headings warns, with the right count
- [ ] A raw commit that adds headings warns
- [ ] An ordinary raw commit that changes no structure is **silent**
- [ ] The review says how `expected_change` expresses "may change anything",
      and why that is not the same as "expects zero"

## 4. Focus

- [ ] After a raw commit, focus is the outline — **always**, including when the
      previously focused section still exists with the same title. That case is
      the tempting one and it must behave identically

## 5. Drafts

- [ ] Entering the raw view with an uncommitted section draft commits or blocks
- [ ] Leaving it with an uncommitted raw draft commits or blocks
- [ ] No sequence leaves two drafts live at once

## 6. What must still work

- [ ] "Show plain file text" on a **malformed `.json`** still opens, still
      shows the source, and the document is still recoverable. This is
      `RELEASE_CHECKLIST.md` step 20's last resort
- [ ] A user who opens the view to look is not one keystroke from changing it
      (handoff §3.4) — say how that is expressed

## 7. GUI pass

Under the GUI rules. The byte comparison matters more than the picture:

- [ ] Edit the raw text, commit, save: the file on disk is exactly what the
      textarea held
- [ ] Ctrl+Z: the file returns, byte-identical, in one step
- [ ] The warning appears when headings vanish, and not otherwise

## 8. Report content

1. Gate output and the test count.
2. The §2 guarantees as results.
3. The §3 answer about `expected_change`.
4. PR 1's three symptoms, especially "a navigation after a commit pushes no new
   undo record".
5. GUI evidence.
6. What was not run, or not proven.
