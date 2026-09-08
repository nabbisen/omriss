//! Shared preflight checks for structural editing operations.

use crate::doc::revision::DocumentRevision;
use crate::index::outline::NodeId;
use crate::range::ByteRange;
use crate::{Document, Outline};

use super::error::StructuralEditError;

pub(super) fn check_revision(
    doc: &Document,
    base: DocumentRevision,
) -> Result<(), StructuralEditError> {
    if doc.revision() != base {
        return Err(StructuralEditError::RevisionMismatch {
            expected: doc.revision(),
            actual: base,
        });
    }
    Ok(())
}

pub(super) fn is_descendant(outline: &Outline, ancestor: NodeId, candidate: NodeId) -> bool {
    outline
        .path(candidate)
        .map(|path| path.iter().any(|n| n.id == ancestor))
        .unwrap_or(false)
}

/// Returns the separator that must be inserted between `left` and `right`
/// for the result to remain structurally well-formed, using the document's
/// own newline sequence (RFC-065 §4.1).
///
/// Every structural splice site concatenates two pieces of text that used
/// to be separated by a real document boundary (a heading line's own
/// newline, or the gap before the next heading) but, after the splice, sit
/// directly adjacent in the new source. Gluing them together with no
/// protection at all welds two lines into one — turning what was meant to
/// stay a heading line into plain paragraph text (RFC-065 §2.1–§2.4, §2.6).
///
/// ## Why a single line break is sometimes not enough
///
/// A single line break is sufficient when `right` begins with something
/// CommonMark always treats as starting a fresh block regardless of what
/// precedes it: an ATX heading marker (`#`), or `right` already starting
/// with its own line break.
///
/// It is **not** sufficient when `right` could begin with a plain
/// paragraph-shaped line, because CommonMark merges consecutive non-blank
/// lines into *one* paragraph — only a blank line (two line breaks, not
/// one) starts a genuinely new block. If a setext underline (`===`/`---`)
/// later appears in what became that merged paragraph, it captures the
/// *entire* merged paragraph as its title, not just its own line. This is
/// RFC-065 §2.6's exact mechanism (`"0"` + `"A"` merging into a corrupted
/// `"0 A"` heading once the line between them is removed) — and it recurs
/// for `move_section` and `delete_section` in general, not only for the one
/// case the RFC's example happened to show: `left` ending in a single
/// existing line break is *not* proof the boundary is safe, if `right` is
/// paragraph-shaped and a setext underline follows it. This was found only
/// empirically, by RFC-066's property tests still failing after an initial
/// single-line-break-only implementation of this function — the RFC's own
/// root-cause model (`left ends in newline` / `right starts with newline`)
/// undercounted what "already well-formed" requires.
///
/// So: `required` is 1 when `right` starts with `#` (ATX-safe), else 2
/// (paragraph-shaped, setext-vulnerable). `existing` counts the line
/// breaks already present at the boundary — trailing on `left` plus
/// leading on `right`, each of `"\r\n"`/`"\n"`/`"\r"` counting as one.
/// Returns `""` when `existing >= required`, `newline` when exactly one
/// more break is needed, or a blank line built from `newline` when two
/// more are needed (i.e. no break exists yet and `right` is
/// setext-vulnerable).
///
/// ## When `right` is freshly-authored content, not existing structure
///
/// This blank-line caution only matters when `right` represents *existing*
/// document content that this splice makes newly adjacent to `left` — a
/// following heading in a move or delete, or a relocated subtree. When
/// `right` is instead brand-new content the caller is writing for the
/// first time (`replace_section_body`'s left edge, where `right` is the
/// caller's `new_body`), there is no pre-existing structure on the other
/// side of it to protect from a merge — whatever shape `right` itself ends
/// up forming (even a heading the caller's own text happens to spell out)
/// is simply what was written, not a corruption. Use
/// [`heading_line_terminator`] for that case instead: it only needs to
/// answer "does the heading line this body follows correctly end," which a
/// single line break always answers.
pub(crate) fn joining_separator<'a>(left: &str, right: &str, newline: &'a str) -> &'a str {
    if left.is_empty() || right.is_empty() {
        return "";
    }
    // ATX headings interrupt a paragraph safely with just one line break —
    // CommonMark does not require a blank line before them, unlike setext.
    // Anything else on the right could be (or start with) a paragraph line
    // that a later setext underline would capture once merged with
    // whatever precedes it, so it needs a genuine blank line: a single
    // existing break is not enough on its own to prove the boundary safe.
    let right_is_atx = right.trim_start_matches(['\n', '\r']).starts_with('#');
    let required: usize = if right_is_atx { 1 } else { 2 };
    let existing = trailing_break_count(left) + leading_break_count(right);
    match required.saturating_sub(existing) {
        0 => "",
        1 => newline,
        _ => blank_line(newline),
    }
}

