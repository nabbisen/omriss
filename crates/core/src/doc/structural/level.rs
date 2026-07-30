//! Promote / Demote (RFC-023).

use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::{HeadingLevel, NodeId};
use crate::range::ByteRange;
use crate::{Document, Outline};

use super::error::StructuralEditError;
use super::preflight::{check_revision, is_descendant};

/// Builds a new ATX marker string for `current_level ± delta`.
/// Returns `None` if the result would be outside H1..H6.
fn adjusted_marker(current_level: HeadingLevel, delta: i8) -> Option<String> {
    let new_depth = current_level.as_u8() as i8 + delta;
    if !(1..=6).contains(&new_depth) {
        return None;
    }
    Some("#".repeat(new_depth as usize))
}

/// Core of promote/demote: shifts the selected section subtree one heading
/// level while preserving its internal parent/child relationships.
fn change_heading_level(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
    delta: i8, // -1 = promote, +1 = demote
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    let level = node.level.ok_or(StructuralEditError::StaleNode(id))?;
    adjusted_marker(level, delta).ok_or(StructuralEditError::InvalidLevel)?;

    let source = doc.source();
    let outline = doc.outline();
    let subtree_ids: Vec<NodeId> = outline
        .iter()
        .filter(|candidate| is_descendant(outline, id, candidate.id))
        .map(|candidate| candidate.id)
        .collect();
    let mut marker_edits = Vec::with_capacity(subtree_ids.len());
    for node_id in subtree_ids {
        let n = outline
            .node(node_id)
            .ok_or(StructuralEditError::StaleNode(node_id))?;
        let level = n.level.ok_or(StructuralEditError::StaleNode(node_id))?;
        let heading_start = n.heading_range.start;
        if source.as_bytes().get(heading_start) != Some(&b'#') {
            return Err(StructuralEditError::UnsupportedHeadingStyle);
        }
        let new_marker = adjusted_marker(level, delta).ok_or(StructuralEditError::InvalidLevel)?;
        let old_marker_len = level.as_u8() as usize;
        marker_edits.push((heading_start, old_marker_len, new_marker));
    }

    let mut shifted_subtree = source[node.full_range.as_range()].to_string();
    for (heading_start, old_marker_len, new_marker) in marker_edits.iter().rev() {
        let local_start = heading_start - node.full_range.start;
        shifted_subtree.replace_range(local_start..local_start + old_marker_len, new_marker);
    }

    let new_source = if delta < 0 && has_following_sibling(outline, id)? {
        let parent_id = node.parent_id.ok_or(StructuralEditError::InvalidLevel)?;
        let parent = outline
            .node(parent_id)
            .ok_or(StructuralEditError::StaleNode(parent_id))?;
        let range = node.full_range;
        let mut s = String::with_capacity(source.len() - range.len() + shifted_subtree.len());
        s.push_str(&source[..range.start]);
        s.push_str(&source[range.end..parent.full_range.end]);
        s.push_str(&shifted_subtree);
        s.push_str(&source[parent.full_range.end..]);
        s
    } else {
        let range = node.full_range;
        let mut s = String::with_capacity(source.len() - range.len() + shifted_subtree.len());
        s.push_str(&source[..range.start]);
        s.push_str(&shifted_subtree);
        s.push_str(&source[range.end..]);
        s
    };

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

fn has_following_sibling(outline: &Outline, id: NodeId) -> Result<bool, StructuralEditError> {
    let node = outline.node(id).ok_or(StructuralEditError::StaleNode(id))?;
    let Some(parent_id) = node.parent_id else {
        return Ok(false);
    };
    let parent = outline
        .node(parent_id)
        .ok_or(StructuralEditError::StaleNode(parent_id))?;
    let Some(position) = parent.children.iter().position(|child| *child == id) else {
        return Err(StructuralEditError::StaleNode(id));
    };
    Ok(position + 1 < parent.children.len())
}

pub(crate) fn promote_section(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    change_heading_level(doc, id, base_rev, -1)
}

pub(crate) fn demote_section(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    change_heading_level(doc, id, base_rev, 1)
}
