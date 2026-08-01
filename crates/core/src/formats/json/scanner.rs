//! Strict RFC 8259 JSON scanner producing a byte-range-annotated value tree
//! (RFC-054 §7.2).
//!
//! Hand-written rather than built on an existing crate for two reasons no
//! `serde_json::Value`-shaped API can satisfy at once: `JsonAdapter::build_structure`
//! needs the exact byte range of every value (`serde_json::Value` discards
//! source position once parsed), and every duplicate object key must be
//! preserved in source order, never merged (RFC-054 §15.2) — a `Map`
//! representation cannot represent two entries with the same key at all.
//!
//! Not tolerant: no comments, no trailing commas, no unquoted keys, no
//! single-quoted strings, no `NaN`/`Infinity`, no leading `+` or leading
//! zeros on numbers (RFC-052 §14.1, RFC-054 §0.5). Anything a permissive
//! "JSON5"-style reader would accept, this rejects with a typed error.

use crate::range::ByteRange;

use super::string_literal;

/// One parsed JSON value, with the exact byte range of its literal text
/// (the whole `{...}`/`[...]`/literal, not just its content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum JsonValue {
    Object {
        members: Vec<JsonMember>,
        range: ByteRange,
    },
    Array {
        items: Vec<JsonValue>,
        range: ByteRange,
    },
    String {
        range: ByteRange,
    },
    Number {
        range: ByteRange,
    },
    Bool {
        value: bool,
        range: ByteRange,
    },
    Null {
        range: ByteRange,
    },
}

impl JsonValue {
    pub(super) fn range(&self) -> ByteRange {
        match self {
            JsonValue::Object { range, .. }
            | JsonValue::Array { range, .. }
            | JsonValue::String { range }
            | JsonValue::Number { range }
            | JsonValue::Bool { range, .. }
            | JsonValue::Null { range } => *range,
        }
    }
}

/// One `"key": value` entry of a [`JsonValue::Object`], in source order.
/// Never merged with a same-named sibling (RFC-054 §15.2): each member
/// keeps its own value and its own tree position, however many siblings
/// share its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct JsonMember {
    pub key: String,
    pub key_range: ByteRange,
    pub value: JsonValue,
}

/// A strict RFC 8259 parse failure. `at` is the byte offset the scanner had
/// reached when it rejected the input; `reason` is a short, code-facing
/// description for tests and diagnostics — never a user-facing string
/// (`omriss-core` must never name an i18n catalog key, RFC-001; that
/// mapping is J4's `omriss-ui` table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct JsonParseError {
    pub at: usize,
    pub reason: &'static str,
}

/// Maximum container nesting depth. Enforced *before* recursing, so it
/// bounds the scanner's own call stack, not just the values it is willing
/// to describe. 128 is deep enough for any realistic document — RFC-054
/// §12.1's "deep nesting within limit" test targets exactly this — and
/// shallow enough that even a pessimistic per-frame stack cost could never
/// approach a thread's stack budget; a naive recursive-descent parser with
/// no limit at all is exactly the stack-overflow risk RFC-053 §18 raised
/// for Markdown, except Markdown's fix was discovering the shipped indexer
/// was already iterative — this scanner is genuinely recursive, so the
/// limit is the mitigation, not a discovery about existing code.
const MAX_NESTING_DEPTH: usize = 128;

/// Parses `source` as a single strict JSON document: exactly one value,
/// optionally surrounded by whitespace, nothing else.
pub(super) fn parse(source: &str) -> Result<JsonValue, JsonParseError> {
    let scanner = Scanner {
        source,
        bytes: source.as_bytes(),
    };
    let start = scanner.skip_ws(0);
    let (value, end) = scanner.parse_value(start, 0)?;
    let end = scanner.skip_ws(end);
    if end != scanner.bytes.len() {
        return Err(JsonParseError {
            at: end,
            reason: "trailing content after the top-level value",
        });
    }
    Ok(value)
}

struct Scanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
}

