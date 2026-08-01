//! Stub adapters for formats not yet implemented.
//!
//! Per the RFC-053 non-change-scope: "the enum variants may exist; the
//! adapters must return unsupported until RFC-054+." No parsing logic of
//! any kind — every method returns
//! [`StructureErrorKind::UnsupportedFeature`].

use crate::formats::edit::{AppliedEdit, StructureCommand, ValidatedEdit};
use crate::formats::error::{
    ApplyEditError, EditValidationError, FocusError, StructureCommandError, StructureError,
    StructureErrorKind,
};
use crate::formats::focused_content::FocusedContent;
use crate::formats::structure::DocumentStructure;
use crate::{Document, DocumentFormat, DocumentFormatAdapter, DocumentRevision, NodeId};

/// JSON support is not implemented; see RFC-054.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonAdapter;

/// TOML support is not implemented; see RFC-055.
#[derive(Debug, Default, Clone, Copy)]
pub struct TomlAdapter;

/// YAML support is not implemented; see RFC-056 (feasibility spike).
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlExperimentalAdapter;

impl DocumentFormatAdapter for JsonAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Json
    }

    fn build_structure(
        &self,
        _source: &str,
        _revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError> {
        Err(unsupported())
    }

    fn focused_content(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        Err(unsupported())
    }

    fn validate_focused_edit(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
        _draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError> {
        Err(unsupported())
    }

    fn apply_validated_edit(
        &self,
        _document: &mut Document,
        _edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError> {
        Err(unsupported())
    }

    fn structure_command(
        &self,
        _document: &mut Document,
        _structure: &DocumentStructure,
        _command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        Err(unsupported())
    }
}

impl DocumentFormatAdapter for TomlAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Toml
    }

    fn build_structure(
        &self,
        _source: &str,
        _revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError> {
        Err(unsupported())
    }

    fn focused_content(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        Err(unsupported())
    }

    fn validate_focused_edit(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
        _draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError> {
        Err(unsupported())
    }

    fn apply_validated_edit(
        &self,
        _document: &mut Document,
        _edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError> {
        Err(unsupported())
    }

    fn structure_command(
        &self,
        _document: &mut Document,
        _structure: &DocumentStructure,
        _command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        Err(unsupported())
    }
}

impl DocumentFormatAdapter for YamlExperimentalAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::YamlExperimental
    }

    fn build_structure(
        &self,
        _source: &str,
        _revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError> {
        Err(unsupported())
    }

    fn focused_content(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        Err(unsupported())
    }

    fn validate_focused_edit(
        &self,
        _source: &str,
        _structure: &DocumentStructure,
        _node_id: NodeId,
        _draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError> {
        Err(unsupported())
    }

    fn apply_validated_edit(
        &self,
        _document: &mut Document,
        _edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError> {
        Err(unsupported())
    }

    fn structure_command(
        &self,
        _document: &mut Document,
        _structure: &DocumentStructure,
        _command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        Err(unsupported())
    }
}

/// Builds an "unsupported feature" error of whichever error type the call
/// site needs, via the `From<StructureErrorKind>` impls in
/// `crate::formats::error`.
fn unsupported<E: From<StructureErrorKind>>() -> E {
    StructureErrorKind::UnsupportedFeature.into()
}
