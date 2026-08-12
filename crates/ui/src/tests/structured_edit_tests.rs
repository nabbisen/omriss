//! RFC-054 J7b: `EditorSession`'s write half for a focused JSON node —
//! `structured_draft_differs`/`structured_draft_state` (RFC-053 §9.3's
//! `DraftState`, generalized past Markdown's `current_snapshot().body`),
//! `validate_structured_draft`, `commit_structured_draft`, and
//! `focused_structured_raw_text` (the container raw-editor's real text,
//! distinct from `focused_content()`'s truncated `summary`).
//!
//! J7a's read-only accessors (`focus`, `focused_structured_content`) are
//! `session_format_tests.rs`'s territory, unmoved by this file.

use omriss_core::{DocumentFormat, DraftState};

use crate::{EditorSession, node_id_from_raw};

fn open_json(source: &str) -> EditorSession {
    EditorSession::open_detected(
        source.to_string(),
        Some("pkg.json".into()),
        crate::FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap()
}

fn focus_child(session: &mut EditorSession, title: &str) {
    let root = session.document_map_nodes();
    let child = root
        .children
        .iter()
        .find(|c| c.title == title)
        .unwrap_or_else(|| panic!("no child titled {title:?}"));
    session.focus(node_id_from_raw(child.id)).unwrap();
}

// ── structured_draft_differs / structured_draft_state ──────────────────────

#[test]
fn draft_state_is_clean_when_draft_matches_the_committed_value() {
    let mut session = open_json(r#"{"a": "hello"}"#);
    focus_child(&mut session, "a");
    assert!(!session.structured_draft_differs("hello"));
    assert_eq!(session.structured_draft_state("hello"), DraftState::Clean);
}

#[test]
fn draft_state_is_valid_uncommitted_for_a_changed_valid_draft() {
    let mut session = open_json(r#"{"a": 1}"#);
    focus_child(&mut session, "a");
    assert!(session.structured_draft_differs("42"));
    assert_eq!(
        session.structured_draft_state("42"),
        DraftState::ValidUncommitted
    );
}

#[test]
fn draft_state_is_invalid_uncommitted_for_a_changed_invalid_draft() {
    let mut session = open_json(r#"{"a": 1}"#);
    focus_child(&mut session, "a");
    assert_eq!(
        session.structured_draft_state("not a number"),
        DraftState::InvalidUncommitted
    );
}

#[test]
fn draft_state_for_a_container_compares_against_its_full_raw_text() {
    let mut session = open_json(r#"{"a": {"x": 1}}"#);
    focus_child(&mut session, "a");
    let raw = session.focused_structured_raw_text().unwrap();
    assert_eq!(raw, r#"{"x": 1}"#);
    assert_eq!(session.structured_draft_state(&raw), DraftState::Clean);
    assert_eq!(
        session.structured_draft_state(r#"{"x": 2}"#),
        DraftState::ValidUncommitted
    );
    assert_eq!(
        session.structured_draft_state("[1, 2]"),
        DraftState::InvalidUncommitted,
        "type-changing a Group to a List is invalid, RFC-054 §8.5"
    );
}

#[test]
fn draft_state_is_clean_when_nothing_is_focused() {
    let session = open_json(r#"{"a": 1}"#);
    assert_eq!(
        session.structured_draft_state("anything"),
        DraftState::Clean
    );
}

// ── focused_structured_raw_text ─────────────────────────────────────────────

#[test]
fn focused_structured_raw_text_is_none_for_a_scalar_value() {
    let mut session = open_json(r#"{"a": 1}"#);
    focus_child(&mut session, "a");
    assert!(session.focused_structured_raw_text().is_none());
}

#[test]
fn focused_structured_raw_text_is_none_for_markdown() {
    let mut session =
        EditorSession::open("# A\nbody\n".to_string(), Some("doc.md".into())).unwrap();
    let a_id = session.outline_items()[0].id;
    session.focus(a_id).unwrap();
    assert!(session.focused_structured_raw_text().is_none());
}

// ── validate_structured_draft ───────────────────────────────────────────────

#[test]
fn validate_structured_draft_accepts_a_valid_text_draft() {
    let mut session = open_json(r#"{"a": "old"}"#);
    focus_child(&mut session, "a");
    assert!(session.validate_structured_draft("new").is_ok());
}

#[test]
fn validate_structured_draft_rejects_an_invalid_number_draft() {
    let mut session = open_json(r#"{"a": 1}"#);
    focus_child(&mut session, "a");
    assert!(session.validate_structured_draft("+5").is_err());
}

// ── commit_structured_draft + byte preservation ─────────────────────────────

#[test]
fn commit_structured_draft_applies_a_valid_edit_and_preserves_unrelated_bytes() {
    let source = r#"{"a": "old", "b": 2}"#;
    let mut session = open_json(source);
    focus_child(&mut session, "a");

    let result = session.commit_structured_draft("new").unwrap();
    assert_eq!(session.source(), r#"{"a": "new", "b": 2}"#);
    assert_eq!(result.old_revision.0 + 1, result.new_revision.0);
}

#[test]
fn commit_structured_draft_rejects_an_invalid_draft_without_mutating() {
    let source = r#"{"a": 1, "b": 2}"#;
    let mut session = open_json(source);
    focus_child(&mut session, "a");

    let err = session.commit_structured_draft("not a number");
    assert!(err.is_err());
    assert_eq!(session.source(), source, "an invalid draft must not mutate");
}

#[test]
fn commit_structured_draft_commits_a_container_raw_text_edit() {
    let source = r#"{"a": {"x": 1}}"#;
    let mut session = open_json(source);
    focus_child(&mut session, "a");

    session
        .commit_structured_draft(r#"{"x": 2, "y": 3}"#)
        .unwrap();
    assert_eq!(session.source(), r#"{"a": {"x": 2, "y": 3}}"#);
}

#[test]
fn undo_restores_the_exact_original_file_after_a_structured_commit() {
    let source = r#"{"a": "old", "b": [1, 2]}"#;
    let mut session = open_json(source);
    focus_child(&mut session, "a");
    session.commit_structured_draft("new").unwrap();
    assert_ne!(session.source(), source);

    session.undo().unwrap();
    assert_eq!(session.source(), source);
}

#[test]
fn undo_does_not_lose_focus_on_the_json_node_being_edited() {
    // Found live, not by a unit test that only checked `document.source()`
    // after undo (the test above): `prune_dead_history` used to check
    // `document.outline()` unconditionally, and no JSON-space id is ever a
    // member of the Markdown outline `Document::parse` builds over JSON
    // text -- so every undo()/redo() silently kicked the view back to the
    // outline, discarding focus on the node the user was just editing.
    let mut session = open_json(r#"{"a": "old", "b": 2}"#);
    let a_id = session.document_map_nodes().children[0].id;
    let a_id = node_id_from_raw(a_id);
    session.focus(a_id).unwrap();
    session.commit_structured_draft("new").unwrap();

    session.undo().unwrap();

    assert_eq!(
        session.view_mode(),
        crate::ViewMode::Focus(a_id),
        "undo must not prune a still-valid JSON focus target"
    );
}

#[test]
fn redo_does_not_lose_focus_on_the_json_node_being_edited() {
    let mut session = open_json(r#"{"a": "old", "b": 2}"#);
    let a_id = session.document_map_nodes().children[0].id;
    let a_id = node_id_from_raw(a_id);
    session.focus(a_id).unwrap();
    session.commit_structured_draft("new").unwrap();
    session.undo().unwrap();

    session.redo().unwrap();

    assert_eq!(
        session.view_mode(),
        crate::ViewMode::Focus(a_id),
        "redo must not prune a still-valid JSON focus target"
    );
}

#[test]
fn the_focused_node_stays_focused_after_a_successful_commit() {
    // A value edit changes content, never tree position (RFC-054 §13.3),
    // so the same NodeId remains valid and focused after commit --
    // confirmed by successfully reading content through it again.
    let mut session = open_json(r#"{"a": "old", "b": 2}"#);
    focus_child(&mut session, "a");
    session.commit_structured_draft("new").unwrap();

    let content = session
        .focused_structured_content()
        .expect("still focused on a valid node");
    let omriss_core::FocusedContent::StructuredValue { display_text, .. } = content else {
        panic!("expected StructuredValue");
    };
    assert_eq!(display_text, "new");
}
