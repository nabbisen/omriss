//! The edit a text-area draft makes to the document source (RFC-076, RFC-084).
//!
//! **Every** text area that commits into the source goes through this module:
//! the plain file text (the whole source), a Markdown section's body, and a JSON
//! group's raw text (a byte range of the source). The one general function is
//! [`text_area_edit`]; [`raw_edit`] is its whole-source form.
//!
//! A text area's value always uses `\n`: the browser turns `\r\n` and a lone
//! `\r` into `\n` before the application sees it. Committing that value as the
//! new source would silently rewrite every line ending of a CRLF file, and
//! "open the raw view, change nothing, save" would no longer leave the file
//! byte-identical — the invariant the whole project rests on.
//!
//! So a draft is never the new source. It is compared with the source *as the
//! text area would show it*, and the difference becomes one minimal replacement
//! of the original bytes: every byte outside the changed span keeps the line
//! ending it had, mixed endings included. Text the user inserted takes the
//! document's own style, as everywhere else in omriss.

use std::ops::Range;

use omriss_core::{document_newline, normalize_to_lf};

/// A single replacement of `range` of the source by `replacement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEdit {
    /// Byte range of the *original* source to replace.
    pub range: Range<usize>,
    /// What replaces it, in the document's own line-ending style.
    pub replacement: String,
}

/// The edit that turns `source` into what `draft` says, or `None` when the
/// draft shows exactly what the source shows (nothing to commit). The whole-source
/// form of [`text_area_edit`]: inserted newlines take the source's own style.
pub fn raw_edit(source: &str, draft: &str) -> Option<RawEdit> {
    text_area_edit(source, draft, document_newline(source))
}

/// The edit that turns `original` — a stretch of the document that a text area
/// showed — into what `draft` says, or `None` when the draft shows exactly what
/// `original` shows.
///
/// The range of the returned edit is relative to `original`. `newline` is the
/// **document's** newline (see [`document_newline`]), not `original`'s: a stretch with
/// no line break in it (a one-line JSON array, an empty body) still belongs to a
/// CRLF document, and a newline the user types into it must be CRLF.
pub fn text_area_edit(original: &str, draft: &str, newline: &str) -> Option<RawEdit> {
    text_area_edit_after("", original, draft, newline)
}

/// [`text_area_edit`] for a stretch that does not begin the document: `before` is
/// the document's text just ahead of `original` (all of it, or its tail).
///
/// Only its last byte matters. A body that follows a heading ended by a lone
/// `\r` must not start with a bare `\n` the edit inserted: the two would fuse
/// into one CRLF, the heading's ending would change and the line break the user
/// typed would vanish (RFC-084 §10.3, D6).
pub fn text_area_edit_after(
    before: &str,
    original: &str,
    draft: &str,
    newline: &str,
) -> Option<RawEdit> {
    let source = original;
    let shown = normalize_to_lf(source);
    let wanted = normalize_to_lf(draft);
    if shown == wanted {
        return None;
    }

    // The unchanged head and tail, measured in the text-area view and kept
    // from overlapping, on character boundaries.
    let head = common_prefix_len(&shown, &wanted);
    let tail = common_suffix_len(&shown[head..], &wanted[head..]);

    let mut start = source_offset_from_start(source, head);
    let mut end = source.len() - source_len_of_tail(source, tail);
    let middle = wanted[head..wanted.len() - tail].to_string();
    let mut replacement = with_newline(newline, &middle);

    // A lone `\r` meeting a `\n` becomes one CRLF, and a text area would then show
    // one line break where the draft has two. That happens when the edit puts a
    // `\n` (typed, or the tail's own) right after a lone `\r`: the user typed a
    // line break there, or deleted the text between a `\r` and a `\n`.
    //
    // * If the `\r` is inside `original`, the edit takes it in and writes it as the
    //   document's newline — the one byte beside the user's span an edit may
    //   rewrite, and again for each lone `\r` before it (a run of them).
    // * If it is outside — a heading's, ahead of a body — it cannot be taken in.
    //   The edit writes its own leading line breaks as `\r`, the neighbour's
    //   style, which cannot fuse; where it deleted all of them, the tail's lone
    //   `\n` run is read as `\r` too (RFC-084 §10.3, D6).
    loop {
        let left = if start > 0 {
            source.as_bytes().get(start - 1).copied()
        } else if source.starts_with('\n') {
            None // the outside `\r` and this `\n` are one CRLF, not a lone `\r`
        } else {
            before.as_bytes().last().copied()
        };
        let right = source.as_bytes().get(end).copied();
        if left == Some(b'\r') && replacement.as_bytes().first().or(right.as_ref()) == Some(&b'\n')
        {
            if start > 0 {
                start -= 1;
                replacement.insert_str(0, newline);
            } else if replacement.starts_with('\n') {
                let run = replacement.bytes().take_while(|&b| b == b'\n').count();
                replacement.replace_range(..run, &"\r".repeat(run));
            } else {
                end += 1;
                replacement.push('\r');
            }
            continue;
        }
        if replacement.ends_with('\r') && right == Some(b'\n') {
            end += 1;
            replacement.push('\r');
            continue;
        }
        break;
    }
    Some(RawEdit {
        range: start..end,
        replacement,
    })
}

/// `original` with `draft`'s changes applied (see [`text_area_edit`]): the new
/// text of that stretch of the document, every byte outside the changed span
/// exactly as it was. `None` when the draft changes nothing.
pub fn apply_text_area_draft(original: &str, draft: &str, newline: &str) -> Option<String> {
    apply_text_area_draft_after("", original, draft, newline)
}

/// [`apply_text_area_draft`] for a stretch that follows `before` in the document
/// (see [`text_area_edit_after`]).
pub fn apply_text_area_draft_after(
    before: &str,
    original: &str,
    draft: &str,
    newline: &str,
) -> Option<String> {
    let edit = text_area_edit_after(before, original, draft, newline)?;
    let mut merged = String::with_capacity(original.len() + edit.replacement.len());
    merged.push_str(&original[..edit.range.start]);
    merged.push_str(&edit.replacement);
    merged.push_str(&original[edit.range.end..]);
    Some(merged)
}

fn with_newline(newline: &str, text_area_text: &str) -> String {
    if newline == "\n" {
        text_area_text.to_string()
    } else {
        text_area_text.replace('\n', newline)
    }
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum()
}

fn common_suffix_len(a: &str, b: &str) -> usize {
    a.chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum()
}

/// The byte offset in `source` of text-area offset `shown_offset`. Each
/// `\r\n` is one text-area byte but two source bytes.
fn source_offset_from_start(source: &str, shown_offset: usize) -> usize {
    let bytes = source.as_bytes();
    let (mut at, mut shown) = (0, 0);
    while shown < shown_offset {
        at += newline_width_at(bytes, at);
        shown += 1;
    }
    at
}

/// How many bytes at the end of `source` correspond to the last `shown_tail`
/// bytes of the text-area view.
fn source_len_of_tail(source: &str, shown_tail: usize) -> usize {
    let bytes = source.as_bytes();
    let (mut at, mut shown) = (bytes.len(), 0);
    while shown < shown_tail {
        at -= if at >= 2 && bytes[at - 2] == b'\r' && bytes[at - 1] == b'\n' {
            2
        } else {
            1
        };
        shown += 1;
    }
    bytes.len() - at
}

fn newline_width_at(bytes: &[u8], at: usize) -> usize {
    if bytes[at] == b'\r' && bytes.get(at + 1) == Some(&b'\n') {
        2
    } else {
        1
    }
}
