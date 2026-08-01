//! Strict RFC 8259 string literal decoding: escapes, surrogate pairs, and
//! control-character rejection.
//!
//! Split out of `scanner.rs` (which was over the 500 ELOC split
//! threshold): decoding one quoted string is a self-contained concern with
//! no dependency on the container-parsing logic around it, and vice versa.

use crate::range::ByteRange;

use super::scanner::JsonParseError;

/// Parses a quoted string starting at `pos` (which must be `"` in `bytes`),
/// returning its decoded (unescaped) text, the byte range of the whole
/// literal including both quotes, and the position just past it.
///
/// Operates on raw bytes for structural dispatch (quote, backslash,
/// control-character rejection), which is safe without decoding
/// codepoints: `source` is guaranteed valid UTF-8 (it is `&str`), and
/// every byte of a multi-byte UTF-8 sequence is >= 0x80 — it can never
/// collide with `"` (0x22), `\` (0x5C), or a control byte (< 0x20).
/// Multi-byte characters are copied through verbatim via `source` slicing,
/// never rebuilt byte-by-byte.
pub(super) fn parse_string(
    source: &str,
    bytes: &[u8],
    pos: usize,
) -> Result<(String, ByteRange, usize), JsonParseError> {
    let start = pos;
    let mut cur = pos + 1;
    let mut decoded = String::new();
    let mut run_start = cur;

    loop {
        match bytes.get(cur).copied() {
            None => {
                return Err(JsonParseError {
                    at: cur,
                    reason: "unterminated string literal",
                });
            }
            Some(b'"') => {
                decoded.push_str(&source[run_start..cur]);
                cur += 1;
                break;
            }
            Some(b'\\') => {
                decoded.push_str(&source[run_start..cur]);
                let (ch, next) = parse_escape(bytes, cur)?;
                decoded.push(ch);
                cur = next;
                run_start = cur;
            }
            Some(b) if b < 0x20 => {
                return Err(JsonParseError {
                    at: cur,
                    reason: "unescaped control character in string",
                });
            }
            Some(b) => {
                cur += utf8_char_len(b);
            }
        }
    }

    Ok((decoded, ByteRange { start, end: cur }, cur))
}

/// `backslash_pos` is the byte offset of the `\`. Returns the decoded
/// character and the position just past the whole escape sequence.
fn parse_escape(bytes: &[u8], backslash_pos: usize) -> Result<(char, usize), JsonParseError> {
    let esc_pos = backslash_pos + 1;
    match bytes.get(esc_pos).copied() {
        Some(b'"') => Ok(('"', esc_pos + 1)),
        Some(b'\\') => Ok(('\\', esc_pos + 1)),
        Some(b'/') => Ok(('/', esc_pos + 1)),
        Some(b'b') => Ok(('\u{8}', esc_pos + 1)),
        Some(b'f') => Ok(('\u{c}', esc_pos + 1)),
        Some(b'n') => Ok(('\n', esc_pos + 1)),
        Some(b'r') => Ok(('\r', esc_pos + 1)),
        Some(b't') => Ok(('\t', esc_pos + 1)),
        Some(b'u') => parse_unicode_escape(bytes, esc_pos + 1),
        Some(_) => Err(JsonParseError {
            at: esc_pos,
            reason: "invalid escape sequence",
        }),
        None => Err(JsonParseError {
            at: esc_pos,
            reason: "unterminated escape sequence",
        }),
    }
}

/// `pos` is just past `\u`. Handles surrogate pairs (RFC 8259 permits
/// encoding astral codepoints as a `\uD800`-`\uDBFF` high surrogate
/// immediately followed by a `\uDC00`-`\uDFFF` low surrogate).
fn parse_unicode_escape(bytes: &[u8], pos: usize) -> Result<(char, usize), JsonParseError> {
    let high = parse_hex4(bytes, pos)?;
    let end = pos + 4;

    if (0xD800..=0xDBFF).contains(&high) {
        if bytes.get(end..end + 2) == Some(b"\\u") {
            let low = parse_hex4(bytes, end + 2)?;
            if (0xDC00..=0xDFFF).contains(&low) {
                let combined =
                    0x10000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
                let ch = char::from_u32(combined).ok_or(JsonParseError {
                    at: pos,
                    reason: "invalid surrogate pair",
                })?;
                return Ok((ch, end + 6));
            }
        }
        return Err(JsonParseError {
            at: pos,
            reason: "unpaired high surrogate in \\u escape",
        });
    }
    if (0xDC00..=0xDFFF).contains(&high) {
        return Err(JsonParseError {
            at: pos,
            reason: "unpaired low surrogate in \\u escape",
        });
    }

    let ch = char::from_u32(u32::from(high)).ok_or(JsonParseError {
        at: pos,
        reason: "invalid \\u escape",
    })?;
    Ok((ch, end))
}

fn parse_hex4(bytes: &[u8], pos: usize) -> Result<u16, JsonParseError> {
    let slice = bytes.get(pos..pos + 4).ok_or(JsonParseError {
        at: pos,
        reason: "incomplete \\u escape",
    })?;
    let text = std::str::from_utf8(slice).map_err(|_| JsonParseError {
        at: pos,
        reason: "invalid \\u escape",
    })?;
    u16::from_str_radix(text, 16).map_err(|_| JsonParseError {
        at: pos,
        reason: "invalid hex digits in \\u escape",
    })
}

/// Byte length of the UTF-8 character starting with `lead_byte`. Valid for
/// any byte of a valid UTF-8 string (continuation bytes never reach here,
/// since callers only invoke this at a character boundary).
fn utf8_char_len(lead_byte: u8) -> usize {
    if lead_byte & 0x80 == 0 {
        1
    } else if lead_byte & 0xE0 == 0xC0 {
        2
    } else if lead_byte & 0xF0 == 0xE0 {
        3
    } else {
        4
    }
}
