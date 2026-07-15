# RFC-053: Document Format Adapter Architecture

**Project:** omriss — Omriss Editor
**Milestone:** M11 — Format Adapter Foundation
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-052
**Related RFCs:** RFC-054, RFC-055, RFC-056

---

## 1. Summary

This RFC defines a document format adapter architecture for omriss.

The adapter boundary allows omriss to support multiple structured plain-text formats while preserving one product model:

```text
Source text is canonical.
A format adapter derives a Document Map from that source text.
The right panel edits focused content through source-preserving operations.
```

Markdown remains the first and primary adapter. JSON, TOML, and possible YAML support must plug into the same boundary instead of creating separate application architectures.

## 2. Motivation

Markdown, JSON, TOML, and YAML have different syntax and editing rules, but the omriss user experience should stay consistent:

```text
Open file → view structure → select item → edit focused content → save safely
```

Without a format adapter boundary, the UI and core would become full of format-specific conditionals. That would make the app harder to test and would risk damaging the source-preserving principle.

## 3. Design principles

### 3.1 Source text is owned by the document session

Adapters may inspect source text and propose edits, but the document session owns canonical text, dirty state, undo/redo, save, and external file checks.

### 3.2 Derived structure is disposable

The Document Map is rebuilt after committed edits. It must not become the source of truth.

### 3.3 Adapters must be conservative

If an adapter cannot safely produce or apply an edit, it must refuse and return a friendly error. It must not guess.

### 3.4 Full-file serialization is not the normal save path

Adapters must perform focused source-range replacement or structure-aware source edits. They must not parse into a generic data object and serialize the entire file by default.

## 4. Architecture overview

```text
AppShell
  ├─ DocumentMapPanel
  ├─ FocusedContentPanel
  ├─ DialogHost
  └─ StatusBar
        │
        ▼
DocumentSession
  ├─ SourceText
  ├─ ActiveFormatAdapter
  ├─ DocumentStructure
  ├─ FocusState
  ├─ UndoRedoHistory
  └─ SaveController
        │
        ▼
Format adapters
  ├─ MarkdownAdapter
  ├─ JsonAdapter
  ├─ TomlAdapter
  └─ YamlExperimentalAdapter
```

## 5. Core data types

The public boundary names and enum variants in this section are canonical (RFC-053 is the type authority). Private representation details may vary during implementation provided the public API and RFC semantics are unchanged.

```rust
pub enum DocumentFormat {
    Markdown,
    Json,
    Toml,
    YamlExperimental, // spike / read-only candidate; never `Yaml` or `YamlCandidate`
    PlainText,        // show-source only; no synthetic structure
    Unsupported,      // friendly unsupported-file message
}

// NodeId is the EXISTING shipped core identity type from RFC-006
// (`omriss_core::NodeId(pub u64)`). It is reused as-is, never redefined, and never
// shown in normal UI. There is no separate `FocusedNodeId`; focus is a state
// role over `NodeId`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

pub struct SourceText {
    text: String,
    line_ending: LineEnding,
    revision: u64,
}

pub enum LineEnding {
    Lf,
    Crlf,
    Mixed,
}
```

## 6. Document structure model

```rust
pub struct DocumentStructure {
    pub format: DocumentFormat,
    pub root_id: NodeId,
    pub nodes: Vec<StructureNode>,
    pub revision: u64,
}

pub struct StructureNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub title: String,
    pub kind: StructureNodeKind,
    pub depth: usize,
    pub source_range: Option<ByteRange>,
    pub editable_range: Option<ByteRange>,
    pub children: Vec<NodeId>,
    pub capabilities: NodeCapabilities,
}

pub enum StructureNodeKind {
    DocumentRoot,
    MarkdownSection,
    Group,
    List,
    Value,
    RawRegion,
    Unsupported,
}

pub struct NodeCapabilities {
    pub can_select: Capability,
    pub can_edit_content: Capability,
    pub can_add_inside: Capability,
    pub can_add_after: Capability,
    pub can_rename: Capability,
    pub can_move_up: Capability,
    pub can_move_down: Capability,
    pub can_move_inside_previous: Capability,
    pub can_move_out_one_level: Capability,
    pub can_join_with_previous: Capability,
    pub can_delete: Capability,
    pub can_show_plain_text: Capability,
}

// Granular per-action capability lets the Document Map disable exactly one menu
// item (e.g. allow `can_move_down` but not `can_move_up`) instead of gating all
// movement together. Capabilities are populated by `build_structure` (which has
// full source + tree context), so there is no separate stateless lookup.
pub enum Capability {
    Allowed,
    Disabled { reason: CapabilityReason },
    Hidden,
}

// Typed, core-owned reason. It must NOT be a UI message key: `NodeCapabilities`
// is produced in the `omriss-core` crate, which never depends on `omriss-ui`
// (RFC-001). `omriss-ui` maps `CapabilityReason` to localized catalog text
// (RFC-043) at render time.
pub enum CapabilityReason {
    RootNode,
    NoSibling,
    NoParent,
    ReadOnlyFormat,
    ExperimentalFormat,
    UnsafePreservation,
    UnsupportedForFormat,
    ExternalChangeConflict, // session-overlay only — see note below
}

`ExternalChangeConflict` is **not emitted by format adapters**. It is applied as
a session-level overlay after adapter capabilities are built, when external
file-modification state disables otherwise-valid operations. (A dedicated
`SessionCapabilityReason` may split this out later; for now the rule is that
adapters never produce it.)
```

