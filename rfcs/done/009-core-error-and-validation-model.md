<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-009: Core Error and Validation Model

**Project:** omriss — Omriss Editor  
**Milestone:** M1 — Core Document Engine  
**Status.** Implemented (v0.1.0)  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  

---

## 1. Summary

Define predictable, recoverable error behavior for core parsing, indexing, and editing.

## 2. Goals

- Classify errors into user-facing, recoverable internal, and programmer misuse categories.
- Provide stable error enums for UI mapping.
- Avoid panics for document content problems.
- Define validation hooks.

## 3. Non-Goals

- No logging backend decision.
- No localization.
- No telemetry.

## 4. Design

### Error Taxonomy

```rust
pub enum DocumentError {
    InvalidUtf8,
    NodeNotFound(NodeId),
    Range(RangeError),
    Index(IndexError),
}

pub enum EditError {
    RevisionMismatch { expected: DocumentRevision, actual: DocumentRevision },
    StaleNode(NodeId),
    InvalidRange(RangeError),
    IndexAfterEdit(IndexError),
}
```

### User-Facing Mapping

| Core Error | User Message Pattern | Recovery |
|---|---|---|
| `InvalidUtf8` | This file is not valid UTF-8. | Open externally or convert file. |
| `RevisionMismatch` | This section changed before your edit was committed. | Reload focused section. |
| `StaleNode` | This section no longer exists in the current outline. | Return to nearest parent/root. |
| `IndexAfterEdit` | The document was saved as text, but outline refresh failed. | Show raw Markdown view. |

### Validation Levels

- construction validation: `Document::parse`;
- range validation before every edit;
- outline invariant validation after re-index;
- debug-only internal assertions for impossible states.

## 5. Internal Design Notes

### Result Boundary

`omriss-core` returns structured errors. `omriss-ui` decides wording, status display, and modal behavior. The `omriss` app package handles filesystem errors separately but can map them into a common UI error channel.

## 6. Validation and Test Plan

- Each public error variant has at least one unit test.
- UI mapping table has no unmapped public core error.
- Corrupt internal outline fixture fails validation.
- Programmer-only invariant violations are not triggered by normal Markdown input.

## 7. Acceptance Criteria

- No public core method panics on user-provided Markdown.
- Errors can be converted to user-facing messages without string parsing.
- Validation rules are documented beside the relevant structs.

## 8. Dependencies

- RFC-005
- RFC-008

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
