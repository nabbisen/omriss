# RFC-067 Implementation Handoff

**Governing RFC:** [RFC-067](../../proposed/067-structure-projection-caching.md)
**Milestone:** M12 hardening — 0.17.0 ship gate, §3.1–3.2 only
**Sequence:** independent.

---

## 1. Purpose

Make JSON usable at realistic file sizes by putting a memoisation layer under
RFC-054 §0.4's build-fresh rule — without weakening the rule.

Measured today, release build, 405 KB JSON / 8000 items: **~25 ms per
keystroke**, four to five full re-parses. At 4 MB that is ~250 ms, which is not
usable. JSON is 0.17.0's headline feature.

## 2. The rule is right; the missing piece is beneath it

RFC-054 §0.4 says build structure fresh, never cache. That is a sound
correctness rule and it must not be discarded — it exists to prevent serving
structure built from **stale source**.

A revision key satisfies it exactly. Revisions are monotonic and every mutation
bumps exactly one, so a revision match is a *proof* of freshness, strictly
stronger than "we rebuilt it recently". A draft does not change the revision,
which is why five parses per keystroke become zero.

**Do not cache on anything but the revision.** Not on source length, not on a
hash, not on a dirty flag. The whole soundness argument is the revision's
monotonicity.

## 3. Slices

| Slice | Content | In 0.17.0 gate |
|---|---|---|
| **C1** | Revision-keyed `DocumentStructure` cache on `EditorSession` | **yes** |
| **C2** | Skip `build_outline` when the format is not Markdown | **yes** |
| **C3** | Amend RFC-054 §0.4 to state the revision-freshness rule | **yes** |
| **C4** | `known-limitations.md` updated with real measured numbers | **yes** |
| **C5** | Invert the splice instead of copying the document | no — 0.18.0 |
| **C6** | Search hot-loop allocations | no — 0.18.0 |

## 4. Constraints

1. **Adapters still own no state** (RFC-053). The cache lives on
   `EditorSession`, not in an adapter.
2. **One entry.** No eviction policy, no cross-document cache. Replaced when the
   revision moves.
3. **No incremental parsing.** Full re-parse per revision is fine; per keystroke
   is not.
4. **No behavioural difference.** This is a performance change. Every existing
   JSON suite must pass **unmodified** — a golden-test change means it is not a
   performance change.
5. C2 is a correctness-neutral win: `document_map_nodes`, `focus` and
   `prune_dead_history` all already branch away from the Markdown outline for
   non-Markdown formats, so it is built and never read.

## 5. Required tests

- Mutating the document invalidates the cache: same node, new revision,
  structure reflects the edit.
- A draft edit does **not** invalidate it.
- Undo invalidates it correctly — the revision moves on undo too.
- A JSON document does not build a Markdown outline.

## 6. Required review-request content

1. **Before/after measurements** for the three numbers in RFC-067 §2, taken the
   same way (release build, `Instant::now()`, same fixture size). Measured, not
   estimated.
2. Confirmation that every existing JSON suite passes unmodified —
   `git diff --stat` on the test tree.
3. The `known-limitations.md` diff, replacing the "not measured against a large
   file" sentence with real numbers.
4. The RFC-054 §0.4 amendment.

## 7. Note

The sentence in `known-limitations.md` that under-describes this behaviour was
written by the architect, not by the dev team. Correcting it is part of this
slice and carries no implication about anyone's work.

## 8. Escalation

Stop and report if:

- the cache cannot be keyed on revision alone without a correctness gap. That
  would mean RFC-067 §3.1's soundness argument is wrong, and it needs revisiting
  rather than patching with a second key;
- any existing JSON test must change;
- the measured improvement is materially smaller than §2 predicts — that means
  the parse count analysis was wrong and the real cost is elsewhere.
