//! Markdown format adapter (RFC-053 S4).
//!
//! `MarkdownAdapter` implements [`DocumentFormatAdapter`] by **wrapping**
//! the shipped, golden-tested Markdown operations — it never reimplements
//! byte-level editing itself. `build_structure` projects the shipped
//! `Outline` (RFC-006/007) into a [`DocumentStructure`] (RFC-053 §6),
//! reusing every `NodeId` the outline already assigned — never re-deriving,
//! hashing, or "improving" them (RFC-053 §13.2). Structural mutation goes
//! through `Document`'s existing replacement path (RFC-053 §7.1), so undo
//! history and revision update as one unit; this adapter never rewrites
//! text itself.
//!
//! `StructureCommand` -> shipped operation mapping (RFC-053 §9.2), each
//! called at exactly one site in `markdown::commands`:
//!
//! ```text
//! Move { InsidePrevious } -> Document::demote_section        (RFC-023)
//! Move { OutOneLevel }    -> Document::promote_section        (RFC-023)
//! Move { Up | Down }      -> Document::move_section           (RFC-024)
//! JoinWithPrevious        -> Document::merge_with_prev_sibling (RFC-025)
//! AddInside / AddAfter    -> Document::split_section           (RFC-025)
//! Rename                  -> Document::rename_section          (RFC-049)
//! Delete                  -> Document::delete_section          (RFC-025)
//! ```
//!
//! `AddInside`/`AddAfter` have no single shipped primitive; they are built
//! from `split_section` plus offset/level arithmetic exactly as the shipped
//! `omriss_ui::session::structural` and `omriss_app::shell::actions`
//! already compose it (see `commands::child_level_after`).

mod commands;
mod error_mapping;
mod projection;

use commands::{add_after, add_inside, command_target, move_to_sibling};
use error_mapping::map_edit_error;
use projection::{find_node, structure_node};

use crate::formats::edit::{AppliedEdit, EditDescription, StructureCommand, ValidatedEdit};
use crate::formats::error::{
    ApplyEditError, EditValidationError, FocusError, StructureCommandError, StructureError,
    StructureErrorKind,
};
use crate::formats::focused_content::FocusedContent;
use crate::formats::structure::DocumentStructure;
use crate::{
    Document, DocumentFormat, DocumentFormatAdapter, MoveDirection, NodeId, ReplaceSectionBody,
};

/// The Markdown format adapter (RFC-053 §17.1).
///
/// Owns no state — every method is a pure function of its arguments, or
/// mutates only through the `Document` it is given, per RFC-053 §3.1/§7.1.
#[derive(Debug, Default, Clone, Copy)]
pub struct MarkdownAdapter;

impl DocumentFormatAdapter for MarkdownAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Markdown
    }

    /// Projects `source` into a [`DocumentStructure`], preserving every
    /// `NodeId` the shipped outline builder assigns (RFC-053 §13.2).
    ///
    /// Markdown's own parser does not reject malformed input — headings are
    /// detected permissively by `pulldown-cmark` — so the only realistic
    /// failure here is an internal outline invariant violation, never
    /// "invalid" user Markdown.
    fn build_structure(&self, source: &str) -> Result<DocumentStructure, StructureError> {
        let document = Document::parse(source.to_string())
            .map_err(|_| StructureErrorKind::InternalInvariantFailed)?;
        let outline = document.outline();

        let nodes = outline
            .iter()
            .map(|section| structure_node(outline, section))
            .collect();

        Ok(DocumentStructure {
            format: DocumentFormat::Markdown,
            root_id: outline.root_id(),
            nodes,
            revision: document.revision(),
        })
    }

    fn focused_content(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        let node = find_node(structure, node_id)?;
        let range = node
            .editable_range
            .ok_or(StructureErrorKind::InternalInvariantFailed)?;
        let body = source
            .get(range.as_range())
            .ok_or(StructureErrorKind::InternalInvariantFailed)?
            .to_string();
        Ok(FocusedContent::MarkdownSection {
            title: node.title.clone(),
            body,
            preview_available: true,
        })
    }

    /// Any text is a valid Markdown section body (RFC-053 §9.3: "Markdown
    /// body drafts are at worst `ValidUncommitted`"), so this never
    /// actually rejects a draft — it only resolves the replacement range.
    fn validate_focused_edit(
        &self,
        _source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
        draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError> {
        let node = find_node(structure, node_id)?;
        let range = node
            .editable_range
            .ok_or(StructureErrorKind::InternalInvariantFailed)?;
        Ok(ValidatedEdit {
            node_id,
            base_revision: structure.revision,
            replacement_range: range,
            replacement_text: draft.to_string(),
            description: EditDescription::FocusedContentReplacement,
        })
    }

    /// Wraps the shipped `Document::replace_section_body` (RFC-004/005).
    fn apply_validated_edit(
        &self,
        document: &mut Document,
        edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError> {
        let result = document
            .replace_section_body(ReplaceSectionBody {
                node_id: edit.node_id,
                base_revision: edit.base_revision,
                new_body: edit.replacement_text,
            })
            .map_err(map_edit_error)?;
        Ok(AppliedEdit { result })
    }

    fn structure_command(
        &self,
        document: &mut Document,
        structure: &DocumentStructure,
        command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        let target = command_target(&command);
        if !structure.nodes.iter().any(|n| n.id == target) {
            return Err(StructureErrorKind::UnsafeRange.into());
        }
        let base_rev = structure.revision;

        let result = match command {
            StructureCommand::Move {
                target,
                direction: MoveDirection::InsidePrevious,
            } => document
                .demote_section(target, base_rev)
                .map_err(Into::into),
            StructureCommand::Move {
                target,
                direction: MoveDirection::OutOneLevel,
            } => document
                .promote_section(target, base_rev)
                .map_err(Into::into),
            StructureCommand::Move {
                target,
                direction: MoveDirection::Up,
            } => move_to_sibling(document, target, base_rev, true),
            StructureCommand::Move {
                target,
                direction: MoveDirection::Down,
            } => move_to_sibling(document, target, base_rev, false),
            StructureCommand::JoinWithPrevious { target } => document
                .merge_with_prev_sibling(target, base_rev)
                .map_err(Into::into),
            StructureCommand::Rename { target, new_name } => document
                .rename_section(target, &new_name, base_rev)
                .map_err(Into::into),
            StructureCommand::Delete { target } => document
                .delete_section(target, base_rev)
                .map_err(Into::into),
            StructureCommand::AddInside { target, spec } => {
                add_inside(document, target, spec, base_rev)
            }
            StructureCommand::AddAfter { target, spec } => {
                add_after(document, target, spec, base_rev)
            }
        }?;

        Ok(AppliedEdit { result })
    }
}
