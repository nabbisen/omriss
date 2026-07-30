//! Rename section (RFC-049).

use crate::Document;
use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::NodeId;
use crate::range::ByteRange;

use super::error::StructuralEditError;
use super::preflight::check_revision;

pub(crate) fn rename_section(
    doc: &mut Document,
    id: NodeId,
    new_title: &str,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let new_title = new_title.trim();
    if new_title.is_empty() || new_title.contains('\n') || new_title.contains('\r') {
        return Err(StructuralEditError::InvalidTitle);
    }

    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }

    let source = doc.source();
    let heading = &source[node.heading_range.as_range()];
    let line_len = heading.find('\n').unwrap_or(heading.len());
    let line = &heading[..line_len];
    let title_range = if line.starts_with('#') {
        atx_title_range(node.heading_range.start, line).ok_or(StructuralEditError::InvalidTitle)?
    } else {
        ByteRange::new(
            node.heading_range.start,
            node.heading_range.start + line_len,
        )
        .map_err(|_| StructuralEditError::InvalidTitle)?
    };

    let old_text = source[title_range.as_range()].to_string();
    let result = doc.apply_replacement(title_range, new_title)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text,
        new_range: result.new_range,
        new_text: new_title.to_string(),
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

fn atx_title_range(line_start: usize, line: &str) -> Option<ByteRange> {
    let bytes = line.as_bytes();
    let marker_len = bytes.iter().take_while(|&&b| b == b'#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }

    let mut title_start = marker_len;
    while matches!(bytes.get(title_start), Some(b' ' | b'\t')) {
        title_start += 1;
    }

    let mut title_end = line.len();
    while title_end > title_start && matches!(bytes[title_end - 1], b' ' | b'\t') {
        title_end -= 1;
    }

    let mut hash_start = title_end;
    while hash_start > title_start && bytes[hash_start - 1] == b'#' {
        hash_start -= 1;
    }
    if hash_start < title_end && hash_start > title_start {
        let mut before_hash = hash_start;
        while before_hash > title_start && matches!(bytes[before_hash - 1], b' ' | b'\t') {
            before_hash -= 1;
        }
        if before_hash < hash_start {
            title_end = before_hash;
        }
    }

    ByteRange::new(line_start + title_start, line_start + title_end).ok()
}
