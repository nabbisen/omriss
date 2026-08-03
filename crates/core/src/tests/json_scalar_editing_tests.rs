//! RFC-054 J5: `JsonAdapter`'s scalar value editing —
//! `focused_content`/`validate_focused_edit`/`apply_validated_edit` for
//! `Value` nodes whose literal is not `null` (RFC-054 §8). The
//! requirement this slice is judged on (RFC-054 §12.2, the acceptance
//! checklist's "BYTE PRESERVATION" line): editing one value must change
//! only that value's own bytes — indentation, key order, and line
//! endings elsewhere in the file must be byte-identical before and after.

use crate::{
    Document, DocumentFormatAdapter, DocumentRevision, EditDescription, FocusedContent,
    JsonAdapter, StructureErrorKind, ValueKind,
};

fn adapter() -> JsonAdapter {
    JsonAdapter
}

fn structure(source: &str) -> crate::DocumentStructure {
    adapter()
        .build_structure(source, DocumentRevision::INITIAL)
        .expect("valid JSON must build")
}

fn find_id(structure: &crate::DocumentStructure, title: &str) -> crate::NodeId {
    structure
        .nodes
        .iter()
        .find(|n| n.title == title)
        .unwrap_or_else(|| panic!("no node titled {title:?}"))
        .id
}

fn doc(source: &str) -> Document {
    Document::parse(source.to_string()).expect("parse")
}

// ── focused_content: scalar kinds ───────────────────────────────────────────

#[test]
fn focused_content_reports_a_text_value_decoded() {
    let source = r#"{"a": "hello \"world\""}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(source, &s, a_id)
        .expect("Value node");
    let FocusedContent::StructuredValue {
        value_kind,
        display_text,
        editable_text,
        ..
    } = content
    else {
        panic!("expected StructuredValue");
    };
    assert_eq!(value_kind, ValueKind::Text);
    assert_eq!(display_text, "hello \"world\"");
    assert_eq!(editable_text, "hello \"world\"");
}

#[test]
fn focused_content_reports_a_number_value() {
    let source = r#"{"a": 3.5}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(source, &s, a_id)
        .expect("Value node");
    let FocusedContent::StructuredValue {
        value_kind,
        display_text,
        editable_text,
        ..
    } = content
    else {
        panic!("expected StructuredValue");
    };
    assert_eq!(value_kind, ValueKind::Number);
    assert_eq!(display_text, "3.5");
    assert_eq!(editable_text, "3.5");
}

#[test]
fn focused_content_reports_a_bool_value() {
    let source = r#"{"a": true}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(source, &s, a_id)
        .expect("Value node");
    let FocusedContent::StructuredValue {
        value_kind,
        display_text,
        ..
    } = content
    else {
        panic!("expected StructuredValue");
    };
    assert_eq!(value_kind, ValueKind::OnOff);
    assert_eq!(display_text, "true");
}

#[test]
fn focused_content_reports_a_null_value_as_no_value_with_empty_text() {
    let source = r#"{"a": null}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(source, &s, a_id)
        .expect("Value node");
    let FocusedContent::StructuredValue {
        value_kind,
        display_text,
        editable_text,
        ..
    } = content
    else {
        panic!("expected StructuredValue");
    };
    assert_eq!(value_kind, ValueKind::NoValue);
    assert_eq!(display_text, "");
    assert_eq!(editable_text, "");
}

// ── focused_content: containers ─────────────────────────────────────────────

