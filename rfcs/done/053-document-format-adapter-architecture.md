# RFC-053: Document Format Adapter Architecture

**Project:** omriss — Omriss Editor
**Milestone:** M11 — Format Adapter Foundation
**Status.** Implemented (main, unreleased) — delivered as slices S1–S6
(`0f9d747`, `3a2fa5c`, `e84dcbe`, `854c40f`, `b060a47`, `46cd110`, `b744efc`).
Nine of the eleven §19 criteria are closed. Three items are **deferred to
RFC-054, not resolved here**:

1. **Criterion 7, end-to-end half.** The adapter-boundary mechanism is proven —
   `build_structure` failure cannot mutate source, and `PlainTextAdapter`
   supplies the fallback structure. The session actually choosing that fallback,
   and the UI offering "Show plain file text", require session wiring that no
   slice performed.
2. **Criterion 10.** `StructureErrorKind` production is complete for Markdown
   and PlainText, but the kind → friendly-message table was never built. Its
   home is fixed as `omriss-ui` (§11), matching the `CapabilityReason` split.
3. **The `&mut Document` coupling.** §7's `apply_validated_edit` and
   `structure_command` take `Document`, which is Markdown-specific: it owns a
   heading `Outline` and its public edit operations are section-shaped. This is
   correct for `MarkdownAdapter` and is why S4b's wrapping is clean, but a JSON
   or TOML adapter needs a format-neutral byte-range splice with history
   recording, which `Document` does not expose. RFC-054 must resolve this before
   it can implement a second format.

Nothing here wires the adapter into `EditorSession`; the shipped session still
calls `Document` operations directly. Markdown behavior is byte-for-byte
unchanged throughout: all 239 baseline tests passed unmodified across every
slice, and nine byte-identical comparison tests prove the command wrapping.
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
  ├─ Document          canonical text + revision + undo history (shipped)
  ├─ ActiveAdapter     closed enum, see §7.2
  ├─ DocumentStructure
  ├─ FocusState
  └─ SaveController
        │
        ▼
Format adapters
  ├─ MarkdownAdapter
  ├─ JsonAdapter
  ├─ TomlAdapter
  ├─ YamlExperimentalAdapter
  └─ PlainTextAdapter
```

`Document` already owns undo/redo, so the session does not hold a separate
history. Adapters appear below the session because they never own state; they
read `&str` and propose edits (§5.1).

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
//
// ByteRange is likewise the EXISTING shipped type (`omriss_core::ByteRange`).
// It is reused as-is. It must NOT be redefined as a bare `{ start, end }`
// struct: the shipped type carries `new()` (validating), `validate_in()`,
// `contains_range()`, `len()`, `is_empty()`, and `as_range()`. Redefining it
// would silently discard UTF-8 boundary validation.
//
// DocumentRevision is likewise the EXISTING shipped type
// (`omriss_core::DocumentRevision(pub u64)` with `next()`). Every revision
// field in this RFC means `DocumentRevision`, never a bare `u64`.
```

### 5.1 Types this RFC does NOT introduce

An earlier draft of this section declared `SourceText`, `LineEnding`, and its
own `ByteRange`. All three are withdrawn, because the codebase already owns
those responsibilities and duplicating them would create two sources of truth
for canonical text.

| Withdrawn | Use instead | Why |
|---|---|---|
| `ByteRange { start, end }` | `omriss_core::ByteRange` | shipped type validates UTF-8 boundaries; the flat struct does not |
| `revision: u64` | `omriss_core::DocumentRevision` | shipped newtype with `next()`; a bare `u64` invites mixing revisions with counts |
| `SourceText { text, line_ending, revision }` | `omriss_core::Document` (owner) + `&str` passed to adapters | §3.1 already assigns canonical-text ownership to the document session. A second owning struct would contradict it. |
| `LineEnding { Lf, Crlf, Mixed }` | `omriss_ui::NewlinePolicy` / `FileTextProfile` | already shipped and covered by tests |

**Adapters therefore receive `&str`, not an owning source type.** They read
source text and propose edits; they never own it.

**Line-ending handling.** `FileTextProfile` currently lives in `omriss-ui`. No
adapter in this RFC needs it: focused range replacement preserves bytes outside
the replaced range regardless of line-ending style. If RFC-054 or RFC-055 finds
that inserting *new* lines requires the profile, that RFC must request moving
`FileTextProfile` into `omriss-core` explicitly. It must not duplicate the type.

## 6. Document structure model

