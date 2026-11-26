//! RFC-084 §10.1 (D5): editing a JSON string value changes only the bytes of
//! the characters the user changed. Every escape they did not touch keeps the
//! spelling the file gave it, and undo restores the original exactly.
//!
//! A draft here is what a text area delivers: every line break is `\n`.

use crate::{
    Document, DocumentFormatAdapter, DocumentRevision, FocusedContent, JsonAdapter, normalize_to_lf,
};

fn structure(source: &str) -> crate::DocumentStructure {
    JsonAdapter
        .build_structure(source, DocumentRevision::INITIAL)
        .expect("valid JSON")
}

fn find_id(s: &crate::DocumentStructure, title: &str) -> crate::NodeId {
    s.nodes.iter().find(|n| n.title == title).unwrap().id
}

/// Edits the value titled `title` in `source` to `draft`; returns the new source.
fn edit(source: &str, title: &str, draft: &str) -> String {
    let mut document = Document::parse(source.to_string()).unwrap();
    let s = structure(source);
    let id = find_id(&s, title);
    let validated = JsonAdapter
        .validate_focused_edit(source, &s, id, draft)
        .expect("a string draft is always valid");
    JsonAdapter
        .apply_validated_edit(&mut document, validated)
        .unwrap();
    document.source().to_string()
}

/// The value's text as the editor would deliver it with `x` typed at the end.
fn with_x_typed(source: &str, title: &str) -> String {
    let s = structure(source);
    let id = find_id(&s, title);
    let FocusedContent::StructuredValue { editable_text, .. } =
        JsonAdapter.focused_content(source, &s, id).unwrap()
    else {
        panic!("a value");
    };
    format!("{}x", normalize_to_lf(&editable_text))
}

// ── Task 028's six values, byte-exact ───────────────────────────────────────

const TASK_028: &str = r#"{
  "nl": "line1\nline2",
  "crlf": "a\r\nb",
  "tab": "col1\tcol2",
  "eacute": "café",
  "literal": "naïve café 日本語",
  "slash": "a\/b"
}"#;

