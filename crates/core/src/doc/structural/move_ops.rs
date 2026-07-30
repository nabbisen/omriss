//! Move section (RFC-024).

use crate::Document;
use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::NodeId;
use crate::range::ByteRange;

use super::error::StructuralEditError;
use super::preflight::{check_revision, is_descendant};

/// Where to place a moved section relative to another node (RFC-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    /// Place the moved section immediately before `node` (same parent level).
    Before(NodeId),
    /// Place the moved section immediately after `node` (same parent level).
    After(NodeId),
    /// Place as the first child of `node`.
    AsFirstChildOf(NodeId),
    /// Place as the last child of `node`.
    AsLastChildOf(NodeId),
}

/// Moves the full section subtree of `id` to `target`.
/// The exact source bytes are preserved; no blank-line normalization occurs.
pub(crate) fn move_section(
    doc: &mut Document,
    id: NodeId,
    target: MoveTarget,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;

    let src_node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if src_node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }
    let src_range = src_node.full_range;

    // Resolve insertion point and validate.
    let target_id = match target {
        MoveTarget::Before(t)
        | MoveTarget::After(t)
        | MoveTarget::AsFirstChildOf(t)
        | MoveTarget::AsLastChildOf(t) => t,
    };
    if target_id == id {
        return Err(StructuralEditError::CannotMoveSelf);
    }
    let target_node = doc
        .outline()
        .node(target_id)
        .ok_or(StructuralEditError::StaleTarget(target_id))?;

    // Prevent moving into a descendant.
    if is_descendant(doc.outline(), id, target_id) {
        return Err(StructuralEditError::CannotMoveIntoDescendant);
    }

    let insert_pos = match target {
        MoveTarget::Before(_) => target_node.heading_range.start,
        MoveTarget::After(_) => target_node.full_range.end,
        MoveTarget::AsFirstChildOf(_) => target_node.heading_range.end,
        MoveTarget::AsLastChildOf(_) => target_node.full_range.end,
    };

    // Build the new source string without any offset arithmetic.
    let source = doc.source();
    let moved = source[src_range.as_range()].to_string();
    let new_source = if insert_pos <= src_range.start {
        // Inserting before the source range.
        let mut s = String::with_capacity(source.len());
        s.push_str(&source[..insert_pos]);
        s.push_str(&moved);
        s.push_str(&source[insert_pos..src_range.start]);
        s.push_str(&source[src_range.end..]);
        s
    } else if insert_pos >= src_range.end {
        // Inserting after the source range.
        let mut s = String::with_capacity(source.len());
        s.push_str(&source[..src_range.start]);
        s.push_str(&source[src_range.end..insert_pos]);
        s.push_str(&moved);
        s.push_str(&source[insert_pos..]);
        s
    } else {
        // Target insertion point is inside the source range — shouldn't reach
        // here after descendant check, but guard it.
        return Err(StructuralEditError::CannotMoveIntoDescendant);
    };

    // Apply as a full-source replacement so the history captures the exact
    // before/after and undo is byte-exact (RFC-044).
    let full_range = ByteRange {
        start: 0,
        end: source.len(),
    };
    let old_source = source.to_string();
    let result = doc.apply_replacement(full_range, &new_source)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text: old_source,
        new_range: result.new_range,
        new_text: new_source,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}
