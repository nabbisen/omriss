//! RFC-053 §13.3 / S4a: `MarkdownAdapter::build_structure` identity
//! requirements — rebuild determinism and focus survival across an
//! unrelated edit.

use crate::{
    Document, DocumentFormatAdapter, MarkdownAdapter, ReplaceSectionBody, StructureNodeKind,
};

fn adapter() -> MarkdownAdapter {
    MarkdownAdapter
}

#[test]
fn build_structure_projects_root_and_sections() {
    let structure = adapter()
        .build_structure("# A\n\n## A1\nbody\n\n# B\n")
        .expect("build");

    assert_eq!(structure.nodes.len(), 4); // root, A, A1, B
    let root = structure
        .nodes
        .iter()
        .find(|n| n.id == structure.root_id)
        .expect("root present");
    assert_eq!(root.kind, StructureNodeKind::DocumentRoot);
    assert!(root.capabilities.can_delete.is_hidden());
    assert!(root.capabilities.can_show_plain_text.is_allowed());

    let a = structure.nodes.iter().find(|n| n.title == "A").unwrap();
    assert_eq!(a.kind, StructureNodeKind::MarkdownSection);
    assert_eq!(a.parent_id, Some(structure.root_id));
    assert_eq!(a.depth, 1);

    let a1 = structure.nodes.iter().find(|n| n.title == "A1").unwrap();
    assert_eq!(a1.parent_id, Some(a.id));
    assert_eq!(a1.depth, 2);
    assert!(a1.capabilities.can_move_out_one_level.is_allowed());
}

#[test]
fn rebuild_determinism_same_source_produces_same_ids() {
    let source = "# A\n\n## A1\nbody\n\n# B\nbody\n";
    let first = adapter().build_structure(source).expect("build");
    let second = adapter().build_structure(source).expect("build");

    let first_ids: Vec<_> = first.nodes.iter().map(|n| n.id).collect();
    let second_ids: Vec<_> = second.nodes.iter().map(|n| n.id).collect();
    assert_eq!(first_ids, second_ids);
    assert_eq!(first.root_id, second.root_id);
}

#[test]
fn focus_survives_an_edit_to_an_unrelated_node() {
    let source = "# A\nbody\n\n# B\nbody\n";
    let before = adapter().build_structure(source).expect("build");
    let b_id = before
        .nodes
        .iter()
        .find(|n| n.title == "B")
        .expect("B exists")
        .id;

    // Edit A's body through the shipped replacement path; B is unrelated.
    let mut document = Document::parse(source.to_string()).expect("parse");
    let a_id = document.outline().root().children[0];
    document
        .replace_section_body(ReplaceSectionBody {
            node_id: a_id,
            base_revision: document.revision(),
            new_body: "changed body\n\n".to_string(),
        })
        .expect("edit applies");

    let after = adapter()
        .build_structure(document.source())
        .expect("rebuild");
    let b_after = after
        .nodes
        .iter()
        .find(|n| n.id == b_id)
        .expect("B's id still resolves after rebuild");
    assert_eq!(b_after.title, "B");
}

#[test]
fn deep_nesting_preserves_ancestor_depth() {
    let source = "# A\n\n## A1\n\n### A1a\nbody\n";
    let structure = adapter().build_structure(source).expect("build");
    let deepest = structure.nodes.iter().find(|n| n.title == "A1a").unwrap();
    assert_eq!(deepest.depth, 3);
}

#[test]
fn depth_is_tree_depth_not_heading_level_across_a_skipped_level() {
    // RFC-007: a skipped heading level (H1 -> H4) attaches to the nearest
    // shallower heading, so tree depth and heading level diverge here.
    // "Jumped" is heading-level 4 (H4) but tree-depth 2: root(0) -> Top(1)
    // -> Jumped(2), since Jumped attaches directly under "Top" with no
    // synthetic H2/H3 in between.
    let source = "# Top\n\n#### Jumped\nbody\n";
    let structure = adapter().build_structure(source).expect("build");
    let top = structure.nodes.iter().find(|n| n.title == "Top").unwrap();
    let jumped = structure
        .nodes
        .iter()
        .find(|n| n.title == "Jumped")
        .unwrap();
    assert_eq!(
        jumped.parent_id,
        Some(top.id),
        "Jumped attaches under Top directly"
    );
    assert_eq!(
        jumped.depth, 2,
        "depth must count ancestors (2), not heading level (which would suggest 3 or 4)"
    );
}
