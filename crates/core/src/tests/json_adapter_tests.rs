//! RFC-054 J2/J5/J6: `JsonAdapter` build_structure/projection output —
//! tree shape, node identity, malformed-input rejection, and node
//! capabilities. J2 built strict RFC 8259 parsing and projection into
//! `DocumentStructure` (RFC-054 §5/§6); J5 made `Value` nodes (except
//! `null`) editable; J6 made `Group`/`List` nodes editable too (container
//! raw editing). `null` alone still refuses, permanently within RFC-054's
//! scope (§15 question 3 forbids type-changing it).
//!
//! Adapter-*method* behavior (`focused_content`/`validate_focused_edit`/
//! `apply_validated_edit`, byte preservation) lives in its own files, not
//! here: `json_scalar_editing_tests.rs` for `Value` nodes,
//! `json_container_editing_tests.rs` for `Group`/`List` nodes (RFC-054 J5
//! review, finding J5-IMPL-001 — split along this seam once the file
//! neared the 500-ELOC threshold rather than after crossing it). See
//! `unsupported_adapter_tests.rs` for `structure_command`'s uniform
//! refusal, unaffected by any slice (RFC-054 §13 Phase 4 is out of scope
//! for the whole handoff).

use crate::{
    Capability, CapabilityReason, DocumentFormat, DocumentFormatAdapter, DocumentRevision,
    JsonAdapter, StructureErrorKind, StructureNodeKind,
};

fn adapter() -> JsonAdapter {
    JsonAdapter
}

fn build(source: &str) -> crate::DocumentStructure {
    adapter()
        .build_structure(source, DocumentRevision::INITIAL)
        .expect("valid JSON must build")
}

fn find<'a>(structure: &'a crate::DocumentStructure, title: &str) -> &'a crate::StructureNode {
    structure
        .nodes
        .iter()
        .find(|n| n.title == title)
        .unwrap_or_else(|| panic!("no node titled {title:?}"))
}

// ── Basic projection ────────────────────────────────────────────────────────

#[test]
fn build_structure_reports_the_json_format() {
    let structure = build("{}");
    assert_eq!(structure.format, DocumentFormat::Json);
}

