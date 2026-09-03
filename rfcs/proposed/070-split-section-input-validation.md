# RFC-070: `split_section` Input Validation

**Project:** omriss — Omriss Editor
**Milestone:** M13 — published-API correctness (0.18.0)
**Status.** Proposed
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
