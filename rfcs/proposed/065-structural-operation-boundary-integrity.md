# RFC-065: Structural Operation Boundary Integrity

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-004, RFC-023, RFC-024, RFC-025, RFC-026
**Related RFCs:** RFC-002, RFC-006, RFC-018, RFC-066

---

## 1. Summary

Five structural operations assemble new source text by concatenating byte
ranges without reasoning about the boundary between them. When a range does not
end in a newline — which happens whenever a file lacks a trailing newline, or a
heading is the last line — the concatenation welds two lines together and
destroys a heading, a title's markup, or the user's prose.

Two of these are release-blocking under `RELEASE_CHECKLIST.md`'s own list. All
five share one root cause and one shape of fix.

**Source of this RFC:** AUDIT-0170-002, -003, -004, -005, and -013, each
reproduced independently by the architect before acceptance. Verbatim outputs
below are from that reproduction, not from the audit report.

## 2. The defects

### 2.1 Typing into a bare trailing heading (Blocking)

`replace_section_body` splices into a `body_range` that is empty and begins
immediately after the title text, so the body lands *inside the heading line*.

```text
open   "# One\nbody\n\n# Last"     focus "Last"
commit "typed text"
source "# One\nbody\n\n# Lasttyped text"
outline ["One", "Lasttyped text"]
```

The primary editing path, triggered by ordinary typing. The heading absorbs the
prose and the outline row silently renames itself.

`docs/src/source-preservation.md` names missing trailing newlines explicitly
among what is preserved bit-for-bit. This contradicts it directly.

### 2.2 Moving a section with no trailing newline (Blocking)

`move_section` concatenates three slices with no separator logic.

```text
before "# A\nalpha\n# B\nbeta"     move B before A
after  "# B\nbeta# A\nalpha\n"
outline ["B"]
```

Heading A is gone. Two clicks in the Document Map destroy it. Undo recovers it
if the user notices; the vanishing outline row makes noticing likely, not
certain.

### 2.3 Promote teleports a top-level section to end of file (Major)

`level.rs` relocates to `parent.full_range.end` whenever a following sibling
exists. For a root-parented section, that offset is the end of the file.

```text
before "## A\nalpha\n\n## B\nbeta\n"     promote A
after  "## B\nbeta\n# A\nalpha\n\n"
```

### 2.4 Join rewrites the heading as flattened plain text (Major)

`merge_with_prev_sibling` replaces the heading's *source* with `node.title`,
which the indexer produced by flattening inline events.

```text
before "## One\nalpha\n\n## **bold** and [link](http://x)\nbeta\n"
after  "## One\nalpha\n\nbold and link\nbeta\n"
```

The link URL is destroyed. Emphasis, code spans and inline HTML likewise.

### 2.5 Inserted headings hardcode LF (Minor)

`split_section` writes `\n` regardless of the file's line endings, so "Add
section" on a CRLF file produces mixed endings — contradicting
`source-preservation.md`'s "CRLF and LF line endings, exactly as found".

Scope note: this is confined to *inserted* text. Existing bytes are preserved
correctly, and the `crlf.md` fixture proves it. `NewlinePolicy` and
`had_trailing_newline` exist on the UI-layer `FileTextProfile` and have no
reader anywhere — see §6.

### 2.6 Deleting a section welds its neighbours together (Major)

Found by RFC-066's P2 property, not by the audit. `delete_section` removes the
node's `full_range` with `apply_replacement(range, "")` and no separator logic,
so the bytes that were on either side become adjacent.

```text
before "a\n===\n0\n## A\nA\n==="   titles ["a", "A", "A"]
delete the ATX "## A"
after  "a\n===\n0\nA\n==="          titles ["a", "0 A"]
```

The deleted heading was the only thing separating the first section's body
(`"0"`) from the following setext heading's text (`"A"`). Once removed, those
become one two-line paragraph, and the `===` underneath turns the whole thing
into a single setext heading titled `"0 A"`. **Two titles lost, one corrupted
title invented.**

This is the same root cause as §2.1–2.4 through a sixth call site, and it is a
primary operation — every "delete section" action in the Document Map reaches
it. Reproduced independently by the architect before acceptance.

## 3. Root cause

Every one of these is the same mistake: **an operation that knows its ranges but
not its edges.** The code asks "what bytes am I inserting, and where?" and never
"what is immediately to the left and right of that position, and is the result
still well-formed Markdown?"