## 7. Adapter trait

```rust
pub trait DocumentFormatAdapter {
    fn format(&self) -> DocumentFormat;

    fn detect(path: &std::path::Path, text: &str) -> DetectionConfidence
    where
        Self: Sized;

    fn build_structure(&self, source: &SourceText) -> Result<DocumentStructure, StructureError>;

    fn focused_content(
        &self,
        source: &SourceText,
        structure: &DocumentStructure,
        node_id: NodeId,
    ) -> Result<FocusedContent, FocusError>;

    fn validate_focused_edit(
        &self,
        source: &SourceText,
        structure: &DocumentStructure,
        node_id: NodeId,
        draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError>;

    fn apply_validated_edit(
        &self,
        source: &mut SourceText,
        edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError>;

    fn structure_command(
        &self,
        source: &mut SourceText,
        structure: &DocumentStructure,
        command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError>;
}
```

This trait is intentionally broad. The implementation may split it into smaller traits if that is cleaner:

- `FormatDetector`;
- `StructureBuilder`;
- `FocusedContentProvider`;
- `FocusedEditValidator`;
- `StructureCommandHandler`.

## 8. Focused content model

```rust
pub enum FocusedContent {
    MarkdownSection {
        title: String,
        body: String,
        preview_available: bool,
    },
    StructuredValue {
        title: String,
        value_kind: ValueKind,
        display_text: String,
        editable_text: String,
    },
    StructuredGroup {
        title: String,
        child_count: usize,
        summary: String,
        raw_text_available: bool,
    },
    Unsupported {
        title: String,
        reason: FriendlyReason,
        raw_text_available: bool,
    },
}

pub enum ValueKind {
    Text,
    Number,
    OnOff,
    NoValue, // JSON null / absent value; user-facing "No value", distinct from empty text ""
    RawText,
}
```

The UI must translate this into plain labels.

## 9. Edit model

### 9.1 Focused replacement

Most content edits should become focused replacement edits:

```rust
pub struct ValidatedEdit {
    pub node_id: NodeId,
    pub base_revision: u64,
    pub replacement_range: ByteRange,
    pub replacement_text: String,
    pub description: EditDescription,
}
```

Before applying:

- base revision must match;
- replacement range must be valid UTF-8 boundary;
- replacement result must be valid for the format;
- unrelated bytes must remain untouched.

### 9.2 Structure command

Structure commands may require more than one range edit.

```rust
pub enum StructureCommand {
    AddInside { target: NodeId, spec: NewNodeSpec },
    AddAfter { target: NodeId, spec: NewNodeSpec },
    Rename { target: NodeId, new_name: String },
    Move { target: NodeId, direction: MoveDirection },
    JoinWithPrevious { target: NodeId },
    Delete { target: NodeId },
}

pub enum MoveDirection {
    Up,
    Down,
    InsidePrevious, // Markdown: demote
    OutOneLevel,    // Markdown: promote
}
```

Adapters may return `Unsupported` for commands that are not safe for a format.

Movement uses one vocabulary (`MoveDirection`); there are no separate
`Promote` / `Demote` commands. For Markdown the adapter maps these onto the
already-shipped core operations rather than reimplementing them:
`Move { InsidePrevious }` → demote (RFC-023), `Move { OutOneLevel }` → promote
(RFC-023), `Move { Up | Down }` → section move via `MoveTarget` (RFC-024),
`JoinWithPrevious` → merge (RFC-025), and `AddInside` / `AddAfter` / `Rename` /
`Delete` → the existing section operations. The unified command set is a
UI-facing layer; it does not refactor or regress shipped Markdown editing.

### 9.3 Draft state and the invalid-draft gate

A focused draft carries one of three states. This is editor-local; it is not a
second user-visible saved-document dirty state.

```rust
pub enum DraftState {
    Clean,
    ValidUncommitted,
    InvalidUncommitted,
}
```

Markdown body drafts are at worst `ValidUncommitted` (any text is valid as a
section body) and commit on navigation, save, preview, search, or blur.
Structured drafts may be `InvalidUncommitted`; while invalid the editor blocks
navigation, save, and preview, keeps focus on the field, and shows plain
guidance (RFC-050). Undo coalesces to one entry per focused-edit session.

## 10. Detection policy

Format detection order:

1. explicit extension;
2. lightweight content validation if extension is ambiguous;
3. user choice if needed;
4. unsupported file message.

