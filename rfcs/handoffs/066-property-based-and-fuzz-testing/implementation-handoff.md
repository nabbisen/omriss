# RFC-066 Implementation Handoff

**Governing RFC:** [RFC-066](../../proposed/066-property-based-and-fuzz-testing.md)
**Milestone:** M12 hardening — 0.17.0 ship gate
**Sequence:** **first** in the hardening sequence, before RFC-065.

---

## 1. Purpose

Add the test layer that would have caught all three of the audit's
release-blocking defects, and demonstrate that it does.

This slice deliberately **fixes nothing**. Its output is failing tests.

## 2. Why this comes first

RFC-065 fixes five defects. The question that matters afterwards is not "do
those five cases pass now?" but "is the invariant restored?" A property written
before the fix answers the second question; a property written after it answers
neither, because a property that has only ever been green is indistinguishable
from one that is wrong.

So: write the properties, watch them fail, record the counterexamples, hand
them to RFC-065.

## 3. Slices

| Slice | Content | Ends with |
|---|---|---|
| **P1** | `proptest` dev-dependency on `omriss-core`; the document generator | generator produces the shapes in §4, demonstrated |
| **P2** | Property 1 (replace-then-undo round trip) | **failing**, with a shrunk counterexample |
| **P3** | Property 2 (structural op node-count and title invariants) | **failing**, with shrunk counterexamples for promote, move, join |
| **P4** | `cargo-fuzz` target over `formats::json::scanner::parse` | builds; documented run with no crash |
| **P5** | `TESTING.md` property-testing section | states when a property is required |

P4 may be dropped from the 0.17.0 gate if it proves awkward to wire; say so
rather than half-doing it. P1–P3 are the gate.

## 4. The generator is the deliverable, not an afterthought

Uniform random Markdown will not find these defects. The generator's entire
value is its bias, and it must produce, with meaningful frequency:

- documents **ending in a bare heading with no body and no trailing newline** —
  this exact shape is AUDIT-0170-002;
- documents with **no trailing newline** at all — AUDIT-0170-003;
- heading levels 1–6 **including skips** (`#` then `###`);
- **empty bodies** between consecutive headings;
- **CRLF** as well as LF, and occasionally mixed;
- occasional **setext** headings;
- headings whose titles contain **inline markup** — emphasis, code spans,
  links with URLs — since AUDIT-0170-004 destroys exactly those;
- **root-parented sections with following siblings** — AUDIT-0170-005.

Document the bias in a module comment. A reviewer must be able to check that
the generator can reach a given defect without running it.

## 5. The properties

```text
P1  For any document source S and any node N in its outline:
    replacing N's body with any string B, then undoing,
    reproduces S byte-for-byte.

P2  For any document source S and any structural operation O valid on
    node N: the outline node count after O differs from before by exactly
    the delta O defines, and every surviving node's title is unchanged
    unless O is rename or join.
```

P1 catches -002. P2's count clause catches -003 and -005; its title clause
catches -004.

## 6. Constraints

1. **Change no production code.** If a property cannot be expressed without a
   production change, stop and report — that is a finding about the API, and it
   belongs in the review request, not in this slice.
2. **Modify no existing test.** The 239 Markdown baseline and the
   `structural_ops*` golden suites are protected.
3. **Do not fix the defects you find.** They are RFC-065's. Finding a *sixth*
   defect is a good outcome; report it, do not repair it.
4. Default `proptest` case count in CI; note the suite time delta.
5. No `mod.rs`; split files over 500 ELOC.

## 7. Required review-request content

1. **The failing output**, pasted — for each property, the shrunk
   counterexample `proptest` produced, verbatim.
2. A mapping from each counterexample to the audit finding it corresponds to,
   and an explicit statement for any counterexample that maps to **none** —
   that is a new defect and the most valuable thing this slice can produce.
3. The generator's documented bias, and how you confirmed it reaches each of
   §4's shapes.
4. Suite time before and after.
5. Confirmation that `git diff` touches no file under `crates/*/src/`.

## 8. Escalation

Stop and report if:

- a property passes against current `main`. It is then not testing what it
  claims, and shipping it would be worse than shipping nothing;
- `proptest` cannot be added without disturbing the published dependency
  surface of `omriss-core`;
- the generator cannot produce §4's shapes without contorting.
