//! RFC-053 §6 S2: structure vocabulary and Markdown capability computation,
//! asserted at core level ahead of `MarkdownAdapter` (S4). Mirrors the
//! scenarios in `omriss_ui::tests::document_map_tests`.

use crate::formats::structure::{
    Capability, CapabilityReason, DocumentStructure, NodeCapabilities, StructureNode,
    StructureNodeKind, markdown_node_capabilities,
};
use crate::{Document, DocumentFormat, DocumentRevision, NodeId};

fn doc(source: &str) -> Document {
    Document::parse(source.to_string()).expect("parse")
}

// ── Vocabulary types ─────────────────────────────────────────────────────────

#[test]
fn document_structure_types_are_constructible() {
    let structure = DocumentStructure {
        format: DocumentFormat::Markdown,
        root_id: NodeId(0),
        nodes: vec![StructureNode {
            id: NodeId(1),
            parent_id: Some(NodeId(0)),
            title: "A".to_string(),
            kind: StructureNodeKind::MarkdownSection,
            depth: 1,
            source_range: None,
            editable_range: None,
            children: Vec::new(),
            capabilities: NodeCapabilities::hidden(),
        }],
        revision: DocumentRevision(0),
    };
    assert_eq!(structure.nodes.len(), 1);
    assert_eq!(structure.nodes[0].kind, StructureNodeKind::MarkdownSection);
}

#[test]
fn root_capabilities_are_hidden_except_show_plain_text() {
    let caps = NodeCapabilities::hidden();
    assert!(caps.can_select.is_hidden());
    assert!(caps.can_edit_content.is_hidden());
    assert!(caps.can_delete.is_hidden());
    assert!(caps.can_move_up.is_hidden());
    assert!(caps.can_show_plain_text.is_allowed());
}

// ── Markdown capability computation ──────────────────────────────────────────

#[test]
fn first_child_cannot_move_up() {
    let document = doc("# A\n\n# B\n");
    let outline = document.outline();
    let first = outline.root().children[0];
    let caps = markdown_node_capabilities(outline, first);
    assert!(matches!(
        caps.can_move_up,
        Capability::Disabled {
            reason: CapabilityReason::NoSibling
        }
    ));
}

#[test]
fn last_child_cannot_move_down() {
    let document = doc("# A\n\n# B\n");
    let outline = document.outline();
    let last = outline.root().children[1];
    let caps = markdown_node_capabilities(outline, last);
    assert!(matches!(
        caps.can_move_down,
        Capability::Disabled {
            reason: CapabilityReason::NoSibling
        }
    ));
}

#[test]
fn middle_child_can_move_up_and_down() {
    let document = doc("# A\n\n# B\n\n# C\n");
    let outline = document.outline();
    let middle = outline.root().children[1];
    let caps = markdown_node_capabilities(outline, middle);
    assert!(caps.can_move_up.is_allowed());
    assert!(caps.can_move_down.is_allowed());
}

#[test]
fn only_child_cannot_move_up_down_or_join_or_move_out() {
    let document = doc("# A\nbody\n");
    let outline = document.outline();
    let only = outline.root().children[0];
    let caps = markdown_node_capabilities(outline, only);
    assert!(matches!(
        caps.can_move_up,
        Capability::Disabled {
            reason: CapabilityReason::NoSibling
        }
    ));
    assert!(matches!(
        caps.can_move_down,
        Capability::Disabled {
            reason: CapabilityReason::NoSibling
        }
    ));
    assert!(matches!(
        caps.can_join_with_previous,
        Capability::Disabled {
            reason: CapabilityReason::NoSibling
        }
    ));
    assert!(matches!(
        caps.can_move_out_one_level,
        Capability::Disabled {
            reason: CapabilityReason::NoParent
        }
    ));
}

#[test]
fn top_level_section_cannot_move_out_one_level() {
    let document = doc("# A\n\n# B\n");
    let outline = document.outline();
    let section = outline.root().children[0];
    let caps = markdown_node_capabilities(outline, section);
    assert!(matches!(
        caps.can_move_out_one_level,
        Capability::Disabled {
            reason: CapabilityReason::NoParent
        }
    ));
}

#[test]
fn deep_node_can_move_out_one_level() {
    let document = doc("# A\n\n## A1\nbody\n\n## A2\nbody\n\n# B\n");
    let outline = document.outline();
    let a = outline.root().children[0];
    let a1 = outline.node(a).unwrap().children[0];
    let caps = markdown_node_capabilities(outline, a1);
    assert!(caps.can_move_out_one_level.is_allowed());
}

#[test]
fn second_child_can_join_with_previous_when_previous_has_no_children() {
    let document = doc("# A\n\n# B\n");
    let outline = document.outline();
    let second = outline.root().children[1];
    let caps = markdown_node_capabilities(outline, second);
    assert!(caps.can_join_with_previous.is_allowed());
}

#[test]
fn cannot_join_when_previous_sibling_has_children() {
    let document = doc("# A\n\n## A.1\nchild\n\n# B\nbody\n");
    let outline = document.outline();
    let second = outline.root().children[1];
    let caps = markdown_node_capabilities(outline, second);
    assert_eq!(
        caps.can_join_with_previous,
        Capability::Disabled {
            reason: CapabilityReason::UnsafePreservation
        }
    );
}

#[test]
fn demote_blocked_at_h6_depth_limit() {
    let document = doc("###### A\n\n###### B\n");
    let outline = document.outline();
    let b = outline.root().children[1];
    let caps = markdown_node_capabilities(outline, b);
    assert!(matches!(
        caps.can_move_inside_previous,
        Capability::Disabled {
            reason: CapabilityReason::NoSibling
        }
    ));
}

#[test]
fn non_root_section_can_always_be_selected_edited_added_to_renamed_and_deleted() {
    let document = doc("# A\n\n# B\n");
    let outline = document.outline();
    let first = outline.root().children[0];
    let caps = markdown_node_capabilities(outline, first);
    assert!(caps.can_select.is_allowed());
    assert!(caps.can_edit_content.is_allowed());
    assert!(caps.can_add_inside.is_allowed());
    assert!(caps.can_add_after.is_allowed());
    assert!(caps.can_rename.is_allowed());
    assert!(caps.can_delete.is_allowed());
    assert!(caps.can_show_plain_text.is_allowed());
}
