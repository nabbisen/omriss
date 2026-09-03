<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-004: Source-Preserving Section Replacement

**Project:** omriss — Omriss Editor  
**Milestone:** M0 — Technical Spike  
**Status.** Implemented (v0.1.0); whitespace policy amended 2026-09-01 by RFC-065 — see §4  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  

---

## 1. Summary

Prove the key product invariant: editing one section body can update the canonical Markdown source without rewriting unrelated document bytes.

## 2. Goals

- Define `replace_section_body` for M0.
- Preserve heading line and child sections unless explicitly edited.
- Preserve unrelated bytes exactly.
- Define failure behavior for stale nodes and invalid ranges.

## 3. Non-Goals

- No promote/demote or section movement.
- No collaborative merge.
- No auto-formatting.

## 4. Design

### Operation Semantics

`replace_section_body(node_id, new_body, base_revision)` replaces only the current node's body range.

```text
heading line: preserved
body text: replaced
child sections: preserved
siblings: preserved
ancestors: preserved
```

Example:

```markdown
# A
A body

## A.1
child

# B
B body
```

Replacing body of `A` changes only `A body

` and leaves `## A.1` and `# B` untouched.

### Whitespace Policy

M0 should not auto-normalize blank lines. If the user enters text with or without trailing newline, the operation stores exactly the provided replacement inside the body range. UI may later warn if the result visually collapses into a child heading, but core must not silently rewrite.

> **Amended 2026-09-01 (RFC-065).** The paragraph above is superseded in one
> specific respect: **core now inserts the minimum separator needed to keep the
> document's block structure intact at a body boundary**, and no longer stores a
> replacement that would weld the body onto an adjacent heading.
>
> The original policy was scoped to M0 and paired with a safety net — "UI may
> later warn if the result visually collapses into a child heading" — that was
> never built. What the UI grew instead was a partial mitigation
> (`body_for_focused_editor_commit` appends a newline when the edited section is
> not the last one), which leaves the last section unprotected. That gap is
> AUDIT-0170-002: typing into a file that ends in a bare heading writes the
> user's prose into the heading and silently renames the outline row.
>
> "Visually collapses" also understated it. The collapse is not visual: the
> following heading stops being a heading, in the bytes, and stays that way on
> save and reopen. Verified against `setext.md`, where replacing one body welds
> the next setext heading into the paragraph above it and produces a heading
> titled `"REPLACED-BODY-MARKER Setext Two"`.
>
> Everything else in this policy stands. Core still stores the caller's bytes
> verbatim, still performs no blank-line normalization, and still never reflows,
> trims, or reformats. The amendment authorises exactly one thing: adding a line
> break where its absence would destroy structure the user did not edit. That is
> narrower than normalization and is required by this RFC's own premise —
> replacing section A must leave `# B` untouched, and `# B` ceasing to be a
> heading is not "untouched".
>
> Once core guarantees this, the UI's newline-append becomes redundant and
> should be removed; it is the direct cause of the draft-divergence defects in
> RFC-069 §2.1.

## 5. Internal Design Notes

### API Sketch

```rust
pub fn replace_section_body(
    &mut self,
    id: NodeId,
    replacement: &str,
    base_revision: DocumentRevision,
) -> Result<EditResult, EditError>;
```

`EditResult` contains:

```rust
pub struct EditResult {
    pub old_revision: DocumentRevision,
    pub new_revision: DocumentRevision,
    pub replaced_range: ByteRange,
    pub new_range: ByteRange,
    pub reindexed: bool,
}
```

After replacement, the document must re-index before returning success. If re-index fails, M0 should retain text and return an error state that can be shown in raw source view. Later RFCs may introduce transaction rollback.

## 6. Validation and Test Plan

- Golden test: prefix and suffix outside replaced body are byte-identical.
- CRLF document replacement preserves unrelated CRLF bytes.
- Stale revision returns `RevisionMismatch`.
- Invalid node ID returns `NodeNotFound`.

## 7. Acceptance Criteria

- M0 can edit a focused section body and save a valid Markdown file.
- Byte-preservation tests pass for representative fixtures.
- No AST serialization is used to produce saved Markdown.
## 8. Dependencies

- RFC-002
- RFC-003

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