#[test]
fn focused_content_reports_a_group_with_child_count_and_preview() {
    let source = r#"{"a": {"x": 1, "y": 2}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(source, &s, a_id)
        .expect("Group node");
    let FocusedContent::StructuredGroup {
        child_count,
        summary,
        raw_text_available,
        ..
    } = content
    else {
        panic!("expected StructuredGroup");
    };
    assert_eq!(child_count, 2);
    assert_eq!(summary, r#"{"x": 1, "y": 2}"#);
    assert!(raw_text_available);
}

#[test]
fn focused_content_reports_a_list_as_structured_group_too() {
    // FocusedContent has one container variant, StructuredGroup, used for
    // both Group and List kinds (RFC-053 §8 does not define a separate
    // "StructuredList").
    let source = r#"{"a": [1, 2, 3]}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(source, &s, a_id)
        .expect("List node");
    let FocusedContent::StructuredGroup { child_count, .. } = content else {
        panic!("expected StructuredGroup");
    };
    assert_eq!(child_count, 3);
}

#[test]
fn focused_content_summary_is_truncated_for_a_long_container() {
    let long_key = "x".repeat(100);
    let source = format!(r#"{{"a": {{"{long_key}": 1}}}}"#);
    let s = structure(&source);
    let a_id = find_id(&s, "a");
    let content = adapter()
        .focused_content(&source, &s, a_id)
        .expect("Group node");
    let FocusedContent::StructuredGroup { summary, .. } = content else {
        panic!("expected StructuredGroup");
    };
    assert!(
        summary.ends_with('\u{2026}'),
        "expected an ellipsis: {summary:?}"
    );
    assert!(
        summary.chars().count() <= 81,
        "expected <= 80 chars + ellipsis: {summary:?}"
    );
}

// ── validate_focused_edit: encoding and validation per kind ────────────────

#[test]
fn validate_focused_edit_encodes_a_text_draft_with_escaping() {
    let source = r#"{"a": "old"}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let edit = adapter()
        .validate_focused_edit(source, &s, a_id, "hello \"world\"")
        .expect("valid text draft");
    assert_eq!(edit.replacement_text, r#""hello \"world\"""#);
    assert_eq!(edit.description, EditDescription::FocusedContentReplacement);
}

#[test]
fn validate_focused_edit_accepts_a_valid_number_draft() {
    let source = r#"{"a": 1}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let edit = adapter()
        .validate_focused_edit(source, &s, a_id, "42")
        .expect("valid number draft");
    assert_eq!(edit.replacement_text, "42");
}

#[test]
fn validate_focused_edit_rejects_an_invalid_number_draft() {
    let source = r#"{"a": 1}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "abc")
        .expect_err("not a number");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_rejects_a_leading_plus_number_draft() {
    // RFC-054 §8.2: "No leading plus sign" -- reuses the scanner's strict
    // grammar, so this is rejected exactly like the J2 parser rejects it.
    let source = r#"{"a": 1}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "+5")
        .expect_err("leading plus is not valid JSON");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_accepts_true_and_false_drafts() {
    let source = r#"{"a": false}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let edit = adapter()
        .validate_focused_edit(source, &s, a_id, "true")
        .expect("valid bool draft");
    assert_eq!(edit.replacement_text, "true");
}

#[test]
fn validate_focused_edit_rejects_an_invalid_onoff_draft() {
    let source = r#"{"a": true}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "yes")
        .expect_err("not true/false");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_uses_the_structures_own_revision_as_base() {
    let source = r#"{"a": 1}"#;
    let s = adapter()
        .build_structure(source, DocumentRevision(7))
        .unwrap();
    let a_id = find_id(&s, "a");
    let edit = adapter()
        .validate_focused_edit(source, &s, a_id, "2")
        .unwrap();
    assert_eq!(edit.base_revision, DocumentRevision(7));
}

// ── apply_validated_edit + byte preservation (RFC-054 §12.2, the slice's
//    central requirement) ────────────────────────────────────────────────────

#[test]
fn editing_a_string_value_preserves_all_unrelated_bytes() {
    let source = r#"{
  "name": "old",
  "version": 1
}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let name_id = find_id(&s, "name");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, name_id, "new")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    let expected = r#"{
  "name": "new",
  "version": 1
}"#;
    assert_eq!(document.source(), expected);
}

#[test]
fn editing_a_number_preserves_all_unrelated_bytes() {
    let source = r#"{"count": 1, "label": "items"}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let count_id = find_id(&s, "count");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, count_id, "42")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(document.source(), r#"{"count": 42, "label": "items"}"#);
}

#[test]
fn editing_a_boolean_preserves_all_unrelated_bytes() {
    let source = r#"{"enabled": false, "name": "x"}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let enabled_id = find_id(&s, "enabled");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, enabled_id, "true")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(document.source(), r#"{"enabled": true, "name": "x"}"#);
}

#[test]
fn editing_a_deeply_nested_value_preserves_all_unrelated_bytes() {
    let source = r#"{"a": {"b": {"c": "old", "d": "keep"}}, "e": "keep too"}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let c_id = find_id(&s, "c");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, c_id, "new")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(
        document.source(),
        r#"{"a": {"b": {"c": "new", "d": "keep"}}, "e": "keep too"}"#
    );
}

#[test]
fn crlf_line_endings_elsewhere_are_preserved() {
    let source = "{\r\n  \"a\": \"old\",\r\n  \"b\": 2\r\n}";
    let mut document = doc(source);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, "new")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(
        document.source(),
        "{\r\n  \"a\": \"new\",\r\n  \"b\": 2\r\n}"
    );
}

#[test]
fn only_the_edited_values_own_bytes_change() {
    // Byte-level before/after, character by character, not just a whole-
    // string comparison: proves the edit did not shift or touch anything
    // outside the target value's own range.
    let before = r#"{"a": "x", "b": "y"}"#;
    let mut document = doc(before);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");
    let a_range = s
        .nodes
        .iter()
        .find(|n| n.id == a_id)
        .unwrap()
        .source_range
        .unwrap();

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, "zz")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();
    let after = document.source();

    assert_eq!(&after[..a_range.start], &before[..a_range.start]);
    let suffix_before = &before[a_range.end..];
    assert_eq!(&after[after.len() - suffix_before.len()..], suffix_before);
    assert_eq!(after, r#"{"a": "zz", "b": "y"}"#);
}

#[test]
fn undo_restores_the_exact_original_file_after_a_scalar_edit() {
    let source = r#"{"a": "old", "b": [1, 2, {"c": true}]}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, "new")
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();
    assert_ne!(document.source(), source);

    document.undo().unwrap();
    assert_eq!(document.source(), source);
}

#[test]
fn revision_increments_by_exactly_one_after_a_scalar_edit() {
    let source = r#"{"a": 1}"#;
    let mut document = doc(source);
    let start_revision = document.revision();
    let s = structure(document.source());
    let a_id = find_id(&s, "a");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, "2")
        .unwrap();
    let applied = adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(applied.result.old_revision, start_revision);
    assert_eq!(document.revision(), applied.result.new_revision);
    assert_eq!(document.revision().0, start_revision.0 + 1);
}

#[test]
fn a_stale_base_revision_is_rejected_before_any_mutation() {
    let source = r#"{"a": 1, "b": 2}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");
    let b_id = find_id(&s, "b");

    // Commit an edit to "a", advancing the document's revision...
    let edit_a = adapter()
        .validate_focused_edit(document.source(), &s, a_id, "9")
        .unwrap();
    adapter()
        .apply_validated_edit(&mut document, edit_a)
        .unwrap();

    // ...then try to commit an edit to "b" prepared against the stale
    // structure `s` (still revision 0).
    let edit_b = adapter()
        .validate_focused_edit(source, &s, b_id, "9")
        .unwrap();
    let err = adapter()
        .apply_validated_edit(&mut document, edit_b)
        .expect_err("base_revision is stale");
    assert_eq!(err.kind, StructureErrorKind::UnsafeRange);
    // Rejected before mutation: only "a"'s edit landed.
    assert_eq!(document.source(), r#"{"a": 9, "b": 2}"#);
}
