//! Plain-text fallback adapter (RFC-053 S5, RFC-052 §5.2).
//!
//! `PlainTextAdapter` is the format omriss falls back to when it cannot (or
//! must not) invent structure: an unknown extension, or — once RFC-054+
//! wires the session (not this slice) — a format whose real adapter failed
//! to build structure. Per RFC-052 §5.2: "omriss must not invent a
//! hierarchy for a format it does not understand." Accordingly this
//! adapter:
//!
//! - always succeeds at `build_structure`, for any `&str` — there is
//!   nothing to parse, so there is nothing that can fail;
//! - produces exactly **one** node, no synthetic children;
//! - hides every editing capability except `can_show_plain_text`;
//! - refuses every edit-shaped method rather than guessing (RFC-053 §3.3).
//!
//! This slice proves the mechanism at the adapter boundary only — nothing
//! in `crates/ui`/`crates/app` calls this yet. Wiring the session to fall
//! back to `PlainTextAdapter` when a real adapter's `build_structure` fails
//! is RFC-054+ work.

use crate::formats::edit::{AppliedEdit, StructureCommand, ValidatedEdit};
use crate::formats::error::{
    ApplyEditError, EditValidationError, FocusError, StructureCommandError, StructureError,
    StructureErrorKind,
};
use crate::formats::focused_content::FocusedContent;
use crate::formats::structure::{
    DocumentStructure, NodeCapabilities, StructureNode, StructureNodeKind,
};
use crate::{ByteRange, Document, DocumentFormat, DocumentFormatAdapter, DocumentRevision, NodeId};

/// The plain-text fallback adapter (RFC-052 §5.2).
///
/// Owns no state, per RFC-053 §3.1.
#[derive(Debug, Default, Clone, Copy)]
pub struct PlainTextAdapter;

impl DocumentFormatAdapter for PlainTextAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::PlainText
    }

    /// Always succeeds: the whole source becomes one `RawRegion` node with
    /// every editing capability hidden. There is no synthetic structure to
    /// build, so there is nothing that can fail here.
    ///
    /// `revision` is stamped onto the result as-is (RFC-053 §7.0) — the
    /// caller's responsibility to supply the revision `source` was read at.
    fn build_structure(
        &self,
        source: &str,
        revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError> {
        let id = root_node_id();
        Ok(DocumentStructure {
            format: DocumentFormat::PlainText,
            root_id: id,
            nodes: vec![StructureNode {
                id,
                parent_id: None,
                title: String::new(),
                kind: StructureNodeKind::RawRegion,
                depth: 0,
                source_range: Some(ByteRange {
                    start: 0,
                    end: source.len(),
                }),
                editable_range: None,
                children: Vec::new(),
                capabilities: NodeCapabilities::hidden(),
            }],
            revision,
        })
    }

    /// Reports the node as viewable-as-plain-text-only rather than
    /// guessing at content — this is exactly the case
    /// `FocusedContent::Unsupported` exists for.
    fn focused_content(
        &self,
        _source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        if node_id != structure.root_id {
            return Err(StructureErrorKind::UnsafeRange.into());
        }
        Ok(FocusedContent::Unsupported {
            title: String::new(),
            reason: StructureErrorKind::UnsupportedFeature,
            raw_text_available: true,
        })
    }

    /// No content is editable; refuses rather than guessing (RFC-053 §3.3).
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

    /// No structural action is available; every capability is `Hidden`.
    fn structure_command(
        &self,
        _document: &mut Document,
        _structure: &DocumentStructure,
        _command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        Err(StructureErrorKind::UnsupportedFeature.into())
    }
}

/// A deterministic id for the single node, reusing the same
/// root-of-nothing derivation the Markdown outline builder uses for its
/// own synthetic root (`NodeId::from_ordinal_path(&[])`) — both represent
/// "the whole document as one thing," so sharing the derivation is
/// conceptually consistent, not merely convenient.
fn root_node_id() -> NodeId {
    NodeId::from_ordinal_path(&[])
}
