//! Delete, split, and merge section operations (RFC-025).

use crate::Document;
use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::index::outline::{HeadingLevel, NodeId};
use crate::range::ByteRange;

use super::error::StructuralEditError;
use super::preflight::{atx_title_range, check_revision, document_newline, joining_separator};

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
    let source = doc.source();
    // RFC-065 §2.6/§4.1: removing `range` entirely makes whatever was on
    // either side of it directly adjacent — a *removal* seam, not an
    // insertion one, so the question is what becomes adjacent once the
    // range is gone, not what is being inserted (`joining_separator` takes
    // both sides regardless of which case created the boundary).
    let newline = document_newline(source);
    let left = &source[..range.start];
    let right = &source[range.end..];
    let replacement = joining_separator(left, right, newline).to_string();
    let old_text = source[range.as_range()].to_string();
    let result = doc.apply_replacement(range, &replacement)?;
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

    // RFC-065 §2.5/§4.4: use the document's own newline sequence for the
    // inserted heading, rather than a hardcoded `\n` that would leave a
    // CRLF file with one LF-terminated line among its CRLF ones.
    let newline = document_newline(doc.source());
    let marker = "#".repeat(new_level.as_u8() as usize);
    let heading_text = format!("{newline}{marker} {new_title}{newline}{newline}");
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
    // RFC-065 §2.4/§4.2: strip only the heading marker, keeping every other
    // source byte of the line verbatim — inline markup, link URLs, code
    // spans, inline HTML. `node.title` (used previously) is the indexer's
    // *flattened* plain-text title, which is exactly what destroyed a
    // link's URL and any emphasis/code-span markers: merge must operate on
    // the source line, not the derived title string.
    let heading_range = node.heading_range;
    let heading_source = &doc.source()[heading_range.as_range()];
    let replacement =
        strip_heading_marker(heading_source).ok_or(StructuralEditError::UnsafePreservation)?;
    let old_text = heading_source.to_string();
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

/// Splits `s` at its first line break into `(line, terminator, rest)`,
/// where `terminator` is `"\r\n"`, `"\n"`, or `""` (no break found) and
/// `line` never includes it — unlike a plain `s.find('\n')` split, this
/// correctly keeps a `\r` immediately before `\n` as part of the
/// terminator rather than as the last byte of `line`.
fn split_first_line(s: &str) -> (&str, &str, &str) {
    match s.find('\n') {
        Some(i) if i > 0 && s.as_bytes()[i - 1] == b'\r' => {
            (&s[..i - 1], &s[i - 1..=i], &s[i + 1..])
        }
        Some(i) => (&s[..i], &s[i..=i], &s[i + 1..]),
        None => (s, "", ""),
    }
}

/// Strips a heading's marker — ATX `#`s, or the setext underline line —
/// while preserving every other byte of the title line verbatim (RFC-065
/// §4.2). Returns `None` for a heading shape this does not recognise: any
/// ATX line `atx_title_range` itself rejects (more than 6 `#`s, or none),
/// or anything other than a title line followed by a line of only `=` or
/// only `-` for setext. The caller refuses with `UnsafePreservation` rather
/// than guess — that variant exists for exactly this (RFC-065 §4.2, §5).
fn strip_heading_marker(heading_source: &str) -> Option<String> {
    let (first_line, first_terminator, rest) = split_first_line(heading_source);

    if first_line.starts_with('#') {
        let title_range = atx_title_range(0, first_line)?;
        let title = &first_line[title_range.as_range()];
        return Some(format!("{title}{first_terminator}"));
    }

    // Setext: `first_line` is the title text (kept verbatim, markup and
    // all); `rest` must be exactly one underline line of only `=` or only
    // `-`, and nothing else — anything past it is not a shape this
    // function recognises as a single heading element.
    let (second_line, _second_terminator, tail) = split_first_line(rest);
    let is_underline = !second_line.is_empty()
        && (second_line.bytes().all(|b| b == b'=') || second_line.bytes().all(|b| b == b'-'));
    if tail.is_empty() && is_underline {
        return Some(format!("{first_line}{first_terminator}"));
    }

    None
}

#[cfg(test)]
mod strip_heading_marker_tests {
    use super::strip_heading_marker;

    #[test]
    fn atx_strips_marker_and_keeps_markup() {
        assert_eq!(
            strip_heading_marker("## **bold** and [link](http://x)\n"),
            Some("**bold** and [link](http://x)\n".to_string())
        );
    }

    #[test]
    fn atx_strips_optional_closing_hashes() {
        assert_eq!(
            strip_heading_marker("## Title ##\n"),
            Some("Title\n".to_string())
        );
    }

    #[test]
    fn atx_preserves_crlf_terminator() {
        assert_eq!(
            strip_heading_marker("## Title\r\n"),
            Some("Title\r\n".to_string())
        );
    }

    #[test]
    fn atx_with_no_trailing_newline() {
        assert_eq!(strip_heading_marker("## Title"), Some("Title".to_string()));
    }

    #[test]
    fn setext_strips_underline_and_keeps_markup() {
        assert_eq!(
            strip_heading_marker("**bold** and [link](http://x)\n===\n"),
            Some("**bold** and [link](http://x)\n".to_string())
        );
    }

    #[test]
    fn setext_h2_dash_underline() {
        assert_eq!(
            strip_heading_marker("Title\n---\n"),
            Some("Title\n".to_string())
        );
    }

    #[test]
    fn empty_title_keeps_only_the_terminator() {
        // "## \n" -> marker + one space, nothing else: the title span is
        // empty, so only the heading line's own terminator remains.
        assert_eq!(strip_heading_marker("## \n"), Some("\n".to_string()));
    }

    // The following are defensive: every heading `merge_with_prev_sibling`
    // is ever called on was already recognised as a heading by the
    // indexer's own pulldown-cmark-driven parse (RFC-006/007), so a
    // genuinely reachable "heading pulldown-cmark accepts but this
    // function refuses" input was not found within this slice's scope —
    // mirroring `atx_title_range`'s own `marker_len == 0 || > 6` checks in
    // `rename.rs`, which are equally unreachable through a real ATX
    // heading for the same reason. These document the *contract*
    // (`UnsafePreservation` over guessing) rather than a confirmed live
    // repro; a source of real end-to-end unrecognised shapes, if one
    // exists, is future work, not asserted here.
    #[test]
    fn more_than_six_hashes_is_not_recognised() {
        assert_eq!(strip_heading_marker("####### Title\n"), None);
    }

    #[test]
    fn a_lone_paragraph_line_with_no_underline_is_not_recognised() {
        assert_eq!(strip_heading_marker("just a paragraph\n"), None);
    }

    #[test]
    fn setext_title_followed_by_extra_content_is_not_recognised() {
        assert_eq!(strip_heading_marker("Title\n===\nextra\n"), None);
    }
}
