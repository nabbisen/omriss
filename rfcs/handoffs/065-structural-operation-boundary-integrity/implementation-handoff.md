# RFC-065 Implementation Handoff

**Governing RFC:** [RFC-065](../../proposed/065-structural-operation-boundary-integrity.md)
**Milestone:** M12 hardening — 0.17.0 ship gate
**Prerequisite:** RFC-066's P1 and P2 landed and failing.
**Sequence:** second, after RFC-066.

---

## 1. Purpose

Fix five defects that corrupt the user's document, and fix them in one place
rather than five.

Two are release-blocking under `RELEASE_CHECKLIST.md`'s own list. Both were
reproduced against the public API by the architect before this handoff was
written; the outputs in RFC-065 §2 are verbatim, not illustrative.

## 2. Read RFC-065 §3 before planning

The root cause is one sentence: **these operations know their ranges but not
their edges.** Each asks "what bytes am I inserting, and where?" and none asks
"what is immediately left and right of that position, and is the result still
well-formed?"

That is why §4.1 asks for a single `joining_separator` helper in
`preflight.rs` rather than five local patches. Five local patches is how this
happened — the same reasoning was needed in five places and written in none.
**A fix that repairs five call sites independently will be sent back.**

## 3. Slices

| Slice | Content | Risk |
|---|---|---|
| **B1** | `joining_separator` in `preflight.rs`, with unit tests, no call sites yet | low |
| **B2** | Route `replace_section_body`'s left edge through it — closes AUDIT-002 (Blocking) | **high** — primary editing path |
| **B3** | Route `move_section`'s three seams and `level.rs`'s promote-relocation branch — closes AUDIT-003 (Blocking) | **high** |
| **B4** | Promote's real-parent check — closes AUDIT-005 | medium |
| **B5** | Join preserves heading source — closes AUDIT-004 | medium |
| **B6** | Inserted headings derive the newline from the target — closes AUDIT-013 | low |
| **B7** | Withdraw `MoveTarget::AsFirstChildOf` / `AsLastChildOf` | low, but **breaking** |

B2 and B3 are the ship gate. B4–B6 are Major/Minor and should land in the same
sequence while the code is open, but may be split out if B2/B3 review runs long.

## 4. B7 — the API withdrawal

Owner-decided 2026-09-01 and recorded in RFC-024's Status field. Remove both
variants and their match arms. `AsLastChildOf` was a duplicate of `After`;
`AsFirstChildOf` split the target's body. Neither adjusted heading levels, so
neither ever created a child.

Only `move_ops.rs` references them — six lines. The GUI never used either.
Real child-placement semantics are re-proposed with RFC-058, not here.

Note it in `CHANGELOG.md` as a **breaking change for library consumers**,
alongside RFC-062's rename.

## 5. Constraints

1. **The `structural_ops*` golden suites must pass unmodified.** They are
   protected. A change there means the fix is wrong and the test is not the
   thing to repair.
2. **The 239 Markdown baseline must pass unmodified.**
3. **Do not weaken `source-preservation.md`'s claims to match the code.** The
   doc is right; the code changes. This is explicit in RFC-065 §7.4.
4. **Do not reserialize anything.** Every fix is a boundary insertion, never a
   rewrite of surrounding bytes.
5. `UnsafePreservation` is the correct answer for a heading shape the stripper
   in B5 does not recognise. Refusing is not a failure; guessing is.
6. No `mod.rs`; split files over 500 ELOC.

## 6. Required tests

Per RFC-065 §5, plus:

- fixture `heading_only_no_trailing_newline.md` in the catalog. The existing
  `no_trailing_newline.md` has a body, so it never reaches the failing shape —
  which is why `committing_last_section_body_without_trailing_newline_remains_verbatim`
  passes today while the defect is live. That test is not wrong; it is aimed
  elsewhere. Do not modify it.
- a no-trailing-newline move fixture; a CRLF split fixture;
- join preserves link URLs, code spans and emphasis, and refuses unrecognised
  shapes;
- promote on a root-parented section with a following sibling stays put;
- **the RFC-053 §241-256 invariant**, carried here from the audit review:
  capabilities and adapter gates must not contradict for any node the UI can
  reach. Assert it for every node of a projected JSON document. (This replaces
  AUDIT-0170-044, whose recommendation was rejected as contrary to RFC-053.)

## 7. Required review-request content

1. **RFC-066's P1 and P2 now pass**, with the run pasted. This is the primary
   evidence; the per-defect tests are secondary.
2. For B2 and B3: before/after source bytes for each fixed case, quoted.
3. Confirmation that `structural_ops*` and the 239 baseline are untouched —
   `git diff --stat` on the test tree.
4. Per-suite counts against the 431 baseline, counted.
5. For B7: the `CHANGELOG.md` breaking-change entry.
6. Any counterexample RFC-066 produced that is **still** failing, or any new one
   the fixes introduced.

## 8. Escalation

Stop and report if:

- a golden test must change to pass;
- `joining_separator` cannot serve all five call sites — that would mean §3's
  root-cause analysis is wrong and needs revisiting, not working around;
- a fix requires reserializing the document;
- RFC-066's properties still fail after the intended scope is complete.
