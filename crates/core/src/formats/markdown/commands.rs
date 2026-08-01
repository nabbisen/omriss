//! `StructureCommand` dispatch helpers — the actual RFC-023/024/025
//! wrapping. Every function here either calls exactly one shipped
//! `Document` operation, or (for `AddInside`/`AddAfter`, which have no
//! single shipped primitive) computes the same offset/level arithmetic the
//! shipped `omriss_ui::session::structural` already uses before calling
//! `Document::split_section` (RFC-025).

use crate::formats::edit::{NewNodeSpec, StructureCommand};
use crate::formats::error::{StructureCommandError, StructureErrorKind};
use crate::formats::structure::sibling_neighbors;
use crate::{
    Document, DocumentRevision, EditResult, HeadingLevel, MoveTarget, NodeId, Outline, SectionNode,
};

pub(super) fn command_target(command: &StructureCommand) -> NodeId {
    match command {
        StructureCommand::AddInside { target, .. }
        | StructureCommand::AddAfter { target, .. }
        | StructureCommand::Rename { target, .. }
        | StructureCommand::Move { target, .. }
        | StructureCommand::JoinWithPrevious { target }
        | StructureCommand::Delete { target } => *target,
    }
}

/// `Move { Up | Down }`: find the neighbor sibling and delegate to the
/// shipped `Document::move_section` (RFC-024), mirroring
/// `omriss_ui::session::structural::move_focused_up`/`move_focused_down`.
pub(super) fn move_to_sibling(
    document: &mut Document,
    target: NodeId,
    base_rev: DocumentRevision,
    up: bool,
) -> Result<EditResult, StructureCommandError> {
    let (prev, next) = sibling_neighbors(document.outline(), target);
    let move_target = if up {
        prev.map(MoveTarget::Before)
    } else {
        next.map(MoveTarget::After)
    }
    .ok_or(StructureCommandError {
        kind: StructureErrorKind::UnsafeRange,
    })?;
    document
        .move_section(target, move_target, base_rev)
        .map_err(Into::into)
}

/// `AddInside`: appends a new child at the bottom of `target`'s children,
/// one heading level deeper than `target` — or `H1` if `target` is the
/// document root, making this the same operation as the shipped
/// `add_top_level_section`. Built from `Document::split_section` (RFC-025)
/// plus the offset arithmetic shipped in
/// `omriss_ui::session::structural::append_child_to_focused`/`add_top_level_section`.
pub(super) fn add_inside(
    document: &mut Document,
    target: NodeId,
    spec: NewNodeSpec,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructureCommandError> {
    let node = target_section(document.outline(), target)?;
    let new_level = match node.level {
        Some(level) => child_level_after(level),
        None => HeadingLevel::H1,
    };
    let offset = node.full_range.end - node.body_range.start;
    document
        .split_section(target, offset, &spec.title, new_level, base_rev)
        .map_err(Into::into)
}

/// `AddAfter`: appends a new sibling immediately after `target` and its
/// subtree, at `target`'s own heading level. Built from
/// `Document::split_section` (RFC-025) plus the offset arithmetic shipped in
/// `omriss_ui::session::structural::add_after_focused`. Has no shipped
/// equivalent for the document root (there is no "after" position for the
/// single root), so that case is rejected rather than guessed.
pub(super) fn add_after(
    document: &mut Document,
    target: NodeId,
    spec: NewNodeSpec,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructureCommandError> {
    let node = target_section(document.outline(), target)?;
    let level = node.level.ok_or(StructureCommandError {
        kind: StructureErrorKind::UnsupportedFeature,
    })?;
    let offset = node.full_range.end - node.body_range.start;
    document
        .split_section(target, offset, &spec.title, level, base_rev)
        .map_err(Into::into)
}

fn target_section(outline: &Outline, id: NodeId) -> Result<&SectionNode, StructureCommandError> {
    outline.node(id).ok_or(StructureCommandError {
        kind: StructureErrorKind::UnsafeRange,
    })
}

/// The child heading level one level deeper than `level`, saturating at H6
/// (mirrors the shipped `shell::actions::handle_section_title_choice`
/// `AddInside` level computation).
fn child_level_after(level: HeadingLevel) -> HeadingLevel {
    use HeadingLevel::*;
    match level {
        H1 => H2,
        H2 => H3,
        H3 => H4,
        H4 => H5,
        H5 | H6 => H6,
    }
}
