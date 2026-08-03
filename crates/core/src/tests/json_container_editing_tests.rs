//! RFC-054 J6: `JsonAdapter`'s container raw focused editing —
//! `validate_focused_edit`/`apply_validated_edit` for `Group`/`List`
//! nodes (RFC-054 §7.4, §8.5). Split out of `json_adapter_tests.rs`
//! (J5 review finding J5-IMPL-001) as its own file, mirroring
//! `json_scalar_editing_tests.rs`'s shape for `Value` nodes.
//!
//! Same central requirement as J5, restated for containers (RFC-054
//! §12.2): editing one container's raw text must change only that
//! container's own bytes. Same conservative rule §8.5 states explicitly:
//! a replacement must parse as JSON and keep the container's own kind —
//! an object stays an object, an array stays an array. `focused_content`
//! for `Group`/`List` was already real as of J5 (read-only summary); its
//! tests stay in `json_scalar_editing_tests.rs`, unmoved by this file.

use crate::{
    Document, DocumentFormatAdapter, DocumentRevision, EditDescription, StructureErrorKind,
};

fn adapter() -> crate::JsonAdapter {
    crate::JsonAdapter
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

// ── validate_focused_edit: accepted replacements ────────────────────────────

#[test]
fn validate_focused_edit_accepts_a_replacement_object_for_the_root_group() {
    // The root of `{"a": 1}` is itself a Group -- container editing must
    // work there too, not only on a nested container. `validate_focused_edit`
    // gates on `node.kind` directly, independent of `NodeCapabilities`
    // (which hide the root from the UI for an unrelated reason: it is
    // never a selectable row), so this is a real, exercised path.
    let source = r#"{"a": 1}"#;
    let s = structure(source);
    let edit = adapter()
        .validate_focused_edit(source, &s, s.root_id, r#"{"a": 2, "b": 3}"#)
        .expect("object replacement for a Group is valid");
    assert_eq!(edit.replacement_text, r#"{"a": 2, "b": 3}"#);
    assert_eq!(edit.description, EditDescription::FocusedContentReplacement);
}

#[test]
fn validate_focused_edit_accepts_a_replacement_array_for_the_root_list() {
    let source = "[1, 2]";
    let s = structure(source);
    let edit = adapter()
        .validate_focused_edit(source, &s, s.root_id, "[1, 2, 3]")
        .expect("array replacement for a List is valid");
    assert_eq!(edit.replacement_text, "[1, 2, 3]");
}

#[test]
fn validate_focused_edit_accepts_replacing_an_empty_object_with_a_populated_one() {
    let source = r#"{"a": {}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let edit = adapter()
        .validate_focused_edit(source, &s, a_id, r#"{"x": 1}"#)
        .expect("an empty container is still editable");
    assert_eq!(edit.replacement_text, r#"{"x": 1}"#);
}

// ── validate_focused_edit: rejected replacements (RFC-054 §12.3) ───────────

#[test]
fn validate_focused_edit_rejects_malformed_raw_text_for_a_group() {
    let source = r#"{"a": {"x": 1}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "{not json}")
        .expect_err("malformed raw text must be rejected");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_rejects_an_empty_draft_for_a_group() {
    let source = r#"{"a": {"x": 1}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "")
        .expect_err("empty text is not a JSON value");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_rejects_trailing_garbage_after_a_valid_container_replacement() {
    // Reuses the scanner's own strict "exactly one value" grammar (the
    // same rule `trailing_content_after_the_top_level_value_is_rejected`
    // pins for `build_structure`), not a separately re-derived one.
    let source = r#"{"a": {"x": 1}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, r#"{"x": 1} garbage"#)
        .expect_err("trailing content after the value must be rejected");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_rejects_an_array_replacement_for_a_group() {
    // RFC-054 §8.5: conservative first implementation preserves container
    // type -- an object cannot become an array via a raw-text edit, even
    // though `[1, 2]` is itself perfectly valid JSON.
    let source = r#"{"a": {"x": 1}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "[1, 2]")
        .expect_err("type-changing a Group to a List is out of scope, RFC-054 §8.5");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_rejects_an_object_replacement_for_a_list() {
    let source = r#"{"a": [1, 2]}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, r#"{"x": 1}"#)
        .expect_err("type-changing a List to a Group is out of scope, RFC-054 §8.5");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_rejects_a_scalar_replacement_for_a_group() {
    let source = r#"{"a": {"x": 1}}"#;
    let s = structure(source);
    let a_id = find_id(&s, "a");
    let err = adapter()
        .validate_focused_edit(source, &s, a_id, "42")
        .expect_err("a scalar is not appropriate for a Group node, RFC-054 §8.5");
    assert_eq!(err.kind, StructureErrorKind::InvalidSyntax);
}

#[test]
fn validate_focused_edit_uses_the_structures_own_revision_as_base() {
    let source = r#"{"a": {"x": 1}}"#;
    let s = adapter()
        .build_structure(source, DocumentRevision(7))
        .unwrap();
    let a_id = find_id(&s, "a");
    let edit = adapter()
        .validate_focused_edit(source, &s, a_id, r#"{"x": 2}"#)
        .unwrap();
    assert_eq!(edit.base_revision, DocumentRevision(7));
}

// ── apply_validated_edit + byte preservation (RFC-054 §12.2) ───────────────

#[test]
fn editing_a_group_raw_text_preserves_all_unrelated_bytes() {
    let source = r#"{
  "settings": {"x": 1},
  "version": 1
}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let settings_id = find_id(&s, "settings");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, settings_id, r#"{"x": 2, "y": 3}"#)
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    let expected = r#"{
  "settings": {"x": 2, "y": 3},
  "version": 1
}"#;
    assert_eq!(document.source(), expected);
}

#[test]
fn editing_a_list_raw_text_preserves_all_unrelated_bytes() {
    let source = r#"{"tags": ["a"], "label": "items"}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let tags_id = find_id(&s, "tags");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, tags_id, r#"["a", "b", "c"]"#)
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(
        document.source(),
        r#"{"tags": ["a", "b", "c"], "label": "items"}"#
    );
}

#[test]
fn editing_a_deeply_nested_container_preserves_all_unrelated_bytes() {
    let source = r#"{"a": {"b": {"c": {"x": 1}, "d": "keep"}}, "e": "keep too"}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let c_id = find_id(&s, "c");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, c_id, r#"{"x": 2, "y": 3}"#)
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(
        document.source(),
        r#"{"a": {"b": {"c": {"x": 2, "y": 3}, "d": "keep"}}, "e": "keep too"}"#
    );
}

#[test]
fn crlf_line_endings_elsewhere_are_preserved_for_a_container_edit() {
    let source = "{\r\n  \"a\": {\"x\": 1},\r\n  \"b\": 2\r\n}";
    let mut document = doc(source);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, r#"{"x": 2}"#)
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(
        document.source(),
        "{\r\n  \"a\": {\"x\": 2},\r\n  \"b\": 2\r\n}"
    );
}

#[test]
fn only_the_edited_containers_own_bytes_change() {
    // Byte-level before/after, mirroring
    // json_scalar_editing_tests::only_the_edited_values_own_bytes_change:
    // proves the edit did not shift or touch anything outside the target
    // container's own range.
    let before = r#"{"a": {"x": 1}, "b": "y"}"#;
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
        .validate_focused_edit(document.source(), &s, a_id, r#"{"x": 9, "z": 9}"#)
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();
    let after = document.source();

    assert_eq!(&after[..a_range.start], &before[..a_range.start]);
    let suffix_before = &before[a_range.end..];
    assert_eq!(&after[after.len() - suffix_before.len()..], suffix_before);
    assert_eq!(after, r#"{"a": {"x": 9, "z": 9}, "b": "y"}"#);
}

#[test]
fn undo_restores_the_exact_original_file_after_a_container_edit() {
    let source = r#"{"a": {"x": 1}, "b": [1, 2]}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, r#"{"x": 2, "y": 3}"#)
        .unwrap();
    adapter().apply_validated_edit(&mut document, edit).unwrap();
    assert_ne!(document.source(), source);

    document.undo().unwrap();
    assert_eq!(document.source(), source);
}

#[test]
fn revision_increments_by_exactly_one_after_a_container_edit() {
    let source = r#"{"a": {"x": 1}}"#;
    let mut document = doc(source);
    let start_revision = document.revision();
    let s = structure(document.source());
    let a_id = find_id(&s, "a");

    let edit = adapter()
        .validate_focused_edit(document.source(), &s, a_id, r#"{"x": 2}"#)
        .unwrap();
    let applied = adapter().apply_validated_edit(&mut document, edit).unwrap();

    assert_eq!(applied.result.old_revision, start_revision);
    assert_eq!(document.revision(), applied.result.new_revision);
    assert_eq!(document.revision().0, start_revision.0 + 1);
}

#[test]
fn a_stale_base_revision_is_rejected_before_any_mutation_for_a_container_edit() {
    let source = r#"{"a": {"x": 1}, "b": {"y": 2}}"#;
    let mut document = doc(source);
    let s = structure(document.source());
    let a_id = find_id(&s, "a");
    let b_id = find_id(&s, "b");

    // Commit an edit to "a", advancing the document's revision...
    let edit_a = adapter()
        .validate_focused_edit(document.source(), &s, a_id, r#"{"x": 9}"#)
        .unwrap();
    adapter()
        .apply_validated_edit(&mut document, edit_a)
        .unwrap();

    // ...then try to commit an edit to "b" prepared against the stale
    // structure `s` (still revision 0).
    let edit_b = adapter()
        .validate_focused_edit(source, &s, b_id, r#"{"y": 9}"#)
        .unwrap();
    let err = adapter()
        .apply_validated_edit(&mut document, edit_b)
        .expect_err("base_revision is stale");
    assert_eq!(err.kind, StructureErrorKind::UnsafeRange);
    // Rejected before mutation: only "a"'s edit landed.
    assert_eq!(document.source(), r#"{"a": {"x": 9}, "b": {"y": 2}}"#);
}
