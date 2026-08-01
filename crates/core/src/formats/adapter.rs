//! The document format adapter trait and its closed dispatch enum
//! (RFC-053 §7).
//!
//! Adapters own no state (RFC-053 §3.1): every method is a pure function of
//! its arguments, or takes `&mut Document` to apply a mutation through the
//! shipped replacement path so undo history and revision update as one unit
//! (RFC-053 §7.1). Dispatch is a closed enum, not trait objects (RFC-053
//! §7.2): the format set is fixed and small, and an exhaustive `match`
//! makes "did every adapter handle this?" a compile error.

use crate::formats::edit::{AppliedEdit, StructureCommand, ValidatedEdit};
use crate::formats::error::{
    ApplyEditError, EditValidationError, FocusError, StructureCommandError, StructureError,
};
use crate::formats::focused_content::FocusedContent;
use crate::formats::markdown::MarkdownAdapter;
use crate::formats::plain_text::PlainTextAdapter;
use crate::formats::structure::DocumentStructure;
use crate::formats::unsupported::{JsonAdapter, TomlAdapter, YamlExperimentalAdapter};
use crate::{Document, DocumentFormat, DocumentRevision, NodeId};

/// The contract every document format implements (RFC-053 §7).
pub trait DocumentFormatAdapter {
    /// The format this adapter serves.
    fn format(&self) -> DocumentFormat;

    /// Derives a [`DocumentStructure`] from `source`, stamped with
    /// `revision` — the revision `source` was read at (RFC-053 §7.0).
    /// Adapters own no state and cannot know this on their own; the caller
    /// (typically `document.revision()` alongside `document.source()`)
    /// supplies it. `structure_command` uses this value as the base
    /// revision for staleness detection (§9.1), so it must be the true
    /// revision `source` reflects, not a value the adapter invents.
    fn build_structure(
        &self,
        source: &str,
        revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError>;

    /// The right-hand editor's view of `node_id`.
    fn focused_content(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
    ) -> Result<FocusedContent, FocusError>;

    /// Validates a focused-content draft without applying it.
    fn validate_focused_edit(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
        draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError>;

    /// Applies a previously validated focused-content edit.
    fn apply_validated_edit(
        &self,
        document: &mut Document,
        edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError>;

    /// Applies a structural command (move, rename, add, delete, join).
    fn structure_command(
        &self,
        document: &mut Document,
        structure: &DocumentStructure,
        command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError>;
}

/// The active format adapter for the current document (RFC-053 §7.2).
///
/// `Json`/`Toml`/`Yaml` are stub variants per the non-change-scope: "the
/// enum variants may exist; the adapters must return unsupported until
/// RFC-054+." `PlainText` (RFC-053 S5) is now fully populated — the last
/// deferral from S4b resolved.
pub enum ActiveAdapter {
    Markdown(MarkdownAdapter),
    Json(JsonAdapter),
    Toml(TomlAdapter),
    Yaml(YamlExperimentalAdapter),
    PlainText(PlainTextAdapter),
}