```rust
pub struct DocumentStructure {
    pub format: DocumentFormat,
    pub root_id: NodeId,
    pub nodes: Vec<StructureNode>,
    pub revision: DocumentRevision,
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

// `root_id` is the authoritative pointer to the root node. `kind` describes
// what a node *contains*, not its position, and must never be used to locate
// the root: `MarkdownAdapter`'s root is `DocumentRoot`, but `PlainTextAdapter`'s
// single node is `RawRegion` — correctly, since RFC-052 §5.2 forbids dressing
// opaque text as a document root. A consumer matching on `kind == DocumentRoot`
// silently finds nothing for PlainText. Settled during the S5 review.

// `depth` is the ancestor count from the root: the root itself is 0, its
// children are 1, and so on. It is NOT the format's own nesting notation —
// for Markdown it is tree depth, not heading level. The two diverge whenever
// RFC-007 reattaches a skipped heading level (`# A` followed directly by
// `### B` makes B depth 1, not 3). Every adapter must use this definition, so
// that Document Map indentation is consistent across formats. Settled during
// the S4a review.

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
    DepthLimit,             // not yet adopted — see note below
    ExternalChangeConflict, // session-overlay only — see note below
}
```

`DepthLimit` was added during the S2 review. The shipped Markdown capability
logic disables demote at H6 and promote at H1 using `NoSibling` and `NoParent`
respectively, because no accurate variant existed. `NoSibling` renders as
"Nothing to swap with here." even when a sibling plainly exists, which
undermines the reason typed reasons exist at all.

**Adopting `DepthLimit` is a user-visible change** — it alters the text on a
disabled menu item and needs a new catalog key in both `en` and `ja`. It must
therefore be its own task, and must **not** be folded into RFC-053 S3 or S4,
both of which are contract-bound to change no observable behavior. Until that
task runs, adapters continue to emit the shipped reasons, and this variant is
unconstructed.

`ExternalChangeConflict` is **not emitted by format adapters**. It is applied as
a session-level overlay after adapter capabilities are built, when external
file-modification state disables otherwise-valid operations. (A dedicated
`SessionCapabilityReason` may split this out later; for now the rule is that
adapters never produce it.)

## 7. Adapter trait

```rust
pub trait DocumentFormatAdapter {
    fn format(&self) -> DocumentFormat;

