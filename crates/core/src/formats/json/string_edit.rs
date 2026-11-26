//! Editing a JSON string value without re-spelling what the user did not touch
//! (RFC-084 §10.1, D5).
//!
//! A string literal is decoded character by character, each remembering the
//! bytes that spell it. A draft is compared with the decoded text, the changed
//! span is found *in the decoded text*, and only the bytes of the characters in
//! that span are replaced — an escape the user did not touch (`é`, `\/`, a
//! surrogate pair, `\r\n`) keeps the spelling the file gave it. This is the
//! same compare-and-splice the text areas use, applied through the JSON string
//! codec.
//!
//! The draft is what a text area delivers: every line break is `\n`. It is
//! compared with the original **as a text area shows it**, so an untouched
//! `\r\n` is never an edit. A line break the user typed is written in the style
//! the string already uses (`\r\n` if it holds one, `\r` if its only breaks are
//! lone `\r`, else `\n`).

use crate::doc::structural::document_newline;
use crate::newline::normalize_to_lf;
use crate::range::ByteRange;

use super::scalar::encode_string_body;
use super::string_literal::{SpelledChar, parse_string_spans};

/// One character a text area shows, and the bytes of the literal behind it.
struct Shown {
    ch: char,
    bytes: ByteRange,
    /// A lone `\r`: shown as a line break, but fusing with a `\n` placed after it.
    lone_cr: bool,
    /// A lone `\n` (not the end of a `\r\n`).
    lone_lf: bool,
}

/// The replacement a draft makes to the string literal at `literal`: a byte
/// range of `source` (inside the literal, or all of it) and what replaces it.
/// `None` when the draft shows exactly what the literal shows.
pub(super) fn string_edit(
    source: &str,
    literal: ByteRange,
    draft: &str,
) -> Option<(ByteRange, String)> {
    let (spelled, _) = parse_string_spans(source, source.as_bytes(), literal.start).ok()?;
    let units = shown_units(&spelled);
    let decoded: String = spelled.iter().map(|(c, _)| *c).collect();
    let style = document_newline(&decoded);

    let wanted: Vec<char> = normalize_to_lf(draft).chars().collect();
    let shown: Vec<char> = units.iter().map(|u| u.ch).collect();
    if shown == wanted {
        return None;
    }
    let head = shown
        .iter()
        .zip(&wanted)
        .take_while(|(a, b)| a == b)
        .count();
    let tail = shown[head..]
        .iter()
        .rev()
        .zip(wanted[head..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    let closing_quote = literal.end - 1;
    let mut head = head;
    let middle: String = wanted[head..wanted.len() - tail].iter().collect();
    let mut replacement = middle.replace('\n', style);

    // A lone `\r` meeting a `\n` becomes one `\r\n`, and the text area would show
    // one line break where the draft has two. That happens when the edit puts a
    // `\n` right after a lone `\r` — typed, or the tail's own after a deletion.
    // The edit takes the `\r` in and writes it as the string's own newline (and
    // each lone `\r` before it, for a run): the one kind of byte beside the
    // user's span an edit may rewrite.
    let right_is_lone_lf = tail > 0 && units[units.len() - tail].lone_lf;
    while head > 0
        && units[head - 1].lone_cr
        && (replacement.starts_with('\n') || (replacement.is_empty() && right_is_lone_lf))
    {
        head -= 1;
        replacement.insert_str(0, style);
    }

    let start = units.get(head).map_or(closing_quote, |u| u.bytes.start);
    let end = if tail == 0 {
        closing_quote
    } else {
        units[units.len() - tail].bytes.start
    };
    let escaped = encode_string_body(&replacement);
    if head == 0 && tail == 0 {
        // Nothing of the old text survives: the whole literal is new.
        return Some((literal, format!("\"{escaped}\"")));
    }
    Some((ByteRange { start, end }, escaped))
}

fn shown_units(spelled: &[SpelledChar]) -> Vec<Shown> {
    let mut units = Vec::with_capacity(spelled.len());
    let mut i = 0;
    while i < spelled.len() {
        let (ch, bytes) = spelled[i];
        if ch == '\r' {
            if let Some(&('\n', next)) = spelled.get(i + 1) {
                units.push(Shown {
                    ch: '\n',
                    bytes: ByteRange {
                        start: bytes.start,
                        end: next.end,
                    },
                    lone_cr: false,
                    lone_lf: false,
                });
                i += 2;
                continue;
            }
            units.push(Shown {
                ch: '\n',
                bytes,
                lone_cr: true,
                lone_lf: false,
            });
        } else {
            units.push(Shown {
                ch,
                bytes,
                lone_cr: false,
                lone_lf: ch == '\n',
            });
        }
        i += 1;
    }
    units
}