/// The separator needed so `left` (ending in an existing heading or body
/// line) correctly terminates before fresh, caller-authored `right` starts
/// — RFC-065 §2.1's left edge specifically, where `right` is brand-new
/// content with no pre-existing structure on its far side to protect. A
/// single line break always suffices here: a heading line just needs its
/// own line to end, and once it does, whatever `right` contains starts a
/// genuinely new block regardless of shape (see [`joining_separator`]'s
/// doc comment for why that is *not* true when `right` is existing
/// structure instead).
pub(crate) fn heading_line_terminator<'a>(left: &str, right: &str, newline: &'a str) -> &'a str {
    if left.is_empty() || right.is_empty() {
        return "";
    }
    if trailing_break_count(left) >= 1 || leading_break_count(right) >= 1 {
        return "";
    }
    newline
}

/// Number of line breaks (each of `"\r\n"`, `"\n"`, `"\r"` counting as one)
/// that `s` ends with.
fn trailing_break_count(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    let mut count = 0;
    loop {
        if i > 0 && bytes[i - 1] == b'\n' {
            i -= if i >= 2 && bytes[i - 2] == b'\r' {
                2
            } else {
                1
            };
        } else if i > 0 && bytes[i - 1] == b'\r' {
            i -= 1;
        } else {
            break;
        }
        count += 1;
    }
    count
}

/// Number of line breaks that `s` starts with, by the same counting rule
/// as [`trailing_break_count`].
fn leading_break_count(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut count = 0;
    loop {
        if bytes.get(i) == Some(&b'\r') {
            i += if bytes.get(i + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
        } else if bytes.get(i) == Some(&b'\n') {
            i += 1;
        } else {
            break;
        }
        count += 1;
    }
    count
}

/// A blank line built from the document's own newline sequence: `"\n\n"`
/// for LF, `"\r\n\r\n"` for CRLF.
fn blank_line(newline: &str) -> &'static str {
    if newline == "\r\n" {
        "\r\n\r\n"
    } else {
        "\n\n"
    }
}

