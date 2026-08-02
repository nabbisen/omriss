//! RFC-054 J3: session wiring for non-Markdown formats.
//!
//! `EditorSession::open_detected` and the `document_map_nodes()` branch it
//! feeds are the new surface this slice adds. Markdown regression coverage
//! matters as much as the JSON-specific behavior: `open`/`open_with_profile`
//! must produce byte-identical results to before this slice, since they
//! delegate to `open_detected(.., DocumentFormat::Markdown)` internally now.

use omriss_core::DocumentFormat;

use crate::EditorSession;

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
fn json_node_capabilities_match_the_read_only_j2_design() {
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
    assert!(!a.capabilities.can_edit_content.is_allowed());
    assert!(a.capabilities.can_add_inside.is_hidden());
    assert!(a.capabilities.can_delete.is_hidden());
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
