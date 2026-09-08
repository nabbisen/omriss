//! Move section (RFC-024).

use crate::Document;
use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::NodeId;
use crate::range::ByteRange;

use super::error::StructuralEditError;
use super::preflight::{check_revision, document_newline, is_descendant, joining_separator};

/// Where to place a moved section relative to another node (RFC-024).
///
/// `AsFirstChildOf`/`AsLastChildOf` were withdrawn in RFC-065 §4/B7
/// (breaking change, owner-decided 2026-09-01, recorded in RFC-024's
/// Status field): neither ever adjusted heading levels, so neither ever
/// actually created a child — `AsLastChildOf` was a duplicate of `After`,
/// and `AsFirstChildOf` split the target's own body instead of nesting
/// under it. The GUI never called either. Real child-placement semantics
/// are re-proposed by RFC-058, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    /// Place the moved section immediately before `node` (same parent level).
    Before(NodeId),
    /// Place the moved section immediately after `node` (same parent level).
    After(NodeId),
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
        MoveTarget::Before(t) | MoveTarget::After(t) => t,
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
    };

    // Build the new source string without any offset arithmetic.
    let source = doc.source();
    let newline = document_newline(source);
    let moved = source[src_range.as_range()].to_string();
    // Each branch concatenates four pieces and therefore has three seams
    // (RFC-065 §4.1): every one of them used to be protected by a real
    // document boundary (the moved node's own heading newline, or the gap
    // it used to occupy) that the move removes, so each is checked with
    // `joining_separator` rather than assumed safe.
    let new_source = if insert_pos <= src_range.start {
        // Inserting before the source range: A | moved | C | D.
        let a = &source[..insert_pos];
        let c = &source[insert_pos..src_range.start];
        let d = &source[src_range.end..];
        let sep_ab = joining_separator(a, &moved, newline);
        let sep_bc = joining_separator(&moved, c, newline);
        let sep_cd = joining_separator(c, d, newline);
        let mut s =
            String::with_capacity(source.len() + sep_ab.len() + sep_bc.len() + sep_cd.len());
        s.push_str(a);
        s.push_str(sep_ab);
        s.push_str(&moved);
        s.push_str(sep_bc);
        s.push_str(c);
        s.push_str(sep_cd);
        s.push_str(d);
        s
    } else if insert_pos >= src_range.end {
        // Inserting after the source range: A | B | moved | D.
        let a = &source[..src_range.start];
        let b = &source[src_range.end..insert_pos];
        let d = &source[insert_pos..];
        let sep_ab = joining_separator(a, b, newline);
        let sep_bc = joining_separator(b, &moved, newline);
        let sep_cd = joining_separator(&moved, d, newline);
        let mut s =
            String::with_capacity(source.len() + sep_ab.len() + sep_bc.len() + sep_cd.len());
        s.push_str(a);
        s.push_str(sep_ab);
        s.push_str(b);
        s.push_str(sep_bc);
        s.push_str(&moved);
        s.push_str(sep_cd);
        s.push_str(d);
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