impl Scanner<'_> {
    fn get(&self, pos: usize) -> Option<u8> {
        self.bytes.get(pos).copied()
    }

    fn skip_ws(&self, mut pos: usize) -> usize {
        while matches!(self.get(pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            pos += 1;
        }
        pos
    }

    fn parse_value(&self, pos: usize, depth: usize) -> Result<(JsonValue, usize), JsonParseError> {
        if depth > MAX_NESTING_DEPTH {
            return Err(JsonParseError {
                at: pos,
                reason: "container nesting exceeds the supported limit",
            });
        }
        match self.get(pos) {
            Some(b'{') => self.parse_object(pos, depth),
            Some(b'[') => self.parse_array(pos, depth),
            Some(b'"') => {
                let (_, range, end) = self.parse_string(pos)?;
                Ok((JsonValue::String { range }, end))
            }
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(pos),
            Some(b't') => self.parse_keyword(
                pos,
                "true",
                JsonValue::Bool {
                    value: true,
                    range: ByteRange {
                        start: pos,
                        end: pos,
                    },
                },
            ),
            Some(b'f') => self.parse_keyword(
                pos,
                "false",
                JsonValue::Bool {
                    value: false,
                    range: ByteRange {
                        start: pos,
                        end: pos,
                    },
                },
            ),
            Some(b'n') => self.parse_keyword(
                pos,
                "null",
                JsonValue::Null {
                    range: ByteRange {
                        start: pos,
                        end: pos,
                    },
                },
            ),
            Some(_) => Err(JsonParseError {
                at: pos,
                reason: "unexpected character; expected a JSON value",
            }),
            None => Err(JsonParseError {
                at: pos,
                reason: "unexpected end of input; expected a JSON value",
            }),
        }
    }

    /// Matches a fixed keyword (`true`/`false`/`null`) and stamps `template`
    /// with the keyword's real range. `template`'s own range field is a
    /// placeholder overwritten here so callers don't repeat the length
    /// arithmetic.
    fn parse_keyword(
        &self,
        pos: usize,
        keyword: &'static str,
        template: JsonValue,
    ) -> Result<(JsonValue, usize), JsonParseError> {
        let end = pos + keyword.len();
        if self.bytes.get(pos..end) == Some(keyword.as_bytes()) {
            let range = ByteRange { start: pos, end };
            let value = match template {
                JsonValue::Bool { value, .. } => JsonValue::Bool { value, range },
                JsonValue::Null { .. } => JsonValue::Null { range },
                _ => unreachable!("parse_keyword is only called with Bool/Null templates"),
            };
            Ok((value, end))
        } else {
            Err(JsonParseError {
                at: pos,
                reason: "invalid literal; expected true, false, or null",
            })
        }
    }

    fn parse_object(&self, pos: usize, depth: usize) -> Result<(JsonValue, usize), JsonParseError> {
        let start = pos;
        let mut cur = self.skip_ws(pos + 1);
        let mut members = Vec::new();

        if self.get(cur) == Some(b'}') {
            cur += 1;
            return Ok((
                JsonValue::Object {
                    members,
                    range: ByteRange { start, end: cur },
                },
                cur,
            ));
        }

        loop {
            if self.get(cur) != Some(b'"') {
                return Err(JsonParseError {
                    at: cur,
                    reason: "expected a quoted object key",
                });
            }
            let (key, key_range, after_key) = self.parse_string(cur)?;
            cur = self.skip_ws(after_key);
            if self.get(cur) != Some(b':') {
                return Err(JsonParseError {
                    at: cur,
                    reason: "expected ':' after object key",
                });
            }
            cur = self.skip_ws(cur + 1);
            let (value, after_value) = self.parse_value(cur, depth + 1)?;
            members.push(JsonMember {
                key,
                key_range,
                value,
            });
            cur = self.skip_ws(after_value);
            match self.get(cur) {
                Some(b',') => {
                    cur = self.skip_ws(cur + 1);
                    if self.get(cur) == Some(b'}') {
                        return Err(JsonParseError {
                            at: cur,
                            reason: "trailing comma before '}' is not allowed",
                        });
                    }
                }
                Some(b'}') => {
                    cur += 1;
                    break;
                }
                Some(_) => {
                    return Err(JsonParseError {
                        at: cur,
                        reason: "expected ',' or '}' in object",
                    });
                }
                None => {
                    return Err(JsonParseError {
                        at: cur,
                        reason: "unexpected end of input inside object",
                    });
                }
            }
        }

        Ok((
            JsonValue::Object {
                members,
                range: ByteRange { start, end: cur },
            },
            cur,
        ))
    }

    fn parse_array(&self, pos: usize, depth: usize) -> Result<(JsonValue, usize), JsonParseError> {
        let start = pos;
        let mut cur = self.skip_ws(pos + 1);
        let mut items = Vec::new();

        if self.get(cur) == Some(b']') {
            cur += 1;
            return Ok((
                JsonValue::Array {
                    items,
                    range: ByteRange { start, end: cur },
                },
                cur,
            ));
        }

        loop {
            let (value, after_value) = self.parse_value(cur, depth + 1)?;
            items.push(value);
            cur = self.skip_ws(after_value);
            match self.get(cur) {
                Some(b',') => {
                    cur = self.skip_ws(cur + 1);
                    if self.get(cur) == Some(b']') {
                        return Err(JsonParseError {
                            at: cur,
                            reason: "trailing comma before ']' is not allowed",
                        });
                    }
                }
                Some(b']') => {
                    cur += 1;
                    break;
                }
                Some(_) => {
                    return Err(JsonParseError {
                        at: cur,
                        reason: "expected ',' or ']' in array",
                    });
                }
                None => {
                    return Err(JsonParseError {
                        at: cur,
                        reason: "unexpected end of input inside array",
                    });
                }
            }
        }

        Ok((
            JsonValue::Array {
                items,
                range: ByteRange { start, end: cur },
            },
            cur,
        ))
    }

    /// Parses a quoted string starting at `pos` (which must be `"`).
    /// Delegates to `string_literal` — decoding escapes and surrogate
    /// pairs is a self-contained concern, split into its own file to keep
    /// this one under the 500 ELOC split threshold.
    fn parse_string(&self, pos: usize) -> Result<(String, ByteRange, usize), JsonParseError> {
        string_literal::parse_string(self.source, self.bytes, pos)
    }

    /// RFC 8259 `number = [ "-" ] int [ frac ] [ exp ]`: no leading `+`, no
    /// leading zeros (`01` is rejected — after consuming the lone `0`, the
    /// caller sees an unexpected `1` and rejects there), no bare `.5` or
    /// `5.` (a digit is required on both sides of the `.`).
    fn parse_number(&self, pos: usize) -> Result<(JsonValue, usize), JsonParseError> {
        let start = pos;
        let mut cur = pos;

        if self.get(cur) == Some(b'-') {
            cur += 1;
        }

        match self.get(cur) {
            Some(b'0') => cur += 1,
            Some(b'1'..=b'9') => {
                cur += 1;
                while matches!(self.get(cur), Some(b'0'..=b'9')) {
                    cur += 1;
                }
            }
            _ => {
                return Err(JsonParseError {
                    at: cur,
                    reason: "expected a digit",
                });
            }
        }

        if self.get(cur) == Some(b'.') {
            let frac_start = cur;
            cur += 1;
            if !matches!(self.get(cur), Some(b'0'..=b'9')) {
                return Err(JsonParseError {
                    at: frac_start,
                    reason: "expected a digit after '.'",
                });
            }
            while matches!(self.get(cur), Some(b'0'..=b'9')) {
                cur += 1;
            }
        }

        if matches!(self.get(cur), Some(b'e' | b'E')) {
            let exp_start = cur;
            cur += 1;
            if matches!(self.get(cur), Some(b'+' | b'-')) {
                cur += 1;
            }
            if !matches!(self.get(cur), Some(b'0'..=b'9')) {
                return Err(JsonParseError {
                    at: exp_start,
                    reason: "expected a digit in exponent",
                });
            }
            while matches!(self.get(cur), Some(b'0'..=b'9')) {
                cur += 1;
            }
        }

        Ok((
            JsonValue::Number {
                range: ByteRange { start, end: cur },
            },
            cur,
        ))
    }
}
