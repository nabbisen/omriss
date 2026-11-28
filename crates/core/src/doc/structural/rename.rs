//! Rename section (RFC-049).

use crate::Document;
use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::NodeId;
use crate::newline::split_first_line;
use crate::range::ByteRange;

use super::error::StructuralEditError;
use super::preflight::{atx_title_range, check_revision, starts_with_atx_heading, validate_title};

pub(crate) fn rename_section(
    doc: &mut Document,
    id: NodeId,
    new_title: &str,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let new_title = validate_title(new_title)?;

    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }

    let source = doc.source();
    let heading = &source[node.heading_range.as_range()];
    // The title line ends at the first line break of any of the three endings.
    let (line, _, _) = split_first_line(heading);
    let line_len = line.len();
    let title_range = if starts_with_atx_heading(line) {
        atx_title_range(node.heading_range.start, line).ok_or(StructuralEditError::InvalidTitle)?
    } else {
        // A setext heading: every title line, up to but not including the underline
        // (RFC-070 §10). A title that spans lines is replaced whole, so the heading
        // reads what was asked; the underline and the line endings stay as they are.
        ByteRange::new(
            node.heading_range.start,
            node.heading_range.start + setext_title_len(heading, line_len),
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

/// The length of the title of a setext heading, from its first byte to the end of
/// its last title line (before that line's own line break). `heading` is the
/// heading's source: title line(s), then the underline line, then perhaps a line
/// break. A heading of one line has no underline to stop at, so the length of its
/// first line (`first_line_len`) is the answer.
fn setext_title_len(heading: &str, first_line_len: usize) -> usize {
    let mut rest = heading;
    let mut offset = 0;
    // (start, length) of the line before the current one: the last title line.
    let mut previous: Option<(usize, usize)> = None;
    let mut last_title_end = first_line_len;
    while !rest.is_empty() {
        let (line, terminator, tail) = split_first_line(rest);
        if let Some((start, len)) = previous {
            last_title_end = start + len;
        }
        previous = Some((offset, line.len()));
        offset += line.len() + terminator.len();
        rest = tail;
    }
    last_title_end
}
