//! Scalar value classification, decoding, and encoding for focused editing
//! (RFC-054 §8, J5).
//!
//! A `Value` node's `source_range`/`editable_range` (set by `projection`)
//! is always exactly one scalar literal's own byte span, already known
//! well-formed — it came from a successful `scanner::parse`. Classifying
//! and decoding it here never re-validates JSON grammar; that already
//! happened when `build_structure` ran.

use crate::formats::focused_content::ValueKind;
use crate::range::ByteRange;

use super::scanner::{self, JsonValue};
use super::string_literal;

/// The four RFC-054 §8 shapes a JSON scalar literal can take, before
/// translation into the format-neutral `ValueKind` `FocusedContent` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScalarKind {
    Text,
    Number,
    OnOff,
    NoValue,
}

impl ScalarKind {
    pub(super) fn to_value_kind(self) -> ValueKind {
        match self {
            Self::Text => ValueKind::Text,
            Self::Number => ValueKind::Number,
            Self::OnOff => ValueKind::OnOff,
            Self::NoValue => ValueKind::NoValue,
        }
    }
}

/// Classifies the scalar literal at `range` by its leading byte. Falls
/// back to `Number` on any unexpected or missing leading byte rather than
/// panicking or failing — a caller passing a `range` that does not
/// actually start a scalar is a contract violation this function does not
/// try to diagnose; the subsequent number-grammar check simply rejects the
/// content, which is a safe failure rather than an indexing panic.
pub(super) fn classify(source: &str, range: ByteRange) -> ScalarKind {
    match source.as_bytes().get(range.start) {
        Some(b'"') => ScalarKind::Text,
        Some(b't') | Some(b'f') => ScalarKind::OnOff,
        Some(b'n') => ScalarKind::NoValue,
        _ => ScalarKind::Number,
    }
}

/// Decodes a JSON string literal at `range` (must start with `"`, as
/// guaranteed by `classify` returning `Text`) into its unescaped text.
pub(super) fn decode_string(source: &str, range: ByteRange) -> String {
    string_literal::parse_string(source, source.as_bytes(), range.start)
        .map(|(decoded, ..)| decoded)
        .unwrap_or_default()
}

/// Encodes raw user text into a JSON string literal — quotes plus escapes
/// — per RFC-054 §8.1: "The UI may allow plain text entry and the adapter
/// will JSON-escape it." The inverse of `decode_string`.
pub(super) fn encode_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Validates `draft` as a JSON number literal (RFC-054 §8.2: "must be a
/// valid JSON number. No leading plus sign, no NaN, no Infinity"). Reuses
/// the scanner's own strict number grammar rather than re-deriving it: a
/// bare number is trivially a valid whole top-level JSON document, so
/// `scanner::parse` already enforces every rule §8.2 asks for.
pub(super) fn is_valid_number(draft: &str) -> bool {
    matches!(scanner::parse(draft), Ok(JsonValue::Number { .. }))
}

/// Validates `draft` as RFC-054 §8.3's on/off literal: exactly `true` or
/// `false`, nothing else.
pub(super) fn is_valid_bool(draft: &str) -> bool {
    draft == "true" || draft == "false"
}