That question has no home in the current design. Each operation answers it
independently, or forgets to.

## 4. Design

### 4.1 A single boundary helper

Introduce one place that answers the edge question, in
`core/doc/structural/preflight.rs`:

```rust
/// Returns the separator that must be inserted between `left` and `right`
/// for the result to remain structurally well-formed, using the document's
/// own newline sequence.
pub(crate) fn joining_separator(left: &str, right: &str, newline: &str) -> &'static str
```

Every splice site routes through it: **both** of `replace_section_body`'s edges,
`move_section`'s three seams, `level.rs`'s promote-relocation branch,
`split_section`'s inserted heading, and — the site this RFC originally missed —
`delete_section`'s **removal seam**, where the question is not what is being
inserted but what becomes adjacent once a range is gone.

That omission is instructive: the first four sites are all insertions, and the
helper was scoped to insertion. A deletion creates a boundary just as an
insertion does. `joining_separator` must take the two sides, not the inserted
text.

**"Left edge" was also wrong**, and for the same reason. An earlier draft named
only `replace_section_body`'s left edge, because §2.1's defect welds the
inserted body onto the heading *before* it. RFC-066's P3 found the mirror case
on the right, verified directly:

```text
before  "intro\n\n# H\nbody\n"     replace the root's body with "intro2"
after   "intro2# H\nbody\n"          the "# H" heading is destroyed
```

Editing a document's preamble welds it onto the following heading. Both edges of
every insertion need the same question asked — which is what a
`joining_separator(left, right, newline)` signature is for, but only if §4.1
says so.

A helper rather than five local fixes, because five local fixes is how this
happened: the same reasoning was needed in five places and written in none.

### 4.2 Join preserves heading source

Strip only the marker, keeping the remaining source bytes of the line — for ATX
the inverse of the span `atx_title_range` already computes; for setext, remove
the underline line. Any heading shape the stripper does not recognise is refused
with `UnsafePreservation`. That variant exists for exactly this.

### 4.3 Promote checks for a real parent

```rust
let relocate = delta < 0
    && has_following_sibling(outline, id)?
    && node.parent_id.is_some_and(|p| p != outline.root_id());
```

### 4.4 Derive the newline from the target

Use the technique `plain_heading_replacement` already applies correctly at
`delete_split_merge.rs:146` — read the newline from the target's own
`heading_range` suffix rather than assuming LF.

## 5. Validation and test plan

Example-based tests for each defect are necessary and **not sufficient** — every
one of these shipped past 431 example-based tests. RFC-066's round-trip property
is the real gate, and this RFC should land with it.

- Fixture `heading_only_no_trailing_newline.md` added to the catalog. The
  existing `no_trailing_newline.md` has a body, so it never reaches the 2.1
  shape — which is why
  `committing_last_section_body_without_trailing_newline_remains_verbatim`
  passes while the defect is live. That test is not wrong; it is aimed
  elsewhere.
- A no-trailing-newline move fixture, and a CRLF split fixture.
- Join preserves link URLs, code spans, emphasis; refuses unrecognised shapes.
- Promote on a root-parented section with a following sibling stays in place.
- The `structural_ops*` golden suites remain **unmodified**. Any change there
  means the fix is wrong.

## 6. Question for the owner

`NewlinePolicy` and `had_trailing_newline` are detected on every open and read
by nothing — verified by grep across both GUI crates; only `had_utf8_bom` is
consumed, and `NewlinePolicy::label()` has no caller.

Make them load-bearing (§4.4 gives them a consumer), or delete them? Leaving
detected-but-unread fields is a false signal that the concern is handled — which
is precisely how 2.5 shipped.

**Recommendation: make them load-bearing.** The concern is real, the plumbing
already exists, and §4.4 needs exactly this value.

## 7. Acceptance criteria

1. All six defects in §2 fixed, each with a regression test.
2. RFC-066's P2 and P3 pass, and no property carries `#[ignore]` any more —
   removing that attribute is this RFC's primary acceptance evidence.
3. `structural_ops*` golden suites pass unmodified.
4. `source-preservation.md`'s claims about trailing newlines and line endings
   are true of the implementation — the doc is right; the code changes.
5. No new `mod.rs`; files over 500 ELOC split.