#[test]
fn root_object_projects_members_named_by_key() {
    let structure = build(r#"{"name": "omriss", "version": 3}"#);
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .expect("root present");
    assert_eq!(root.kind, StructureNodeKind::Group);
    assert_eq!(root.children.len(), 2);

    let name = find(&structure, "name");
    assert_eq!(name.kind, StructureNodeKind::Value);
    assert_eq!(name.parent_id, Some(structure.root_id));
    assert_eq!(name.depth, 1);

    let version = find(&structure, "version");
    assert_eq!(version.kind, StructureNodeKind::Value);
}

// ── root_id points directly at the top-level value (RFC-053 S5-IMPL-002) ────

#[test]
fn root_id_is_the_top_level_object_itself_not_a_synthetic_wrapper() {
    let structure = build(r#"{"a": 1}"#);
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .unwrap();
    assert_eq!(root.kind, StructureNodeKind::Group);
    assert_eq!(root.title, "");
}

#[test]
fn root_id_is_the_top_level_array_itself() {
    let structure = build("[1, 2, 3]");
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .unwrap();
    assert_eq!(root.kind, StructureNodeKind::List);
    assert_eq!(root.children.len(), 3);
}

#[test]
fn root_id_is_the_top_level_scalar_itself() {
    // RFC 8259 permits any value as the whole document, not only an
    // object or array.
    let structure = build("42");
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .unwrap();
    assert_eq!(root.kind, StructureNodeKind::Value);
    assert_eq!(structure.nodes.len(), 1);
}

// ── Empty containers ─────────────────────────────────────────────────────────

#[test]
fn empty_object_has_no_children() {
    let structure = build("{}");
    assert_eq!(structure.nodes.len(), 1);
    assert!(structure.nodes[0].children.is_empty());
}

#[test]
fn empty_array_has_no_children() {
    let structure = build("[]");
    assert_eq!(structure.nodes.len(), 1);
    assert_eq!(structure.nodes[0].kind, StructureNodeKind::List);
    assert!(structure.nodes[0].children.is_empty());
}

// ── Nesting shapes ───────────────────────────────────────────────────────────

#[test]
fn nested_object_builds_a_tree_with_correct_depth() {
    let structure = build(r#"{"a": {"b": {"c": 1}}}"#);
    let a = find(&structure, "a");
    assert_eq!(a.kind, StructureNodeKind::Group);
    assert_eq!(a.depth, 1);
    let b = find(&structure, "b");
    assert_eq!(b.depth, 2);
    assert_eq!(b.parent_id, Some(a.id));
    let c = find(&structure, "c");
    assert_eq!(c.depth, 3);
    assert_eq!(c.kind, StructureNodeKind::Value);
}

#[test]
fn nested_array_builds_a_tree_with_correct_depth() {
    // Every level here is titled "Item 1" (RFC-054 §5: an array item is
    // named by ordinal regardless of whether its value is a scalar or a
    // container), so titles alone cannot disambiguate depth -- walk down
    // by id instead of using `find`.
    let structure = build("[[[1]]]");
    let node = |id: crate::NodeId| structure.nodes.iter().find(|n| n.id == id).unwrap();

    let root = node(structure.root_id);
    assert_eq!(root.depth, 0);
    assert_eq!(root.kind, StructureNodeKind::List);

    let level1 = node(root.children[0]);
    assert_eq!(level1.title, "Item 1");
    assert_eq!(level1.depth, 1);
    assert_eq!(level1.kind, StructureNodeKind::List);

    let level2 = node(level1.children[0]);
    assert_eq!(level2.depth, 2);
    assert_eq!(level2.kind, StructureNodeKind::List);

    let level3 = node(level2.children[0]);
    assert_eq!(level3.depth, 3);
    assert_eq!(level3.kind, StructureNodeKind::Value);
}

#[test]
fn object_in_array_produces_group_items_named_by_ordinal() {
    let structure = build(r#"[{"x": 1}, {"y": 2}]"#);
    let item1 = find(&structure, "Item 1");
    assert_eq!(item1.kind, StructureNodeKind::Group);
    let item2 = find(&structure, "Item 2");
    assert_eq!(item2.kind, StructureNodeKind::Group);
    let x = find(&structure, "x");
    assert_eq!(x.parent_id, Some(item1.id));
    let y = find(&structure, "y");
    assert_eq!(y.parent_id, Some(item2.id));
}

#[test]
fn array_in_object_produces_a_list_child_named_by_key() {
    let structure = build(r#"{"tags": ["a", "b"]}"#);
    let tags = find(&structure, "tags");
    assert_eq!(tags.kind, StructureNodeKind::List);
    assert_eq!(tags.children.len(), 2);
    let item1 = find(&structure, "Item 1");
    assert_eq!(item1.parent_id, Some(tags.id));
}

// ── Scalar types ─────────────────────────────────────────────────────────────

#[test]
fn all_scalar_types_are_represented_as_value_nodes() {
    let structure = build(r#"{"s": "text", "n": 3.5, "t": true, "f": false, "nul": null}"#);
    for key in ["s", "n", "t", "f", "nul"] {
        let node = find(&structure, key);
        assert_eq!(node.kind, StructureNodeKind::Value, "key {key}");
        assert!(node.children.is_empty());
    }
}

// ── Duplicate keys (RFC-054 §15.2 — never merged) ───────────────────────────

#[test]
fn duplicate_keys_are_displayed_separately_never_merged() {
    let structure = build(r#"{"a": 1, "a": 2}"#);
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .unwrap();
    assert_eq!(
        root.children.len(),
        2,
        "both duplicate members must survive"
    );

    let a_nodes: Vec<_> = structure.nodes.iter().filter(|n| n.title == "a").collect();
    assert_eq!(a_nodes.len(), 2);
    assert_ne!(
        a_nodes[0].id, a_nodes[1].id,
        "duplicate keys get distinct identity from their ordinal position"
    );
    // Each keeps its own source range -- editing one must not touch the
    // other later (J5); prove the ranges are already disjoint here.
    let r0 = a_nodes[0].source_range.unwrap();
    let r1 = a_nodes[1].source_range.unwrap();
    assert!(r0.end <= r1.start || r1.end <= r0.start);
}

// ── Escapes and Unicode (decoded through object-key titles) ────────────────

#[test]
fn escaped_characters_in_keys_decode_correctly() {
    let structure = build(r#"{"line\nbreak": 1, "quote\"mark": 2, "back\\slash": 3}"#);
    assert!(structure.nodes.iter().any(|n| n.title == "line\nbreak"));
    assert!(structure.nodes.iter().any(|n| n.title == "quote\"mark"));
    assert!(structure.nodes.iter().any(|n| n.title == "back\\slash"));
}

#[test]
fn unicode_escape_and_literal_unicode_keys_decode_correctly() {
    let structure = build(r#"{"café": 1, "日本語": 2}"#);
    assert!(structure.nodes.iter().any(|n| n.title == "café"));
    assert!(structure.nodes.iter().any(|n| n.title == "日本語"));
}

#[test]
fn surrogate_pair_escape_decodes_to_the_astral_character() {
    // U+1F389 PARTY POPPER, encoded as a UTF-16 surrogate pair.
    let structure = build(r#"{"🎉": 1}"#);
    assert!(structure.nodes.iter().any(|n| n.title == "🎉"));
}

// ── Deep nesting within the supported limit ─────────────────────────────────

#[test]
fn nesting_at_a_generous_depth_still_succeeds() {
    let depth = 100;
    let mut source = "[".repeat(depth);
    source.push('1');
    source.push_str(&"]".repeat(depth));
    let structure = build(&source);
    let deepest = structure
        .nodes
        .iter()
        .max_by_key(|n| n.depth)
        .expect("at least one node");
    assert_eq!(deepest.depth, depth);
}

#[test]
fn nesting_far_beyond_the_limit_is_rejected_without_crashing() {
    // Must not panic/abort (stack overflow aborts the process rather than
    // unwinding, so the meaningful assertion is that this line is reached
    // at all) -- mirrors RFC-053 §18's stack-safety proof for Markdown,
    // except here the scanner is genuinely recursive, so a depth limit
    // (not an already-iterative algorithm) is the actual mitigation.
    let depth = 10_000;
    let mut source = "[".repeat(depth);
    source.push('1');
    source.push_str(&"]".repeat(depth));
    let result = adapter().build_structure(&source, DocumentRevision::INITIAL);
    assert!(result.is_err(), "must reject, not overflow the stack");
    assert_eq!(result.unwrap_err().kind, StructureErrorKind::InvalidSyntax);
}

// ── Malformed input: typed error, never a panic ─────────────────────────────

fn assert_rejected(source: &str, why: &str) {
    let result = adapter().build_structure(source, DocumentRevision::INITIAL);
    assert!(result.is_err(), "expected rejection ({why}): {source:?}");
    assert_eq!(
        result.unwrap_err().kind,
        StructureErrorKind::InvalidSyntax,
        "wrong error kind for: {why}"
    );
}

#[test]
fn empty_source_is_rejected() {
    assert_rejected("", "empty input is not a JSON value");
}

#[test]
fn trailing_comma_in_object_is_rejected() {
    assert_rejected(r#"{"a": 1,}"#, "trailing comma");
}

#[test]
fn trailing_comma_in_array_is_rejected() {
    assert_rejected("[1, 2,]", "trailing comma");
}

#[test]
fn comments_are_rejected() {
    assert_rejected(
        "{\"a\": 1 /* comment */}",
        "block comment (JSONC, not strict JSON)",
    );
    assert_rejected("{\"a\": 1} // trailing comment", "line comment");
}

#[test]
fn unquoted_keys_are_rejected() {
    assert_rejected("{a: 1}", "unquoted key");
}

#[test]
fn single_quoted_strings_are_rejected() {
    assert_rejected("{'a': 1}", "single-quoted key");
}

#[test]
fn leading_zero_numbers_are_rejected() {
    assert_rejected("01", "leading zero");
    assert_rejected("[01]", "leading zero inside array");
}

#[test]
fn leading_plus_numbers_are_rejected() {
    assert_rejected("+1", "leading plus is not valid JSON");
}

#[test]
fn nan_and_infinity_are_rejected() {
    assert_rejected("NaN", "NaN is not valid JSON");
    assert_rejected("Infinity", "Infinity is not valid JSON");
    assert_rejected("-Infinity", "-Infinity is not valid JSON");
}

#[test]
fn bare_decimal_point_is_rejected() {
    assert_rejected(".5", "no leading digit before '.'");
    assert_rejected("5.", "no digit after '.'");
}

#[test]
fn unterminated_string_is_rejected() {
    assert_rejected(r#"{"a": "unterminated}"#, "missing closing quote");
}

#[test]
fn trailing_content_after_the_top_level_value_is_rejected() {
    assert_rejected("{} {}", "two top-level values");
    assert_rejected("1 garbage", "junk after a valid value");
}

#[test]
fn unescaped_control_character_in_string_is_rejected() {
    assert_rejected("{\"a\": \"line1\nline2\"}", "raw newline inside a string");
}

// ── RFC-053 §13.3: identity determinism and stability ───────────────────────

#[test]
fn rebuild_determinism_same_source_produces_same_ids() {
    let source = r#"{"a": {"b": 1}, "c": [1, 2]}"#;
    let first = build(source);
    let second = build(source);
    let first_ids: Vec<_> = first.nodes.iter().map(|n| n.id).collect();
    let second_ids: Vec<_> = second.nodes.iter().map(|n| n.id).collect();
    assert_eq!(first_ids, second_ids);
    assert_eq!(first.root_id, second.root_id);
}

#[test]
fn identity_is_stable_under_an_unrelated_value_edit() {
    let before = build(r#"{"a": 1, "b": 2}"#);
    let b_before = find(&before, "b").id;

    // Edit "a"'s value in place -- same key, same position, different
    // content. Node identity is purely position-derived, so "b" must be
    // unaffected.
    let after = build(r#"{"a": 999, "b": 2}"#);
    let b_after = find(&after, "b").id;

    assert_eq!(b_before, b_after);
}

#[test]
fn identity_changes_when_tree_position_changes() {
    // Contrast with the previous test: inserting a new member before "b"
    // shifts "b"'s ordinal, so its id is *not* expected to survive --
    // exactly like Markdown's ids, which are stable across body edits but
    // not across structural ones.
    let before = build(r#"{"a": 1, "b": 2}"#);
    let b_before = find(&before, "b").id;

    let after = build(r#"{"z": 0, "a": 1, "b": 2}"#);
    let b_after = find(&after, "b").id;

    assert_ne!(b_before, b_after);
}

// ── Capabilities ─────────────────────────────────────────────────────────────

#[test]
fn non_null_value_node_capabilities_are_editable_as_of_j5() {
    let structure = build(r#"{"a": 1}"#);
    let a = find(&structure, "a");
    assert!(a.capabilities.can_select.is_allowed());
    assert!(a.capabilities.can_edit_content.is_allowed());
    assert!(a.capabilities.can_show_plain_text.is_hidden());
    assert!(a.capabilities.can_add_inside.is_hidden());
    assert!(a.capabilities.can_rename.is_hidden());
    assert!(a.capabilities.can_delete.is_hidden());
}

#[test]
fn null_value_node_can_edit_content_stays_disabled() {
    // RFC-054 §15 question 3: a value edit may change a value, never its
    // kind -- there is nothing else to edit about a null literal, so it
    // does not become editable in J5 the way other scalars do.
    let structure = build(r#"{"a": null}"#);
    let a = find(&structure, "a");
    assert_eq!(
        a.capabilities.can_edit_content,
        Capability::Disabled {
            reason: CapabilityReason::ReadOnlyFormat
        }
    );
}

#[test]
fn group_and_list_node_capabilities_are_editable_as_of_j6() {
    let structure = build(r#"{"g": {"x": 1}, "l": [1]}"#);
    let g = find(&structure, "g");
    assert_eq!(g.kind, StructureNodeKind::Group);
    assert!(g.capabilities.can_edit_content.is_allowed());
    assert!(g.capabilities.can_show_plain_text.is_allowed());
    let l = find(&structure, "l");
    assert_eq!(l.kind, StructureNodeKind::List);
    assert!(l.capabilities.can_edit_content.is_allowed());
    assert!(l.capabilities.can_show_plain_text.is_allowed());
}

#[test]
fn root_node_has_hidden_capabilities_like_every_other_format() {
    let structure = build(r#"{"a": 1}"#);
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .unwrap();
    assert!(root.capabilities.can_select.is_hidden());
    assert!(root.capabilities.can_edit_content.is_hidden());
}