#[test]
fn task_028_one_character_typed_changes_one_character() {
    // (key, the value's line after typing `x` at the end)
    let cases = [
        ("nl", r#"  "nl": "line1\nline2x","#),
        ("crlf", r#"  "crlf": "a\r\nbx","#),
        ("tab", r#"  "tab": "col1\tcol2x","#),
        ("eacute", r#"  "eacute": "caféx","#),
        ("literal", r#"  "literal": "naïve café 日本語x","#),
        ("slash", r#"  "slash": "a\/bx""#),
    ];
    for (key, line) in cases {
        let after = edit(TASK_028, key, &with_x_typed(TASK_028, key));
        let before_line = TASK_028
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("\"{key}\"")))
            .unwrap();
        let expected = TASK_028.replace(before_line, line);
        assert_eq!(after, expected, "{key}");
    }
}

#[test]
fn an_unchanged_draft_leaves_every_byte() {
    for key in ["nl", "crlf", "tab", "eacute", "literal", "slash"] {
        let s = structure(TASK_028);
        let id = find_id(&s, key);
        let FocusedContent::StructuredValue { editable_text, .. } =
            JsonAdapter.focused_content(TASK_028, &s, id).unwrap()
        else {
            panic!()
        };
        // As the text area delivers it, and as the decoded text is.
        assert_eq!(
            edit(TASK_028, key, &normalize_to_lf(&editable_text)),
            TASK_028
        );
        assert_eq!(edit(TASK_028, key, &editable_text), TASK_028);
    }
}

#[test]
fn a_typed_line_break_takes_the_strings_style() {
    // `\n` string, `\r\n` string, a string with none.
    let doc = r#"{"lf": "a\nb", "crlf": "a\r\nb", "none": "ab"}"#;
    assert_eq!(
        edit(doc, "lf", "a\nb\nc"),
        r#"{"lf": "a\nb\nc", "crlf": "a\r\nb", "none": "ab"}"#
    );
    assert_eq!(
        edit(doc, "crlf", "a\nb\nc"),
        r#"{"lf": "a\nb", "crlf": "a\r\nb\r\nc", "none": "ab"}"#
    );
    assert_eq!(
        edit(doc, "none", "a\nb"),
        r#"{"lf": "a\nb", "crlf": "a\r\nb", "none": "a\nb"}"#
    );
}

#[test]
fn changing_a_character_next_to_an_escape_keeps_the_escape() {
    let doc = r#"{"k": "café \/ 😀"}"#;
    assert_eq!(
        edit(doc, "k", "cafe\u{e9} / \u{1F600}"),
        r#"{"k": "cafeé \/ 😀"}"#
    );
    assert_eq!(
        edit(doc, "k", "caf\u{e9} / \u{1F600}!"),
        r#"{"k": "café \/ 😀!"}"#
    );
}

#[test]
fn a_replaced_value_is_written_whole() {
    assert_eq!(
        edit(r#"{"a": "old", "b": 1}"#, "a", "new \"q\""),
        r#"{"a": "new \"q\"", "b": 1}"#
    );
    assert_eq!(edit(r#"{"a": ""}"#, "a", "x"), r#"{"a": "x"}"#);
    assert_eq!(edit(r#"{"a": "xyz"}"#, "a", ""), r#"{"a": ""}"#);
}

// ── the property (RFC-084 §10.4) ────────────────────────────────────────────

/// A character of a string value, how the file spells it, and what a text area
/// shows for it.
type Token = (&'static str, &'static str);

const TOKENS: &[Token] = &[
    ("a", "a"),
    ("\\u00e9", "é"),
    ("é", "é"),
    ("\\/", "/"),
    ("\\n", "\n"),
    ("\\r\\n", "\n"),
    ("\\r", "\n"),
    ("\\t", "\t"),
    ("\\\"", "\""),
    ("\\\\", "\\"),
    ("\\ud83d\\ude00", "😀"),
    ("😀", "😀"),
    ("\\u0001", "\u{1}"),
];

/// The strings the property runs over: each mixes escapes the encoder would not
/// write with ones it would, and plain characters.
fn strings() -> Vec<Vec<usize>> {
    // Indices into TOKENS.
    vec![
        vec![0, 1, 3, 0],
        vec![0, 4, 0, 5, 0],
        vec![6, 0, 6],
        vec![2, 12, 7, 8, 9],
        vec![10, 0, 11, 0],
        vec![1, 1, 1],
        // Lone CRs in a run, and beside LFs.
        vec![6, 6, 0, 6, 6],
        vec![0, 6, 0, 4, 0, 6, 0, 4],
        vec![],
    ]
}

fn spell(tokens: &[usize]) -> String {
    tokens.iter().map(|&t| TOKENS[t].0).collect()
}
fn shown(tokens: &[usize]) -> String {
    tokens.iter().map(|&t| TOKENS[t].1).collect()
}

/// The property, for one string: every span of it × every snippet.
fn property(tokens: &[usize]) -> usize {
    let source = format!("{{\"k\": \"{}\", \"z\": 1}}", spell(tokens));
    let before = Document::parse(source.clone()).unwrap();
    let mut checked = 0;
    for i in 0..=tokens.len() {
        for j in i..=tokens.len() {
            for snippet in ["", "x", "é", "\n", "\"", "\\", "😀", "a\nb", "\t"] {
                let draft = format!("{}{}{}", shown(&tokens[..i]), snippet, shown(&tokens[j..]));
                let ctx = format!("tokens {tokens:?} span {i}..{j} snippet {snippet:?}");
                let after = edit(&source, "k", &draft);

                // The text area shows exactly the draft.
                let s = structure(&after);
                let FocusedContent::StructuredValue { editable_text, .. } = JsonAdapter
                    .focused_content(&after, &s, find_id(&s, "k"))
                    .unwrap()
                else {
                    panic!()
                };
                assert_eq!(normalize_to_lf(&editable_text), draft, "{ctx}");

                // Nothing outside the literal changed.
                let open = "{\"k\": \"";
                let close = "\", \"z\": 1}";
                assert!(
                    after.starts_with(open) && after.ends_with(close),
                    "{ctx}: {after}"
                );
                let literal = &after[open.len()..after.len() - close.len()];

                // The head and the tail the text area did not change keep the
                // spelling the file gave them. (Which characters those are is
                // decided by comparing the texts, as in the line-ending
                // property: typing an `é` beside an `é` is the same text either
                // way.) Every token shows exactly one character.
                let original: Vec<char> = shown(tokens).chars().collect();
                let wanted: Vec<char> = draft.chars().collect();
                if original == wanted {
                    assert_eq!(after, source, "{ctx}: an unchanged draft changed bytes");
                    checked += 1;
                    continue;
                }
                let head = original
                    .iter()
                    .zip(&wanted)
                    .take_while(|(x, y)| x == y)
                    .count();
                let tail = original[head..]
                    .iter()
                    .rev()
                    .zip(wanted[head..].iter().rev())
                    .take_while(|(x, y)| x == y)
                    .count();
                // Only a lone `\r` right before the edit may be rewritten (it
                // would fuse with a `\n` into a `\r\n`), and so each lone `\r`
                // before it, for a run.
                let lone_cr_run = tokens[..head]
                    .iter()
                    .rev()
                    .take_while(|&&t| TOKENS[t].0 == "\\r")
                    .count();
                assert!(
                    (0..=lone_cr_run).any(|k| literal.starts_with(&spell(&tokens[..head - k]))),
                    "{ctx}: the unchanged head was respelled: {literal}"
                );
                assert!(
                    literal.ends_with(&spell(&tokens[tokens.len() - tail..])),
                    "{ctx}: the unchanged tail was respelled: {literal}"
                );

                // Undo restores the original, byte for byte.
                let mut document = Document::parse(source.clone()).unwrap();
                let s = structure(&source);
                let validated = JsonAdapter
                    .validate_focused_edit(&source, &s, find_id(&s, "k"), &draft)
                    .unwrap();
                JsonAdapter
                    .apply_validated_edit(&mut document, validated)
                    .unwrap();
                document.undo().unwrap();
                assert_eq!(document.source(), source, "{ctx}: undo");
                checked += 1;
            }
        }
    }
    drop(before);
    checked
}

#[test]
fn every_span_of_every_string_changes_only_its_own_bytes() {
    let mut total = 0;
    for tokens in strings() {
        total += property(&tokens);
    }
    assert!(total > 800, "the property ran {total} edits");
}
