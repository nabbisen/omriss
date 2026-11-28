# RFC-070: `split_section` Input Validation

**Project:** omriss — Omriss Editor
**Milestone:** M13 — published-API correctness (0.18.0)
**Status.** Implemented (0.18.0) — `d131b6e`, `4e65200`, `daebed0`, `4d5080f`: shared title validation; an insertion never cuts through heading syntax (§9); RFC-066's P4, strengthened (§10); finding D, a multi-line setext rename, fixed. No `#[ignore]` remains in the repository.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-025, RFC-026, RFC-065
**Related RFCs:** RFC-006, RFC-058, RFC-066

---

## 1. Summary

`split_section` accepts two inputs it never validates: a **title**, which may
contain newlines and heading markers, and an **offset**, which may point inside
a descendant's own heading line. Both corrupt the document. Neither is reachable
from the current GUI; both are fully reachable by any consumer of the published
`omriss-core` API.

`rename_section` validates its title carefully. `split_section`, which creates
headings from the same kind of input, validates nothing.

**Source:** AUDIT-0170-033 (title), and a seventh defect found by RFC-066's P2
property during RFC-065's implementation (offset).

## 2. The defects

### 2.1 Unvalidated title

`rename_section` trims, rejects empty titles, and rejects embedded newlines.
`split_section` does none of it, and is reached from
`add_top_level_section`, `append_child_to_focused`, `add_after_focused`, and
`MarkdownAdapter`'s Add commands.

- Title `"Evil\n# Injected H1\n\ntext"` produces three extra outline nodes.
- Title `""` produces a heading line with a trailing space.

### 2.2 Unvalidated offset

`split_section` bounds the offset against the node's **`full_range`**, not its
body:

```rust
let max_offset = full.end - body.start;
```

`full_range` spans every descendant, so an offset far beyond the body is
accepted and can land inside a child's heading line. Found by RFC-066's P2,
shrunk to:

```text
source        "日本語 heading\n==="        target: the synthetic root
split_section(root, offset_in_body = 9, new_title = "", level = H1)

after         "日本語\n# \n\n heading\n==="
outline       [root, "" (new H1), "heading"]     original title carved in half
```

The heading `"日本語 heading"` becomes `"heading"`, with `"日本語"` orphaned above
the inserted heading.

Root's empty `body_range` makes the smallest reproduction, but the computation is
wrong for **any** node with children: `full_range` always spans its descendants.

## 3. Why this is not part of RFC-065

RFC-065 is about **boundaries** — the seam between two pieces of text, fixed by
inserting a separator. This is about **positions and content**: whether an
insertion point is legal at all, and whether a title is well-formed.

`joining_separator` answers "is there enough line separation here." It cannot
answer "is this a legal place to cut", which requires the outline's descendant
ranges rather than the two adjacent strings. Different question, different data,
different home.

`split_section`'s own seams are already correct — its inserted heading carries an
unconditional leading newline and trailing blank line — which is why RFC-065's
B6 touched only its newline convention.

## 4. Design

### 4.1 Shared title validation

Extract `rename.rs`'s validation into `preflight.rs` as `validate_title`, and
call it from both `rename_section` and `split_section`. One rule, one place —
the same reasoning RFC-065 §4.1 applies to boundaries.

### 4.2 Offset validation

Bound the offset against the **body**, and reject any absolute insertion point
falling strictly inside a descendant's `heading_range`:

```text
for child in the target's full descendant set:
    if insert_pos > child.heading_range.start
        && insert_pos < child.heading_range.end:
        return Err(StructuralEditError::InvalidSplitOffset)
```

A new error variant rather than a reused one: "the offset is inside a heading" is
not "the offset is out of range", and a library consumer needs to tell them
apart.

For setext headings `heading_range` spans both the title line and the underline,
so the check covers cutting between them — a distinct sub-shape needing its own
test.

## 5. Non-goals

1. **No clamping.** Silently moving a caller's offset to a nearby legal position
   is worse than refusing: the caller cannot tell the edit did something other
   than what was asked. Refuse, and let the caller choose.
2. **No GUI change.** Neither defect is GUI-reachable — the cursor offset is
   always inside the edited body, and the title dialog blocks empty input and
   strips newlines. This RFC hardens the library, and the GUI's own guards stay.
3. **No new placement semantics.** Real child placement is RFC-058's.

## 6. Validation and test plan

- Every `rename_section` title rejection is rejected identically by
  `split_section`.
- An offset inside an ATX child heading is refused.
- An offset between a setext title and its underline is refused.
- An offset legally inside the body still splits correctly — the guard must not
  narrow what already works, and the `structural_ops*` suites prove it.
- **RFC-066's `split_increases_count_by_one_and_preserves_titles` passes with
  its `#[ignore]` removed.** That is this RFC's primary acceptance evidence, and
  the last `#[ignore]` in the property suite.

## 7. Acceptance criteria

1. Title validation shared between rename and split.
2. Offset bounded by the body, and refused inside any descendant heading range.
3. `InvalidSplitOffset` added and documented.
4. RFC-066's split property passes un-ignored; no `#[ignore]` remains anywhere.
5. `structural_ops*` and the 239 baseline pass unmodified.

