# RFC-066: Property-Based and Fuzz Testing

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate
**Status.** Proposed
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
```

P1 catches AUDIT-0170-002. P2 catches -003 and -005 — both destroy a heading, so
both change node count by an amount their operation does not define. -004 is
caught by P2's title clause.

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

- P1 and P2 pass over the generator at the default case count.
- Both properties **fail** against the pre-RFC-065 code. A property that passes
  before the fix is not testing what it claims, and this must be demonstrated in
  the review request rather than asserted.
- Every shrunk counterexample found during implementation is added to the
  fixture catalog by name.
- The fuzz target builds and runs for a documented duration without a crash.

## 6. Acceptance criteria

1. P1 and P2 implemented, passing after RFC-065, demonstrably failing before it.
2. A generator whose bias is documented and reviewed.
3. `cargo-fuzz` target over the JSON scanner, with a run recorded.
4. `TESTING.md` gains a property-testing section stating when a property is
   required rather than optional.
5. CI runs the properties; total suite time increase recorded in the review
   request.

## 7. Note on RFC-031

RFC-031 is marked Implemented while its acceptance criterion "regression
thresholds documented before enforcement" was never met, and its central
deliverable — calibrated millisecond thresholds — was never produced. The audit
measured the first numbers on record.

That is a separate gap from this RFC, but the same family: a measurement layer
that was designed and not built. RFC-031's Status field should record it, in the
pattern RFC-016's header now uses. Handled as part of the schedule update, not
here.
