//! The edit a raw-text draft makes to the document source (RFC-076).
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

/// What a browser text area shows for `text`: every `\r\n` and lone `\r`
/// becomes `\n`. This is also what a draft is compared against.
pub fn as_shown_in_a_text_area(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

/// A single replacement of `range` of the source by `replacement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEdit {
    /// Byte range of the *original* source to replace.
    pub range: Range<usize>,
    /// What replaces it, in the document's own line-ending style.
    pub replacement: String,
}

/// The edit that turns `source` into what `draft` says, or `None` when the
/// draft shows exactly what the source shows (nothing to commit).
pub fn raw_edit(source: &str, draft: &str) -> Option<RawEdit> {
    let shown = as_shown_in_a_text_area(source);
    let wanted = as_shown_in_a_text_area(draft);
    if shown == wanted {
        return None;
    }

    // The unchanged head and tail, measured in the text-area view and kept
    // from overlapping, on character boundaries.
    let head = common_prefix_len(&shown, &wanted);
    let tail = common_suffix_len(&shown[head..], &wanted[head..]);

    let mut start = source_offset_from_start(source, head);
    let end = source.len() - source_len_of_tail(source, tail);
    let mut middle = wanted[head..wanted.len() - tail].to_string();

    // A lone `\r` just before the edit would fuse with an inserted `\n` into
    // one CRLF, so the document would show one newline where the draft has two.
    // (A `\r` that is part of a CRLF is never here: the head ends after the
    // pair.) Take that newline into the edit; it is rewritten in the document's
    // own style, which for a lone `\r` is the only byte the edit may touch
    // beyond the span the user changed.
    if start > 0 && source.as_bytes()[start - 1] == b'\r' && middle.starts_with('\n') {
        start -= 1;
        middle.insert(0, '\n');
    }

    Some(RawEdit {
        range: start..end,
        replacement: with_newlines_of(source, &middle),
    })
}

/// The newline `source` uses for text written into it: CRLF if it contains any
/// `\r\n`, else LF — the same preference core applies (`document_newline`).
fn newline_of(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn with_newlines_of(source: &str, text_area_text: &str) -> String {
    match newline_of(source) {
        "\r\n" => text_area_text.replace('\n', "\r\n"),
        _ => text_area_text.to_string(),
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