## 8. Against today's code (2026-11-28)

Read again before handing it over. §4.2 needs correcting in three ways.

1. **`InvalidSplitOffset` already exists**, documented as "split offset falls
   outside the section's body range". The code bounds the offset by
   `full_range`, so the code disagrees with its own documentation. **No new
   variant**: §4.2's argument for one assumed it did not exist. Acceptance
   criterion 3 becomes "the variant does what its documentation says".
2. **One rule, not two.** `offset_in_body` is an offset into the section's own
   body. Bounding it by the body is the whole rule. A descendant's heading is
   never inside a section's own body, so §4.2's loop over descendants cannot
   fire once the bound is right. It is not written. The property and the two
   heading tests (ATX child, setext title and underline) show the bound alone
   is enough.
3. **Two more ways an offset can be illegal inside the body,** both refused
   with the same variant and each with its own test:
   - an offset that is **not on a character boundary** (inside `日`). Nothing
     checks it today; the report says what happens now (a panic or an edit
     error);
   - an offset **between the CR and the LF of a CRLF**, which would split one
     line ending into two (RFC-084's ground).

§4.1 stands: `validate_title` moves to `preflight.rs`, and rename and split
both call it.

**RFC-066's P4 travels with this item** (ROADMAP item 11). RFC-066 is in
`done/`, and its §3.1b records P4 as 0.18.0 work. Rename and join get the
property "every byte of S outside N's own heading line is unchanged". **If
join legitimately changes anything else,** such as a separator at the seam that
RFC-065's `joining_separator` decides, the property states exactly that change
and asserts it. It does not exempt it. If the change cannot be stated exactly,
stop and report: an exemption is a place where nothing is watching
(RFC-066 §3.1b).

## 9. §8 was wrong in two places (2026-11-28)

The dev team stopped on PR 1 and measured both errors. They are the
architect's.

- **§8.2's "the body bound is the whole rule" breaks the application.** Every
  Add command (`append_child_to_focused`, `add_after_focused`,
  `add_top_level_section`, and core's `add_inside` and `add_after`) passes
  `full_range.end - body_range.start` deliberately, to append after a
  section's children. With the body bound, six existing tests fail. §5.2's "not
  GUI-reachable" was true of the cursor offset only. I read the function, not
  its callers: the same method error RFC-067 §3.2 recorded.
- **§8.3's "nothing checks a character boundary" was false.** The code already
  had `is_char_boundary`, and a test now pins it.

### Decision: §4.2 as first written, with honest names

The offset may be anywhere in the section, its body and its subsections, as
the callers need. It is refused when it is:
- outside the section;
- strictly inside any heading line (`heading_range`), which covers a setext
  title and its underline;
- inside a character;
- between CR and LF.

That is the invariant that matters: **an insertion never cuts through
existing heading syntax.** The dev team's prototype passes the property with
its `#[ignore]` removed, at default cases and at 5,000, and keeps every
existing test green. Without the rule, the property fails.

Not chosen: a second way for Add commands to say "after the children", with a
body bound for everything else. That is a wider API change, inside the 0.18.0
freeze, for no more safety than the rule above.

**The names follow:**
- `split_section`'s parameter `offset_in_body` becomes `offset`, documented
  as "measured from the start of the section's body; anywhere in the section,
  its subsections included". Renaming a positional parameter does not break
  callers.
- `InvalidSplitOffset`'s documentation lists exactly the four refusals above.
- The CHANGELOG says what a library caller now gets refused.

## 10. Decisions on PR 1 part 2 and P4 (2026-11-28)

Accepted.
- The offset rule is §9's. The property is un-ignored and not vacuous: 3,088
  of 5,000 cases split. No `#[ignore]` remains in the repository.

**P4 as RFC-066 §3.1b wrote it did not catch §2.8. The dev team measured that
and said so.** §2.8's damage is *inside* the heading line, which "every byte
outside N's heading line" permits. The strengthened property is accepted. Each
operation also states what its own line becomes:
- rename: the node is titled exactly the requested title;
- join of a setext heading: the title line, verbatim.

It catches each reverted guard in 8 of 10 runs, and 0 of 10 when nothing is
reverted. Join writes nothing at the seam, so no exemption was needed there.
RFC-066 §3.1b's own lesson held one level deeper: a property must pin what the
operation is *for*, not only what it leaves alone.

**Finding D: renaming a multi-line setext heading replaces only its first
line.** `"para\nsecond line\n===\n"` renamed to "New" reads "New second line".
The user asked for "New", and the Document Map showed "para second line" as one
title. It is reachable from the GUI.

**Decision: rename replaces the whole title.** All of a setext heading's title
lines become the one requested line. The underline and everything after it
are untouched. Refusing would leave such a heading impossible to rename, for
no safety gain. **PR 3** builds it, and P4's one exemption (headings of at most
two lines) goes. Join keeps refusing a multi-line setext heading
(`UnsafePreservation`): that is an honest refusal, not a wrong result.