    fn build_structure(
        &self,
        source: &str,
        revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError>;

    fn focused_content(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
    ) -> Result<FocusedContent, FocusError>;

    fn validate_focused_edit(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
        draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError>;

    fn apply_validated_edit(
        &self,
        document: &mut Document,
        edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError>;

    fn structure_command(
        &self,
        document: &mut Document,
        structure: &DocumentStructure,
        command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError>;
}
```

### 7.0 `build_structure` takes the revision explicitly

**Added after the S5 review; this corrects a contradiction in the RFC.**

`DocumentStructure.revision` is the revision the structure was derived from, and
`structure_command` uses it as the `base_revision` for the shipped operation's
optimistic-concurrency check (§9.1). An adapter given only `&str` cannot know
that value: `MarkdownAdapter` parsed a throwaway `Document`, whose revision is
always `INITIAL`, so every structure claimed revision 0 regardless of the live
document's state. The consequence was that a command against any document edited
even once failed with `RevisionMismatch`, and rebuilding did not help — the
rebuild also reported 0. The boundary was effectively single-use per document.

The cause was this RFC, not its implementation: §7 originally took
`&SourceText`, which carried a revision. §5.1 withdrew `SourceText` — correctly,
to remove a second owner of canonical text — but that also removed the only
truthful source for this field, leaving §9.1 requiring a value §7 made
unobtainable.

The caller therefore passes it: the session already holds the live `Document`
and calls `build_structure(document.source(), document.revision())`. Adapters
still receive `&str` and own nothing (§5.1 is unchanged); the revision is data
the caller supplies, not state the adapter derives.

Staleness detection is preserved and is the point: a session holding a structure
built at revision N while the document has advanced to N+1 gets its command
rejected, exactly as RFC-002/RFC-008 intend.

### 7.1 Trait shape — decided, not delegated

An earlier draft called this trait "intentionally broad" and permitted the
implementer to split it into `FormatDetector`, `StructureBuilder`,
`FocusedContentProvider`, `FocusedEditValidator`, and `StructureCommandHandler`.
That is an architecture decision and does not belong to the implementer.

**Decision: one trait, as written above.** Five micro-traits would have to be
implemented together by every adapter anyway — no adapter is a useful
"structure builder" that cannot produce focused content — so splitting buys
indirection without buying substitutability. Revisit only if a real adapter
needs to implement a strict subset.

**`detect` is removed from the trait.** Detection is a whole-workspace concern
that must answer "which adapter?" *before* an adapter exists, so it cannot be an
instance method, and as a `Self: Sized` static it would block object safety for
no benefit. It moves to a free function in the detection module (§10).

**Mutation takes `&mut Document`, not a source buffer.** Per §3.1 the document
session owns canonical text, revision, undo history, and dirty state. An adapter
handed a raw mutable buffer could mutate text without recording an undo entry or
incrementing the revision — precisely the failure this architecture exists to
prevent. Adapters mutate only through `Document`'s existing replacement path,
which records history and revision as a unit.

### 7.2 Dispatch — static enum, not trait objects

**Decision: static dispatch over a closed enum.** Resolves §18 Q1.

```rust
pub enum ActiveAdapter {
    Markdown(MarkdownAdapter),
    Json(JsonAdapter),
    Toml(TomlAdapter),
    Yaml(YamlExperimentalAdapter),
    PlainText(PlainTextAdapter),
}
```

Rationale:

- the format set is closed and small, and RFC-052 fixes its membership;
- there is no plugin system and none is planned (§18 Q4), so runtime
  substitutability buys nothing;
- exhaustive `match` makes "did every adapter handle this?" a compile error
  rather than a review question;
- it sidesteps object safety entirely, so the trait stays free to use generics
  or associated types later without a breaking redesign.

`PlainText` gets a real adapter rather than a special case in the session: it
builds a single-node structure with all editing capabilities `Hidden`, per
RFC-052 §5.2's rule that omriss must not invent structure for formats it does
not understand.

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
    pub base_revision: DocumentRevision,
    pub replacement_range: ByteRange,
    pub replacement_text: String,
    pub description: EditDescription,
}
```

`base_revision` is copied from `DocumentStructure.revision`, which the caller
supplied to `build_structure` (§7.0). It must be the live document's revision at
the moment the structure was built — never a constant, and never re-derived
inside the adapter.

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

**The extension mapping is owned by RFC-052 §5.1 and is binding here.** It
includes the shipped `.mdown` and `.txt` → Markdown behavior; detection must not
regress those files to `PlainText`.

**Extension matching is case-insensitive.** RFC-052 §5.1 writes the mapping in
lowercase and is silent on case; `README.MD` is a Markdown file on every
supported platform, so extensions are compared after ASCII-lowercasing. This
was settled during the S1 review and is binding on later slices and on
RFC-054/055/056 — it must not be silently reversed. It creates no discrepancy
with the shipped `MD_EXTENSIONS` file-dialog filter, which configures a native
dialog rather than a comparison omriss performs.

Detection lives in `omriss_core::formats::detection` as free functions, not as a
trait method (§7.1). Resolves §18 Q5.

```rust
pub enum DetectionConfidence {
    No,
    Maybe,
    Likely,
    Certain,
}

pub fn detect_format(path: Option<&std::path::Path>, text: &str) -> DocumentFormat;
pub fn confidence_for(format: DocumentFormat, path: Option<&std::path::Path>, text: &str)
    -> DetectionConfidence;
```

`path` is optional because an unsaved buffer has no path; extension evidence is
then simply absent and content inspection decides.

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

**The mapping table lives in `omriss-ui`, not `omriss-core`.** Adapters return a
typed `StructureErrorKind`; `omriss-ui` maps it to a localized catalog string at
render time. This is the same split already established for `CapabilityReason`
(§6): `omriss-core` must never name an i18n catalog key, or the RFC-001
dependency direction inverts. `StructureError` itself stays minimal — it carries
the kind, and gains fields only when a real consumer needs them. Settled during
the S4a review.

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

`NodeId` is an opaque `u64` handle (RFC-006), never exposed in normal UI.

### 13.1 The binding requirement

Identity is a **testable behavioral requirement**, not a recommended strategy:

> After an edit that does not remove node N, the node the user was focused on
> must still be focused, and the Document Map highlight must still match the
> Writing Area.

Each adapter chooses how to derive `NodeId` values, subject to two rules:

1. **Deterministic.** Building the structure twice from identical source text
   must produce identical ids.
2. **Stable under unrelated edits.** Editing node A must not change the id of
   unrelated node B.

### 13.2 Per-format derivation

- **Markdown: keep the shipped scheme.** The outline builder's existing
  assignment stays as-is. Do not "improve" it during S4 — RFC-023/024/025 focus
  restoration tests depend on current behavior, and changing it converts a
  no-behavior-change slice into a regression risk.
- **JSON:** derive from a JSON-pointer-like path over object keys and array
  indexes.
- **TOML:** derive from the table/key path, plus an occurrence ordinal for
  repeated tables.
- **YAML:** RFC-056 must evaluate stable identity separately; array-index and
  anchor/alias identity are open problems there.

### 13.3 Required tests

Every adapter must ship both:

- rebuild determinism: same source → same ids;
- focus survival: edit an unrelated node, rebuild, and assert the previously
  focused id still resolves to the same logical item.

The second test is required because M10's manual QA could not reliably provoke
the stale-focus case by hand — it is recorded as "Not confirmed: conditional,
not triggered in normal QA" in the keyboard-only pass. Automation covers what
manual QA could not.

## 14. Reconciling the types M10 shipped early

M10 shipped part of this RFC's vocabulary into the **wrong crate**. RFC-048
listed "RFC-053 type core" as a dependency and the implementation went ahead
before RFC-053 existed, so `omriss-ui` currently owns types this RFC assigns to
`omriss-core`. Reconciling them is in scope for RFC-053 and is **not** new
feature work.

Current state, in `crates/ui/src/interface/document_map.rs`:

| Shipped in `omriss-ui` | Disposition under this RFC |
|---|---|
| `MapCapability` | becomes `Capability`, produced in `omriss-core` |
| `MapNodeCapabilities` | becomes `NodeCapabilities`, produced in `omriss-core` |
| `CapabilityReason` | moves to `omriss-core` unchanged in meaning |
| `DraftState` | moves to `omriss-core` (§9.3) |
| `DocumentMapNode` | **stays in `omriss-ui`** |

`DocumentMapNode` stays because it is a *view projection*, not a structure: it
nests children by value and carries `is_selected`, which is session state rather
than document structure. Under this RFC it is derived from `DocumentStructure`
instead of being built directly from the Markdown outline. It gains a `kind`
field sourced from `StructureNodeKind` — an additive change, which is what
RFC-049 §17 anticipated when it required the boundary to accept future node
kinds without redesign.

The English and Japanese catalogs already carry all eight `CapabilityReason`
variants, including `read_only_format`, `experimental_format`, and
`unsafe_preservation` — strings written for formats that do not exist yet. **The
catalog keys must not change during reconciliation.** `omriss-ui` keeps its sole
responsibility of mapping a typed, core-owned reason to localized text (RFC-043).

Binding constraint: this reconciliation must produce **zero user-visible
change**. The existing `omriss-ui` tests (`document_map_tests`, `i18n_tests`)
must pass with assertions unchanged in meaning; a changed assertion is evidence
of an accidental behavior change, not of progress.

## 15. Implementation slices

The trait in §7 must not be implemented in one pass. Required order, each slice
independently reviewable and shippable:

| Slice | Content | Proves |
|---|---|---|
| S1 | `DocumentFormat`, detection module, extension mapping per RFC-052 §5.1 | files are classified without changing any behavior |
| S2 | `DocumentStructure`, `StructureNode`, `StructureNodeKind`, `NodeCapabilities`, `Capability`, `CapabilityReason` in `omriss-core` | the vocabulary exists and is testable without Dioxus |
| S3 | Reconcile `omriss-ui` onto the S2 types (§14) | no user-visible change; existing tests pass |
| S4 | `MarkdownAdapter` wrapping shipped `Document` operations behind the trait | Markdown behavior and every golden test unchanged |
| S5 | `PlainTextAdapter` + parse-failure recovery to plain file text | safe failure works end to end |

S4 is the risk concentration point: it must reuse the shipped promote/demote
(RFC-023), move (RFC-024), and split/merge/delete (RFC-025) implementations
rather than reimplementing them. RFC-054 does not begin until S5 is accepted.

## 16. Preservation tests

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

## 17. Adapter-specific notes

### 17.1 Markdown

Markdown adapter remains the primary implementation. Existing section replacement and structural operations should be adapted into this architecture when low-risk. Concretely, the Markdown adapter wraps the shipped core — `Document` / `replace_section_body` (RFC-004/005), the heading tree (RFC-007), promote/demote (RFC-023), section move via `MoveTarget` (RFC-024), and split/merge/delete (RFC-025) — behind the adapter trait rather than rewriting it, so Markdown behavior and byte-preservation golden tests cannot regress.

### 17.2 JSON

JSON adapter should be implemented first. It should build a structural index with byte ranges for values and containers.

### 17.3 TOML

TOML adapter should preserve comments, key order, table layout, and inline forms. Use a lossless editing strategy rather than full normalization.

### 17.4 YAML

YAML adapter must start as a candidate/read-only feasibility adapter. Editable YAML is not approved by this RFC.

## 18. Security and safety considerations

- Do not execute file contents.
- Do not fetch schemas or network resources automatically.
- Do not process external includes automatically.
- Avoid stack overflow on deeply nested files by enforcing reasonable recursion limits.
- Avoid excessive memory use on very large files.
- Preserve user files by using existing atomic save and external modification checks.

## 19. Acceptance criteria

Each criterion names the evidence that closes it. "Can implement against this
boundary" is not evidence; it is an opinion held before the fact.

| # | Criterion | Evidence |
|---|---|---|
| 1 | `DocumentFormat` and detection exist in `omriss-core` and classify `.md` / `.markdown` / `.mdown` / `.txt` / `.json` / `.toml` / unknown per RFC-052 §5.1 | S1 unit tests, one case per extension |
| 2 | Structure vocabulary exists in `omriss-core` with no Dioxus dependency | S2 tests run under `cargo test -p omriss-core` |
| 3 | Capability production moved from `omriss-ui` to `omriss-core` | S3 diff; `omriss-ui` no longer defines `MapCapability` / `MapNodeCapabilities` |
| 4 | Reconciliation is user-visibly inert | S3: `document_map_tests` and `i18n_tests` pass with assertions unchanged in meaning; catalog keys unchanged |
| 5 | Markdown is served through the adapter without behavior change | S4: all 239 existing tests pass unchanged, including RFC-023/024/025 structural and golden byte-preservation suites |
| 6 | Node identity is deterministic and survives unrelated edits | §13.3 tests, per adapter |
| 7 | Parse failure preserves source text and offers plain file text | S5 test: malformed input → source intact, recovery path offered |
| 8 | `PlainText` produces no synthetic structure | S5 test: single node, editing capabilities `Hidden` |
| 9 | The app crate calls no format-specific rewrite logic | `crates/app` contains no parser or source-mutation call; grep-verifiable |
| 10 | Adapter errors map to the §11 friendly messages | error-mapping tests, one per `StructureErrorKind` |
| 11 | Gates green | `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/check-rfcs.sh` |

Criterion 5 is the blocking one. If any shipped Markdown test requires
modification to pass, the slice is wrong and must be reworked — the test is not
the thing to change.

## 20. Resolved questions

No question in this section is left for the implementer. All five are decided.

1. **Static enum dispatch, not trait objects.** See §7.2 for the decision and
   rationale.
2. **Adapters are modules inside `omriss-core`**
   (`omriss_core::formats::{markdown, json, toml, yaml}`). Separate
   `omriss-json` / `omriss-toml` crates are deferred until a measured need
   (dependency weight, feature-flag maintenance, or independent release/test).
3. **No format-specific settings.** Adapters are pure functions of source text:
   same input, same structure. Introducing per-format settings would make
   structure depend on hidden state, which breaks the §3.2 "derived structure is
   disposable" property and makes golden tests configuration-dependent. If a
   real need appears, it arrives as its own RFC with a migration story.
4. **No plugin-registered adapters.** Out of scope, and not merely on effort
   grounds: a plugin boundary would require committing to API stability this RFC
   explicitly declines (§5 permits private representation change), and loading
   third-party code to parse user documents contradicts §16, which forbids
   executing file content. The closed enum in §7.2 encodes this decision.
5. **`DetectionConfidence` lives in `omriss_core::formats::detection`**, beside
   the free detection functions (§10).

### 20.1 Genuinely open — deferred to the implementing RFCs

These are not blockers for RFC-053 and are recorded so they are not lost:

- whether `SessionCapabilityReason` should split `ExternalChangeConflict` out of
  `CapabilityReason` (§6) — revisit if a second session-level reason appears;
- whether `FocusedContent::StructuredGroup` needs a paging or truncation policy
  for very large groups — RFC-054 decides against real fixtures;
- whether `FileTextProfile` must move to `omriss-core` — RFC-054/055 decides if
  line-aware insertion needs it (§5.1).

## 21. Final decision summary

omriss will support future formats through document format adapters. The source text remains canonical, structures are derived, and all edits must be source-preserving. Full-file serialization is prohibited as the normal save path.
