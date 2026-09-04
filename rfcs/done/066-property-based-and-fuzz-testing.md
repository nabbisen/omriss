# RFC-066: Property-Based and Fuzz Testing

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate
**Status.** Implemented (main, unreleased) — P1, P2 and P3 land with a
documented, continuously-checked generator. One tracked `#[ignore]` remains, on
the `split_section` property, naming **RFC-070**; per criterion 6 a tracked
ignore is a scheduled defect, not a disabled test. The `cargo-fuzz` target is
deferred to 0.18.0 (§3.3): the JSON scanner module is private, and reaching it
needs a visibility change this work correctly refused to make.

In its first outing the suite found four things no one had found by reading:
`delete_section` as a sixth splice site, the right-edge/preamble case, a promote
counterexample destroying two nodes where the hand-written example destroyed
one, and two errors in this RFC's own property definitions.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-040
**Related RFCs:** RFC-004, RFC-026, RFC-034, RFC-042, RFC-064, RFC-065

---

## 1. Summary

Every one of the 431 tests is example-based. The three release-blocking defects
found by the 0.17.0 audit are each a *shape nobody thought to write a fixture
for*, and each would fail a one-line invariant within the first hundred random
inputs.

This RFC adds the missing layer: two round-trip properties over generated
documents, and a fuzz target over the JSON scanner.

**Source of this RFC:** AUDIT-0170-012.

## 2. Why this is the highest-leverage item in the audit

The audit's own closing observation is the argument: *the honesty in this
project is all human — smoke runs, reviews, RFC discipline — and the automation
that would catch what a careful reading misses is exactly what is absent.*

That is precise. The fixture catalog is genuinely adversarial: CRLF, BOM,
setext, skipped levels, no trailing newline, Japanese, HTML, code fences,
duplicate titles. It was written by people thinking hard about edge cases. It
still missed "a heading that is the last line, with no body and no trailing
newline" — because thinking hard about edge cases is not the same as
enumerating them.

Example-based tests encode the cases the author imagined. Properties encode the
invariant itself, and let a generator look for the cases nobody imagined. omriss
has exactly one invariant that matters, stated in the README and in
`docs/src/source-preservation.md`, and it is trivially expressible as a
property.

Adding this **after** fixing RFC-065's defects would be the wrong order: the
properties are how we know the fixes are complete rather than merely aimed at
the five reported shapes.

## 3. Design

### 3.1 The two properties

```text
P1  For any document source S and any node N in its outline:
    replacing N's body with any string B, then undoing,
    reproduces S byte-for-byte.

P2  For any document source S and any structural operation O
    valid on node N: the outline node count after O differs from
    before by exactly the delta O defines, and every surviving
    node's title is unchanged unless O is rename or join.

P3  For any document source S, any node N, and any INERT body B:
    after replacing N's body with B, reading N's body back yields B
    followed only by line-break characters, and no node's title has
    changed.

    Inert means B opens no block construct: no line of B begins with
    a heading marker, an HTML-block opener, a setext underline, a
    fence, a list marker, or a blockquote marker.
```

P2 catches -003, -005 and the `delete_section` defect — all destroy or corrupt a
heading, so all change node count or the title multiset by an amount their
operation does not define. -004 is caught by P2's title clause.

**P3 exists because P1 does not catch AUDIT-0170-002, contrary to what an
earlier draft of this RFC claimed.** That claim was wrong, and the correction
matters enough to state rather than quietly edit.

P1 is a **reversibility** property. AUDIT-0170-002 is an **addressing** defect:
the node's `body_range` is empty and sits *inside* the heading line, so a commit
writes the user's prose into the heading. Undo then replays the exact bytes at
the exact recorded range and restores the source perfectly. Verified directly:

```text
body_range before commit   ByteRange { start: 18, end: 18 }
after commit               "# One\nbody\n\n# Lasttyped text"   title → "Lasttyped text"
body read back             ""            (after writing "typed text")
after undo                 "# One\nbody\n\n# Last"             byte-exact: true
```

Both facts are true at once, and P1 is right to pass. The invariant -002 breaks
is that **`body_range` faithfully addresses the body** — write B, read back B.
That is P3, and the read-back clause is the sharper half: the title corruption is
a consequence of the mis-addressing, not the defect itself.

P1 is kept as specified rather than redefined. Reversibility is a genuinely
valuable property — it is the guard against the history-desync failure of
AUDIT-0170-018 — and weakening a good property to cover for a wrong claim about
it would lose both.

**P3's trailing-line-break clause was added after RFC-065's implementation**, and
is a real weakening that has to be justified rather than waved through. When a
right-edge separator is genuinely required, core writes it inside the replaced
range, and `body_range` is then recomputed as "everything up to the next
heading" (RFC-006) — so the separator becomes part of the body and reads back
with it. There is no third bucket for bytes belonging to neither section.

