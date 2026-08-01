//! JSON format adapter (RFC-054 J2).
//!
//! `JsonAdapter::build_structure` is real as of this slice: `scanner`
//! parses strict RFC 8259 JSON, retaining a byte range for every value
//! (including duplicate object keys, which are never merged — RFC-054
//! §15.2), and `projection` turns the result into a `DocumentStructure`
//! per RFC-054 §5 (structure model) and §6 (node identity).
//!
//! Every edit-shaped method still refuses
//! (`StructureErrorKind::UnsupportedFeature`): this slice is explicitly
//! read-only (RFC-054 task-breakdown, J2 — "Not in this slice: any session
//! wiring. Nothing calls `JsonAdapter` yet"). Scalar editing is J5;
//! container raw editing is J6.

mod projection;
mod scanner;
mod string_literal;

use crate::formats::edit::{AppliedEdit, StructureCommand, ValidatedEdit};
use crate::formats::error::{
    ApplyEditError, EditValidationError, FocusError, StructureCommandError, StructureError,
    StructureErrorKind,
};
use crate::formats::focused_content::FocusedContent;
use crate::formats::structure::DocumentStructure;
use crate::{Document, DocumentFormat, DocumentFormatAdapter, DocumentRevision, NodeId};

/// The JSON format adapter (RFC-054).
///
/// Owns no state, per RFC-053 §3.1.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonAdapter;

impl DocumentFormatAdapter for JsonAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Json
    }

    /// Strict RFC 8259 only (RFC-052 §14.1, RFC-054 §0.5): comments,
    /// trailing commas, unquoted keys, and every other JSONC-style
    /// tolerance are rejected with `StructureErrorKind::InvalidSyntax`,
    /// never silently accepted.
    fn build_structure(
        &self,
        source: &str,
        revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError> {
        let root = scanner::parse(source).map_err(|_| StructureErrorKind::InvalidSyntax)?;
        Ok(projection::build(&root, revision))
    }

    /// Not this slice's scope (J5/J6 give this real content); refuses
    /// rather than guessing (RFC-053 §3.3).
    fn focused_content(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        Err(StructureErrorKind::UnsupportedFeature.into())
    }

    fn validate_focused_edit(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
        _draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError> {
        Err(StructureErrorKind::UnsupportedFeature.into())
    }

    fn apply_validated_edit(
        &self,
        _document: &mut Document,
        _edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError> {
        Err(StructureErrorKind::UnsupportedFeature.into())
    }

    /// RFC-054 §13 Phase 4 (add/delete/rename/move) is out of scope for
    /// this entire handoff, not just this slice; refuses unconditionally.
    fn structure_command(
        &self,
        _document: &mut Document,
        _structure: &DocumentStructure,
        _command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        Err(StructureErrorKind::UnsupportedFeature.into())
    }
}