```rust
pub enum DetectionConfidence {
    No,
    Maybe,
    Likely,
    Certain,
}
```

If a file extension and content disagree, omriss should avoid destructive behavior.

Example:

```text
This file is named .json, but it does not look like valid JSON.

[Show plain file text] [Cancel]
```

## 11. Error model

Adapters return structured internal errors. The UI maps them to friendly messages.

```rust
pub enum StructureErrorKind {
    InvalidSyntax,
    UnsupportedFeature,
    UnsafeRange,
    TooLarge,
    InternalInvariantFailed,
}
```

Normal UI must not expose parser internals.

| Error kind | User-facing message |
|------------|---------------------|
| InvalidSyntax | “This file does not look valid.” |
| UnsupportedFeature | “This part can be viewed, but safe editing is not ready yet.” |
| UnsafeRange | “This change could not be made safely.” |
| TooLarge | “This file is too large to show this way.” |
| InternalInvariantFailed | “Something went wrong. Your file was not changed.” |

## 12. Rebuild and revision policy

After every applied edit:

1. source text revision increments;
2. adapter rebuilds document structure;
3. focus is restored by best-effort stable identity;
4. UI draft is refreshed;
5. dirty state is updated;
6. undo entry is recorded.

For M0 structured support, full rebuild is acceptable. Incremental indexing is out of scope until performance measurements prove a need.

## 13. Node identity policy

Node identity must be stable enough for UI focus after focused content edits.

Recommended strategy:

- Markdown: ordinal path + heading position strategy already used or equivalent.
- JSON: JSON pointer-like path based on object keys and array indexes.
- TOML: table/key path based on TOML key path and occurrence ordinal for repeated tables.
- YAML: feasibility spike must evaluate stable identity separately.

Node IDs must never be exposed in normal UI.

## 14. Preservation tests

Every adapter that supports editing must provide tests proving:

- unrelated source text remains byte-identical;
- line ending style is preserved;
- comments are preserved where the format supports comments;
- ordering is preserved;
- invalid edits are rejected before source mutation;
- undo restores exact previous source text.

Example golden assertion:

```rust
assert_eq!(&before[..range.start], &after[..range.start]);
assert_eq!(&before[range.end..], &after[range.end + delta..]);
```

Actual tests should use clearer helper functions rather than relying only on this sketch.

## 15. Adapter-specific notes

### 15.1 Markdown

Markdown adapter remains the primary implementation. Existing section replacement and structural operations should be adapted into this architecture when low-risk. Concretely, the Markdown adapter wraps the shipped core — `Document` / `replace_section_body` (RFC-004/005), the heading tree (RFC-007), promote/demote (RFC-023), section move via `MoveTarget` (RFC-024), and split/merge/delete (RFC-025) — behind the adapter trait rather than rewriting it, so Markdown behavior and byte-preservation golden tests cannot regress.

### 15.2 JSON

JSON adapter should be implemented first. It should build a structural index with byte ranges for values and containers.

### 15.3 TOML

TOML adapter should preserve comments, key order, table layout, and inline forms. Use a lossless editing strategy rather than full normalization.

### 15.4 YAML

YAML adapter must start as a candidate/read-only feasibility adapter. Editable YAML is not approved by this RFC.

## 16. Security and safety considerations

- Do not execute file contents.
- Do not fetch schemas or network resources automatically.
- Do not process external includes automatically.
- Avoid stack overflow on deeply nested files by enforcing reasonable recursion limits.
- Avoid excessive memory use on very large files.
- Preserve user files by using existing atomic save and external modification checks.

## 17. Acceptance criteria

- A format-neutral adapter boundary is defined in core design.
- Markdown can be represented through the adapter model or bridged without changing user behavior.
- JSON RFC-054 can implement against this boundary.
- TOML RFC-055 can implement against this boundary.
- YAML RFC-056 can implement read-only feasibility against this boundary.
- The UI does not call format-specific source rewrite logic directly.
- Adapter errors are mapped to friendly messages.
- Preservation tests are required for every editable adapter.
- Existing RFC-023/RFC-024/RFC-025 Markdown structural-edit tests pass unchanged after the adapter boundary is introduced.

## 18. Open questions

1. Should adapters be static enum dispatch or trait objects?
2. **Resolved.** Format adapters are modules inside the `omriss-core` crate (`omriss_core::formats::{markdown, json, toml, yaml}`). Separate `omriss-json` / `omriss-toml` crates are deferred until a measured need (dependency weight, feature-flag maintenance, or independent release/test).
3. Should adapters support format-specific settings?
4. Should a future plugin system be allowed to register adapters?
5. Should `DetectionConfidence` live in core or a dedicated detection-registry module? (Not blocking.)

## 19. Final decision summary

omriss will support future formats through document format adapters. The source text remains canonical, structures are derived, and all edits must be source-preserving. Full-file serialization is prohibited as the normal save path.
