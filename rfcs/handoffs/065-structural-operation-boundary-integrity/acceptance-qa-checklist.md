# RFC-065 Acceptance / QA Checklist

## Blocking defects

- [ ] Typing into a file ending in a bare heading no longer writes into the heading
- [ ] Moving a section in a file with no trailing newline no longer destroys a heading
- [ ] Both verified by quoting before/after source bytes

## Major and minor

- [ ] Deleting a section does not weld its neighbours (RFC-065 §2.6)
- [ ] Promote on a root-parented section stays in place
- [ ] Promote whose relocation destination is a setext underline loses no nodes
- [ ] Join preserves link URLs, code spans, emphasis
- [ ] Join refuses unrecognised heading shapes with `UnsafePreservation`
- [ ] "Add section" on a CRLF file inserts CRLF
- [ ] `MoveTarget::AsFirstChildOf` / `AsLastChildOf` removed; CHANGELOG notes the break

## Properties (the real gate)

- [ ] RFC-066 P2 passes with `#[ignore]` removed
- [ ] RFC-066 P3 passes with `#[ignore]` removed
- [ ] P1 still passes (guards a different invariant; never expected to fail)
- [ ] No property anywhere still carries `#[ignore]`
- [ ] Run output pasted in the review request
- [ ] No counterexample remains unexplained

## Protected suites

- [ ] `structural_ops` unmodified and passing
- [ ] `structural_ops_delete_split_merge` unmodified and passing
- [ ] `structural_ops_move_ops` unmodified and passing
- [ ] `structural_ops_promote_demote` unmodified and passing
- [ ] `structural_ops_revision_guard` unmodified and passing
- [ ] `source_preservation` unmodified and passing
- [ ] 239 Markdown baseline unmodified and passing

## New fixtures

- [ ] `heading_only_no_trailing_newline.md` added to the catalog
- [ ] no-trailing-newline move fixture added
- [ ] CRLF split fixture added

## Invariant carried from the audit review

- [ ] Capabilities and adapter gates do not contradict for any UI-reachable node

## Hygiene

- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace` — count recorded against the 431 baseline
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `bash scripts/check-rfcs.sh`
- [ ] No file exceeds 500 ELOC without being split; no `mod.rs`
