//! Delete, split, and merge section operations (RFC-025).

use crate::Document;
use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::{HeadingLevel, NodeId};
use crate::range::ByteRange;

use super::error::StructuralEditError;
use super::preflight::check_revision;

pub(crate) fn delete_section(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }
    let range = node.full_range;
    let old_text = doc.source()[range.as_range()].to_string();
    let result = doc.apply_replacement(range, "")?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text,
        new_range: result.new_range,
        new_text: String::new(),
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

/// Inserts a new heading of `new_level` with `new_title` at `offset_in_body`
/// bytes into the focused section's body. The text before the offset stays as
/// the current section's body; the text after becomes the new section's body.
pub(crate) fn split_section(
    doc: &mut Document,
    id: NodeId,
    offset_in_body: usize,
    new_title: &str,
    new_level: HeadingLevel,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    let body = node.body_range;
    let full = node.full_range;

    // offset_in_body is relative to body_range.start.
    // Valid range: 0..=body.len() places the heading within the section body.
    // full.end - body.start is also valid: it inserts the heading after all
    // existing children (at full_range.end), which is the correct position
    // for appending a new child at the bottom.
    let max_offset = full.end - body.start;
    if offset_in_body > max_offset {
        return Err(StructuralEditError::InvalidSplitOffset);
    }
    let insert_pos = body.start + offset_in_body;
    if !doc.source().is_char_boundary(insert_pos) {
        return Err(StructuralEditError::InvalidSplitOffset);
    }

    let marker = "#".repeat(new_level.as_u8() as usize);
    let heading_text = format!("\n{marker} {new_title}\n\n");
    let insertion = ByteRange::empty_at(insert_pos);
    let result = doc.apply_replacement(insertion, &heading_text)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text: String::new(),
        new_range: result.new_range,
        new_text: heading_text,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

/// Removes the heading marker of `id`, preserving its title as plain leading
/// text and merging its body into the preceding sibling's body. Both sections
/// must share the same parent.
pub(crate) fn merge_with_prev_sibling(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::NoAdjacentSibling);
    }
    // Find the previous sibling.
    let parent = doc
        .outline()
        .node(node.parent_id.unwrap())
        .ok_or(StructuralEditError::StaleNode(id))?;
    let siblings = &parent.children;
    let pos = siblings
        .iter()
        .position(|&c| c == id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if pos == 0 {
        return Err(StructuralEditError::NoAdjacentSibling);
    }
    let prev = doc
        .outline()
        .node(siblings[pos - 1])
        .ok_or(StructuralEditError::StaleNode(id))?;
    if !prev.children.is_empty() {
        return Err(StructuralEditError::UnsafePreservation);
    }
    // Replace the heading element with its parsed plain title so merging does
    // not silently drop user-authored heading text.
    let heading_range = node.heading_range;
    let replacement =
        plain_heading_replacement(&doc.source()[heading_range.as_range()], &node.title);
    let old_text = doc.source()[heading_range.as_range()].to_string();
    let result = doc.apply_replacement(heading_range, &replacement)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text,
        new_range: result.new_range,
        new_text: replacement,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

fn plain_heading_replacement(heading_source: &str, title: &str) -> String {
    if title.is_empty() {
        return String::new();
    }
    let newline = if heading_source.ends_with("\r\n") {
        "\r\n"
    } else if heading_source.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    format!("{title}{newline}")
}
