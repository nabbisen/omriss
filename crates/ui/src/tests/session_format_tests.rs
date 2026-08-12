//! RFC-054 J3/J7a: session wiring for non-Markdown formats.
//!
//! J3: `EditorSession::open_detected` and the `document_map_nodes()`
//! branch it feeds. J7a adds the read half of app wiring: `focus()`
//! succeeding for a real JSON-derived id (not just failing safely for
//! one, which is all J3 could prove before anything built on top of it)
//! and `focused_structured_content()`, the read-only accessor
//! `FocusedContentPane`'s JSON branch renders from. Markdown regression
//! coverage matters as much as the JSON-specific behavior throughout:
//! `open`/`open_with_profile` must produce byte-identical results to
//! before J3, since they delegate to `open_detected(..,
//! DocumentFormat::Markdown)` internally, and `focus()`'s Markdown path
//! is untouched by J7a.

use omriss_core::{DocumentFormat, FocusedContent, NodeId, StructureNodeKind, ValueKind};

use crate::{EditorSession, ViewMode, node_id_from_raw};

#[test]
fn open_and_open_detected_markdown_produce_the_same_session() {
    let source = "# A\n\n## A1\nbody\n\n# B\n";
    let via_open = EditorSession::open(source.to_string(), Some("doc.md".into())).unwrap();
    let via_detected = EditorSession::open_detected(
        source.to_string(),
        Some("doc.md".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Markdown,
    )
    .unwrap();

    assert_eq!(via_open.format(), DocumentFormat::Markdown);
    assert_eq!(via_detected.format(), DocumentFormat::Markdown);
    assert_eq!(via_open.source(), via_detected.source());
    assert_eq!(
        via_open.document_map_nodes(),
        via_detected.document_map_nodes()
    );
}

#[test]
fn format_reflects_what_open_detected_was_given() {
    let session = EditorSession::open_detected(
        r#"{"a": 1}"#.to_string(),
        Some("data.json".into()),
        crate::FileTextProfile::detect(r#"{"a": 1}"#, false),
        DocumentFormat::Json,
    )
    .unwrap();
    assert_eq!(session.format(), DocumentFormat::Json);
}

#[test]
fn valid_json_produces_a_real_document_map_tree() {
    let source = r#"{"name": "omriss", "version": 3}"#;
    let session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();

    let root = session.document_map_nodes();
    assert_eq!(root.children.len(), 2, "name and version");
    let titles: Vec<_> = root.children.iter().map(|c| c.title.as_str()).collect();
    assert!(titles.contains(&"name"));
    assert!(titles.contains(&"version"));
}

#[test]
fn json_node_capabilities_reflect_j5_editability() {
    // Non-null scalar editing landed in RFC-054 J5: can_edit_content is
    // now Allowed for a Value node, matching JsonAdapter's own capability
    // computation (crates/core/src/formats/json/projection.rs). Structural
    // actions remain Hidden regardless -- RFC-054 §13 Phase 4 is out of
    // scope for the whole handoff.
    let source = r#"{"a": 1}"#;
    let session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();

    let root = session.document_map_nodes();
    let a = &root.children[0];
    assert!(a.capabilities.can_select.is_allowed());
    assert!(a.capabilities.can_edit_content.is_allowed());
    assert!(a.capabilities.can_add_inside.is_hidden());
    assert!(a.capabilities.can_delete.is_hidden());
}

/// RFC-054 J3F-IMPL-002: the row icon must not assume every format is
/// Markdown. `DocumentMapNode.kind` is what a row-rendering layer derives
/// its glyph from instead of a hardcoded `#`.
#[test]
fn document_map_node_kind_reflects_json_structure_shape() {
    let source = r#"{"g": {"x": 1}, "l": [1], "v": 2}"#;
    let session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();

    let root = session.document_map_nodes();
    let by_title = |title: &str| root.children.iter().find(|c| c.title == title).unwrap();
    assert_eq!(by_title("g").kind, StructureNodeKind::Group);
    assert_eq!(by_title("l").kind, StructureNodeKind::List);
    assert_eq!(by_title("v").kind, StructureNodeKind::Value);
}

#[test]
fn document_map_node_kind_reflects_markdown_sections() {
    let source = "# A\n\n## A1\nbody\n";
    let session = EditorSession::open(source.to_string(), Some("doc.md".into())).unwrap();
    let root = session.document_map_nodes();
    let a = root.children.iter().find(|c| c.title == "A").unwrap();
    assert_eq!(a.kind, StructureNodeKind::MarkdownSection);
    let a1 = a.children.iter().find(|c| c.title == "A1").unwrap();
    assert_eq!(a1.kind, StructureNodeKind::MarkdownSection);
}

#[test]
fn malformed_json_produces_an_empty_document_map_and_preserves_source() {
    // Trailing comma: strict RFC 8259 rejects this (RFC-052 §14.1).
    let source = r#"{"a": 1,}"#;
    let session = EditorSession::open_detected(
        source.to_string(),
        Some("broken.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();

    // The file still opens -- Document::parse never fails on this text --
    // and the source is preserved exactly (RFC-052 §5.2: a failed
    // build_structure must not touch source, and none of the fallback
    // path touches `document` at all).
    assert_eq!(session.source(), source);

    // No synthetic structure is invented for what JsonAdapter could not
    // parse (RFC-052 §5.2): the PlainTextAdapter fallback's single node is
    // the root itself, which is never rendered as a row, so the map has no
    // children. "Offers plain file text" is the pre-existing, unconditional
    // `show_raw()`/"Show plain text" button, not a new Document Map row.
    let root = session.document_map_nodes();
    assert!(root.children.is_empty());
}

#[test]
fn empty_json_object_produces_a_document_map_with_no_children() {
    // Distinguishes "valid but empty" from "malformed" -- both produce zero
    // children here, but for different reasons; this pins the valid case.
    let source = "{}";
    let session = EditorSession::open_detected(
        source.to_string(),
        Some("empty.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();
    assert!(session.document_map_nodes().children.is_empty());
}

#[test]
fn nothing_is_selected_by_default_in_a_freshly_opened_json_session() {
    let source = r#"{"a": 1, "b": 2}"#;
    let session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();
    let root = session.document_map_nodes();
    assert!(root.children.iter().all(|c| !c.is_selected));
}

// RFC-054 J7a deliberately invalidates
// `focusing_a_json_derived_node_id_fails_safely_without_corrupting_session_state`
// (J3-IMPL-002's coverage, added at the J4 review). That test pinned a real
// safety property -- focus() must not crash or corrupt state when handed a
// JSON-space id -- but it proved it by relying on an accident: `focus()`
// validated every id against the Markdown outline `Document::parse` builds
// over JSON text, which a JSON-derived id was never a member of, so the
// call reliably failed. RFC-054 J7a replaces that accident with a real
// mechanism (`session::focus_bridge::validate_and_snapshot`, dispatching
// on `EditorSession::format` rather than always reading the Markdown
// outline), which makes a *valid* JSON-derived id succeed on purpose --
// the opposite of what the old test asserted.
//
// The safety property itself survives, restated for the new mechanism: an
// id that does not resolve in the *current* structure -- Markdown or
// JSON -- must still fail without corrupting view state. That is
// `focus_fails_safely_for_an_id_absent_from_the_current_structure` below.
// The property the old test could not have tested (nothing implemented it
// yet) is proved separately: `focusing_a_valid_json_value_node_succeeds_and_updates_view_mode`
// and `focused_structured_content_reads_the_focused_json_value`.

#[test]
fn focusing_a_valid_json_value_node_succeeds_and_updates_view_mode() {
    let source = r#"{"a": 1, "b": 2}"#;
    let mut session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();

    let root = session.document_map_nodes();
    let a_id = node_id_from_raw(root.children[0].id);

    let result = session.focus(a_id);

    assert!(result.is_ok(), "a real JSON-derived id must now resolve");
    assert_eq!(session.view_mode(), ViewMode::Focus(a_id));
}

#[test]
fn focus_fails_safely_for_an_id_absent_from_the_current_structure() {
    // The property the replaced test proved, restated for the new
    // mechanism: an id absent from the current structure -- not merely an
    // id from a different format's id space, which no longer applies now
    // that focus() dispatches on `format` instead of always reading the
    // Markdown outline -- must fail without touching view state.
    let source = r#"{"a": 1, "b": 2}"#;
    let mut session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();
    let view_before = session.view_mode();

    let result = session.focus(NodeId(u64::MAX));

    assert!(result.is_err(), "an id absent from the structure must fail");
    assert_eq!(
        session.view_mode(),
        view_before,
        "a failed focus must leave view state exactly as it was -- no crash, no corruption"
    );
    let root_after = session.document_map_nodes();
    assert_eq!(
        root_after.children.len(),
        2,
        "the Document Map is unaffected"
    );
}

#[test]
fn focused_structured_content_reads_the_focused_json_value() {
    let source = r#"{"a": "hello"}"#;
    let mut session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();
    let a_id = node_id_from_raw(session.document_map_nodes().children[0].id);
    session.focus(a_id).unwrap();

    let content = session
        .focused_structured_content()
        .expect("a focused Value node has real content");
    let FocusedContent::StructuredValue {
        value_kind,
        display_text,
        ..
    } = content
    else {
        panic!("expected StructuredValue");
    };
    assert_eq!(value_kind, ValueKind::Text);
    assert_eq!(display_text, "hello");
}

#[test]
fn focused_structured_content_reads_the_focused_json_group() {
    let source = r#"{"a": {"x": 1}}"#;
    let mut session = EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap();
    let a_id = node_id_from_raw(session.document_map_nodes().children[0].id);
    session.focus(a_id).unwrap();

    let content = session
        .focused_structured_content()
        .expect("a focused Group node has real content");
    let FocusedContent::StructuredGroup { child_count, .. } = content else {
        panic!("expected StructuredGroup");
    };
    assert_eq!(child_count, 1);
}

#[test]
fn focused_structured_content_is_none_for_markdown() {
    let source = "# A\nbody\n";
    let mut session = EditorSession::open(source.to_string(), Some("doc.md".into())).unwrap();
    let a_id = session.outline_items()[0].id;
    session.focus(a_id).unwrap();

    assert!(session.focused_structured_content().is_none());
}
