<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-015: File Open and Save Lifecycle

**Project:** omriss — Omriss Editor  
**Milestone:** M3 — File Lifecycle and Recovery  
**Status.** Implemented (v0.3.0)  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  

---

## 1. Summary

Define how Markdown files are opened, tracked, saved, and saved-as across desktop platforms.

## 2. Goals

- Support open, save, save as, and file path state.
- Handle filesystem failures without data loss.
- Define accepted file types.
- Define external modification policy.

## 3. Non-Goals

- No cloud sync.
- No multi-document tabs.
- No binary/non-UTF-8 editor.

## 4. Design

### File Lifecycle State Machine

```text
NoDocument
  -> OpenDialog
  -> Loading
  -> LoadedClean
  -> Editing
  -> LoadedDirty
  -> Saving
  -> LoadedClean | SaveFailed
```

### Open Policy

Accepted extensions by default:

```text
.md, .markdown, .mdown, .txt
```

Opening a file:

1. read bytes;
2. decode according to RFC-018;
3. create `Document` from text;
4. record path, last-known modified time, and clean revision.

### Save Policy

`Save` writes canonical document text to the current path. `Save As` writes to a new selected path and updates current path after success.

### External Modification

M3 should detect when the file modification time differs from the last-known saved metadata before saving. Initial behavior may be conservative:

```text
External change detected -> prompt: overwrite / save as / cancel
```

## 5. Internal Design Notes

### Desktop Boundary

Filesystem operations live in the `omriss` app package or an app service layer, not `omriss-core`. Core only imports/exports text.

## 6. Validation and Test Plan

- Open valid UTF-8 Markdown.
- Save clean document no-op or writes safely.
- Save failure preserves dirty state.
- Save As changes current path only after successful write.
- External modification prompt appears before overwrite.

## 7. Acceptance Criteria

- The user cannot silently lose changes due to a failed save.
- Open/save operations never require custom IPC.
- File lifecycle state is visible through shell status.

## 8. Dependencies

- RFC-010
- RFC-018

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