/// The document's own newline sequence, for splice sites that need to
/// insert one (RFC-065 §4.4: derive it from the document, never assume
/// `"\n"`). `"\r\n"` if it appears anywhere in `source`, else `"\n"` — a
/// single sequence to insert, not the three-way `NewlinePolicy` label the
/// UI layer tracks for display; a splice site only ever needs to know what
/// to *write*, and a document with any CRLF at all should get CRLF
/// insertions rather than an LF that would itself create a mixed-ending
/// file (matching `plain_heading_replacement`'s existing per-line technique
/// at `delete_split_merge.rs`, generalized to whole-document detection for
/// sites that have no single nearby line to sample).
pub(crate) fn document_newline(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// The title span of a single ATX heading line (e.g. `"## Title ##"`),
/// excluding the opening marker and any optional closing `#` run —
/// CommonMark treats a trailing, whitespace-separated run of `#`s as
/// decoration, not title content. `line` must not include its own
/// trailing newline; `line_start` is its absolute byte offset in the
/// document, added to the returned range.
///
/// Shared by `rename.rs` (replacing the title span) and `delete_split_merge.rs`
/// (RFC-065 §4.2: join preserves everything in this span verbatim, inline
/// markup included, instead of the indexer's flattened plain-text title).
/// Returns `None` for a line that isn't a well-formed ATX heading (no `#`,
/// or more than 6) — the caller decides what "not recognised" means for its
/// own operation.
pub(crate) fn atx_title_range(line_start: usize, line: &str) -> Option<ByteRange> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_separator_when_left_already_ends_in_newline_before_atx() {
        assert_eq!(joining_separator("body\n", "# Heading", "\n"), "");
    }

    #[test]
    fn no_separator_when_right_already_starts_with_newline_before_atx() {
        assert_eq!(joining_separator("body", "\n# Heading", "\n"), "");
    }

    #[test]
    fn separator_needed_when_neither_side_has_one() {
        assert_eq!(joining_separator("body", "# Heading", "\n"), "\n");
    }

    #[test]
    fn a_single_break_is_not_enough_before_a_non_atx_right() {
        // `right` is a plain paragraph-shaped line ("next"), not an ATX
        // heading — a single existing break does not prove the boundary
        // safe against a setext underline appearing later in `right`
        // (RFC-065 §2.6's mechanism), so a full blank line is required.
        assert_eq!(joining_separator("body\n", "next", "\n"), "\n");
    }

    #[test]
    fn a_single_leading_break_on_a_non_atx_right_is_also_not_enough() {
        assert_eq!(joining_separator("body", "\nnext", "\n"), "\n");
    }

    #[test]
    fn no_separator_when_already_a_blank_line_before_non_atx_right() {
        assert_eq!(joining_separator("body\n\n", "next", "\n"), "");
        assert_eq!(joining_separator("body", "\n\nnext", "\n"), "");
    }

    #[test]
    fn blank_line_inserted_when_neither_side_has_any_break_before_non_atx_right() {
        assert_eq!(joining_separator("body", "next", "\n"), "\n\n");
        assert_eq!(joining_separator("body", "next", "\r\n"), "\r\n\r\n");
    }

    #[test]
    fn breaks_split_across_both_sides_still_complete_a_blank_line() {
        // One break at the end of `left`, one at the start of `right`:
        // together they already form a blank line once concatenated.
        assert_eq!(joining_separator("body\n", "\nnext", "\n"), "");
    }

    #[test]
    fn no_separator_when_left_is_empty() {
        assert_eq!(joining_separator("", "# Heading", "\n"), "");
    }

    #[test]
    fn no_separator_when_right_is_empty() {
        assert_eq!(joining_separator("body", "", "\n"), "");
    }

    #[test]
    fn both_empty_needs_no_separator() {
        assert_eq!(joining_separator("", "", "\n"), "");
    }

    #[test]
    fn uses_the_documents_own_newline_sequence() {
        assert_eq!(joining_separator("body", "# Heading", "\r\n"), "\r\n");
    }

    #[test]
    fn crlf_left_already_terminated_needs_no_separator() {
        assert_eq!(joining_separator("body\r\n", "# Heading", "\r\n"), "");
    }

    #[test]
    fn document_newline_detects_crlf() {
        assert_eq!(document_newline("# A\r\nbody\r\n"), "\r\n");
    }

    #[test]
    fn document_newline_defaults_to_lf() {
        assert_eq!(document_newline("# A\nbody\n"), "\n");
        assert_eq!(document_newline("# A"), "\n");
        assert_eq!(document_newline(""), "\n");
    }

    #[test]
    fn document_newline_prefers_crlf_when_mixed() {
        // Any CRLF present at all means CRLF insertions, rather than an LF
        // that would create a third line-ending style in an already-mixed
        // file.
        assert_eq!(document_newline("# A\r\nbody\nmore\n"), "\r\n");
    }

    #[test]
    fn a_bare_lf_still_terminates_a_line_in_a_crlf_document() {
        // CommonMark accepts LF, CR, and CRLF equally as a line boundary, so
        // `left` ending in a lone `\n` already terminates its line even
        // though the document's own convention is CRLF — no second
        // separator is inserted on top of it. This is what
        // `source_preservation.rs`'s `replacing_any_body_preserves_all_unrelated_bytes`
        // caught against `crlf.md`, where the caller-supplied replacement
        // body ends in a bare `\n`.
        assert_eq!(joining_separator("body\n", "# Heading", "\r\n"), "");
    }

    #[test]
    fn a_bare_cr_also_terminates_a_line() {
        assert_eq!(joining_separator("body\r", "# Heading", "\n"), "");
    }
}
