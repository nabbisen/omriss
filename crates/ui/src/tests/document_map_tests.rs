//! Tests for DocumentMapNode generation and capability computation (RFC-049).

use crate::EditorSession;
use crate::interface::document_map::{CapabilityReason, DraftState, MapCapability};

// ── DraftState ────────────────────────────────────────────────────────────────

#[test]
fn draft_state_clean_does_not_block_navigation() {
    assert!(!DraftState::Clean.blocks_navigation());
}

#[test]
fn draft_state_valid_uncommitted_does_not_block_navigation() {
    assert!(!DraftState::ValidUncommitted.blocks_navigation());
}

#[test]
fn draft_state_invalid_uncommitted_blocks_navigation() {
    assert!(DraftState::InvalidUncommitted.blocks_navigation());
}

// ── CapabilityReason catalog keys ─────────────────────────────────────────────

#[test]
fn every_capability_reason_has_a_non_empty_catalog_key() {
    let reasons = [
        CapabilityReason::RootNode,
        CapabilityReason::NoSibling,
        CapabilityReason::NoParent,
        CapabilityReason::ReadOnlyFormat,
        CapabilityReason::ExperimentalFormat,
        CapabilityReason::UnsafePreservation,
        CapabilityReason::UnsupportedForFormat,
        CapabilityReason::ExternalChangeConflict,
    ];
    for r in &reasons {
        assert!(
            !r.catalog_key().is_empty(),
            "CapabilityReason::{r:?} has an empty catalog key"
        );
        assert!(
            r.catalog_key().starts_with("capability.disabled."),
            "expected 'capability.disabled.' prefix, got '{}'",
            r.catalog_key()
        );
    }
}

// ── DocumentMapNode tree ──────────────────────────────────────────────────────

fn session_with(markdown: &str) -> EditorSession {
    EditorSession::open(markdown.to_string(), None).expect("parse")
}

#[test]
fn document_map_nodes_returns_root_with_children() {
    let session = session_with("# Alpha\nbody\n\n# Beta\nbody\n");
    let root = session.document_map_nodes();
    assert_eq!(root.children.len(), 2);
    assert_eq!(root.children[0].title, "Alpha");
    assert_eq!(root.children[1].title, "Beta");
}

#[test]
fn document_map_root_has_hidden_capabilities() {
    let session = session_with("# A\n\n# B\n");
    let root = session.document_map_nodes();
    // Root node capabilities are all hidden.
    assert!(root.capabilities.can_delete.is_hidden());
    assert!(root.capabilities.can_move_up.is_hidden());
    assert!(root.capabilities.can_rename.is_hidden());
    // Root can still show plain text.
    assert!(root.capabilities.can_show_plain_text.is_allowed());
}

#[test]
fn first_section_cannot_move_up() {
    let session = session_with("# A\n\n# B\n");
    let root = session.document_map_nodes();
    let first = &root.children[0];
    assert!(
        matches!(first.capabilities.can_move_up, MapCapability::Disabled(_)),
        "first section should not be movable up"
    );
}

#[test]
fn last_section_cannot_move_down() {
    let session = session_with("# A\n\n# B\n");
    let root = session.document_map_nodes();
    let last = &root.children[1];
    assert!(
        matches!(last.capabilities.can_move_down, MapCapability::Disabled(_)),
        "last section should not be movable down"
    );
}

#[test]
fn middle_section_can_move_up_and_down() {
    let session = session_with("# A\n\n# B\n\n# C\n");
    let root = session.document_map_nodes();
    let middle = &root.children[1];
    assert!(middle.capabilities.can_move_up.is_allowed());
    assert!(middle.capabilities.can_move_down.is_allowed());
}

#[test]
fn top_level_section_cannot_move_out_one_level() {
    let session = session_with("# A\n\n# B\n");
    let root = session.document_map_nodes();
    let section = &root.children[0];
    assert!(
        matches!(
            section.capabilities.can_move_out_one_level,
            MapCapability::Disabled(CapabilityReason::NoParent)
        ),
        "top-level section should report NoParent for move_out_one_level"
    );
}

#[test]
fn nested_section_can_move_out_one_level() {
    let session = session_with("# A\n\n## A1\nbody\n\n## A2\nbody\n\n# B\n");
    let root = session.document_map_nodes();
    // A1 is a child of A.
    let a_node = &root.children[0];
    assert_eq!(a_node.title, "A");
    let a1 = &a_node.children[0];
    assert_eq!(a1.title, "A1");
    assert!(
        a1.capabilities.can_move_out_one_level.is_allowed(),
        "nested section should be able to move out one level"
    );
}

#[test]
fn first_child_cannot_join_with_previous() {
    let session = session_with("# A\n\n# B\n");
    let root = session.document_map_nodes();
    let first = &root.children[0];
    assert!(
        matches!(
            first.capabilities.can_join_with_previous,
            MapCapability::Disabled(_)
        ),
        "first section should not be able to join with previous"
    );
}

#[test]
fn second_section_can_join_with_previous() {
    let session = session_with("# A\n\n# B\n");
    let root = session.document_map_nodes();
    let second = &root.children[1];
    assert!(
        second.capabilities.can_join_with_previous.is_allowed(),
        "second section should be able to join with previous"
    );
}

#[test]
fn third_empty_top_level_section_can_join_with_previous_empty_section() {
    let session = session_with("# A.\n# B.\n# C.\n");
    let root = session.document_map_nodes();
    assert_eq!(root.children.len(), 3);
    assert_eq!(root.children[0].title, "A.");
    assert_eq!(root.children[1].title, "B.");
    assert_eq!(root.children[2].title, "C.");
    assert!(
        root.children[2]
            .capabilities
            .can_join_with_previous
            .is_allowed(),
        "third top-level section should be able to merge into the second"
    );
}

#[test]
fn section_cannot_join_when_previous_sibling_has_children() {
    let session = session_with("# A\n\n## A.1\nchild\n\n# B\nbody\n");
    let root = session.document_map_nodes();
    let second = &root.children[1];
    assert_eq!(
        second.capabilities.can_join_with_previous,
        MapCapability::Disabled(CapabilityReason::UnsafePreservation)
    );
}

#[test]
fn selected_node_is_flagged_when_focused() {
    let mut session = session_with("# A\nbody\n\n# B\nbody\n");
    let root = session.document_map_nodes();
    let id_b = root.children[1].id;

    // Not selected yet.
    assert!(!root.children[1].is_selected);

    // Focus B.
    let _ = session.focus(omriss_core::NodeId(id_b));
    let root2 = session.document_map_nodes();
    assert!(
        root2.children[1].is_selected,
        "focused node should be selected"
    );
    assert!(
        !root2.children[0].is_selected,
        "unfocused node should not be selected"
    );
}

#[test]
fn document_without_headings_returns_root_with_no_children() {
    let session = session_with("No headings here.\n");
    let root = session.document_map_nodes();
    assert!(root.children.is_empty());
}

#[test]
fn descendant_count_is_correct() {
    let session = session_with("# A\n\n## A1\n\n## A2\n\n# B\n");
    let root = session.document_map_nodes();
    // A has 2 children; A's descendant_count = 2.
    assert_eq!(root.children[0].descendant_count(), 2);
    // B has no children.
    assert_eq!(root.children[1].descendant_count(), 0);
}