Exact equality is therefore unsatisfiable in that sub-case, and the clause is
the minimum relaxation that admits it. It does not blunt the property: the
defect P3 exists to catch reads back `""` against a written `"typed text"`,
which is not "B followed by line breaks" under any reading. What the clause
permits is precisely the bytes core is now *required* to add, and nothing else —
no reflow, no trimming, no reordering.

**The inert-body restriction was added after RFC-065's follow-up**, and is also
a correction to this RFC rather than to the code. P3's title clause assumed a
body edit cannot change the outline. It can, legitimately, in **both
directions**, because a body is Markdown:

```text
write "# Injected"  →  outline GAINS a node
write "<!--"        →  the following heading stops being a heading
```

Neither is corruption. Verified: after writing `"<!--"`, the bytes `# Two` are
still literally present and undo is byte-exact — CommonMark simply no longer
*interprets* them as a heading, because the user opened an unclosed HTML block.
That is what the user's own text means.

Restricting B to inert content is what makes the title clause a statement about
**omriss's splice** rather than about Markdown's semantics. The distinction this
property must draw is: did *omriss* destroy structure the user did not touch
(AUDIT-0170-002 — yes), or did the *user's own text* change what their document
means (an unclosed comment — yes, and correctly)? Only the first is a defect,
and only inert bodies isolate it.

The surprise a user gets from the second case is real and is tracked by
RFC-071 — as a warning, which is what RFC-004 always said it should be.

### 3.2 Generator

A Markdown generator biased toward the shapes that break things, not toward
average documents: heading levels 1–6 including skips, bodies that are sometimes
empty, trailing newline present or absent, LF and CRLF, occasional setext, and
— critically — **documents ending in a bare heading**.

Uniform random Markdown would rarely produce the failing shapes. The generator's
value is entirely in its bias, and that bias should be reviewed as carefully as
the properties.

### 3.3 Fuzz target

`cargo-fuzz` over `formats::json::scanner::parse`. The scanner is already the
strongest component in the audit's assessment — strict RFC 8259, nesting limit
enforced before recursing, surrogates and control characters rejected. A fuzz
target is how it stays that way as TOML and YAML land beside it.

### 3.4 Tooling choice

`proptest`. Shrinking is the feature that matters here: a failing 40-line
generated document is not a bug report, and `proptest`'s shrinker reduces it to
the minimal reproducer, which is what goes into the fixture catalog as a
permanent example-based test.

Adds a dev-dependency to `omriss-core` only. `TESTING.md` has no policy on
property testing — this RFC establishes one.

## 4. Non-goals

1. **Not a replacement for the fixture catalog.** Every property failure becomes
   a named fixture. Properties find cases; fixtures pin them.
2. **Not a CI time sink.** Default case count in CI, a longer run available
   manually. Fuzzing is not gated on in CI at all initially.
3. **No property tests over the GUI.** The layer that most needs coverage is the
   one that can be tested without a WebView, and that is where this stops.

## 5. Validation and test plan

- P1, P2 and P3 pass over the generator at the default case count once RFC-065
  has landed.
- **P2 and P3 fail** against the pre-RFC-065 code, demonstrated in the review
  request rather than asserted. P1 passes throughout, for the reason in §3.1.
- While RFC-065 is outstanding, the failing properties carry
  `#[ignore = "fails until RFC-065"]` so `cargo test --workspace` stays green
  and CI keeps its signal. RFC-065 removes the attribute, and that removal is
  its primary acceptance evidence.
- Every shrunk counterexample found during implementation is added to the
  fixture catalog by name.
- The fuzz target builds and runs for a documented duration without a crash.

## 6. Acceptance criteria

1. P2 and P3 implemented, passing after RFC-065, demonstrably failing before it.
2. P1 implemented and passing throughout — it guards a different invariant and
   is not expected to fail (§3.1).
3. A generator whose bias is documented and reviewed.
4. `TESTING.md` gains a property-testing section stating when a property is
   required rather than optional.
5. CI runs the properties; total suite time increase recorded in the review
   request.
6. No property carries `#[ignore]` once RFC-065 has landed, **except** one
   whose reason string names a tracked follow-up RFC. An untracked `#[ignore]`
   is a silently disabled test; a tracked one is a scheduled defect. The
   `split_section` offset property is the only permitted case, tracked by
   RFC-070.

## 7. Note on RFC-031

RFC-031 is marked Implemented while its acceptance criterion "regression
thresholds documented before enforcement" was never met, and its central
deliverable — calibrated millisecond thresholds — was never produced. The audit
measured the first numbers on record.

That is a separate gap from this RFC, but the same family: a measurement layer
that was designed and not built. RFC-031's Status field should record it, in the
pattern RFC-016's header now uses. Handled as part of the schedule update, not
here.
