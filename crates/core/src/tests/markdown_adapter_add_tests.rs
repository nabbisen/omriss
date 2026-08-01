//! RFC-053 S4b: `MarkdownAdapter::structure_command` for `AddInside`/
//! `AddAfter` (no single shipped primitive — built from `split_section`
//! plus offset/level arithmetic), and the focused-content edit round trip.

use crate::{
    Document, DocumentFormatAdapter, HeadingLevel, MarkdownAdapter, NewNodeSpec, StructureCommand,
    StructureCommandError, StructureErrorKind,
};

fn adapter() -> MarkdownAdapter {
    MarkdownAdapter
}

fn doc(source: &str) -> Document {
    Document::parse(source.to_string()).expect("parse")
}

fn find_id(structure: &crate::DocumentStructure, title: &str) -> crate::NodeId {
    structure
        .nodes
        .iter()
        .find(|n| n.title == title)
        .unwrap_or_else(|| panic!("no node titled {title:?}"))
        .id
}

#[test]
fn add_inside_wraps_split_section_one_level_deeper() {
    let source = "# A\nbody\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let a_id = find_id(&structure, "A");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::AddInside {
                target: a_id,
                spec: NewNodeSpec {
                    title: "Child".to_string(),
                },
            },
        )
        .expect("command");

    // Shipped equivalent: split_section at full_range.end - body_range.start,
    // one heading level deeper (mirrors omriss_ui::session::structural::append_child_to_focused).
    let mut via_shipped = doc(source);
    let node = via_shipped.outline().node(a_id).unwrap().clone();
    let offset = node.full_range.end - node.body_range.start;
    let rev = via_shipped.revision();
    via_shipped
        .split_section(a_id, offset, "Child", HeadingLevel::H2, rev)
        .expect("split");

    assert_eq!(via_command.source(), via_shipped.source());
}

#[test]
fn add_inside_root_creates_a_top_level_h1_section() {
    let source = "# A\nbody\n";
    let structure = adapter().build_structure(source).expect("build");
    let root_id = structure.root_id;

    let mut document = doc(source);
    adapter()
        .structure_command(
            &mut document,
            &structure,
            StructureCommand::AddInside {
                target: root_id,
                spec: NewNodeSpec {
                    title: "New Top".to_string(),
                },
            },
        )
        .expect("command");

    let rebuilt = adapter()
        .build_structure(document.source())
        .expect("rebuild");
    assert!(rebuilt.nodes.iter().any(|n| n.title == "New Top"));
    // New top-level section is H1, appended after existing top-level content.
    assert!(document.source().ends_with("# New Top\n\n"));
}

#[test]
fn add_after_wraps_split_section_at_the_targets_own_level() {
    let source = "# A\n\n## A1\nbody\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let a1_id = find_id(&structure, "A1");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::AddAfter {
                target: a1_id,
                spec: NewNodeSpec {
                    title: "A2".to_string(),
                },
            },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let node = via_shipped.outline().node(a1_id).unwrap().clone();
    let offset = node.full_range.end - node.body_range.start;
    let rev = via_shipped.revision();
    via_shipped
        .split_section(a1_id, offset, "A2", HeadingLevel::H2, rev)
        .expect("split");

    assert_eq!(via_command.source(), via_shipped.source());
}

#[test]
fn add_after_root_is_rejected() {
    let source = "# A\n";
    let structure = adapter().build_structure(source).expect("build");
    let root_id = structure.root_id;

    let mut document = doc(source);
    let err = adapter()
        .structure_command(
            &mut document,
            &structure,
            StructureCommand::AddAfter {
                target: root_id,
                spec: NewNodeSpec {
                    title: "X".to_string(),
                },
            },
        )
        .expect_err("no shipped equivalent for add-after-root");
    assert_eq!(
        err,
        StructureCommandError {
            kind: StructureErrorKind::UnsupportedFeature
        }
    );
}

#[test]
fn focused_content_then_validate_then_apply_round_trips_a_body_edit() {
    let source = "# A\nold body\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let a_id = find_id(&structure, "A");

    let content = adapter()
        .focused_content(source, &structure, a_id)
        .expect("focused content");
    let crate::FocusedContent::MarkdownSection { title, body, .. } = content else {
        panic!("expected a MarkdownSection");
    };
    assert_eq!(title, "A");
    assert_eq!(body, "old body\n\n");

    let edit = adapter()
        .validate_focused_edit(source, &structure, a_id, "new body\n\n")
        .expect("validate");

    let mut document = doc(source);
    let applied = adapter()
        .apply_validated_edit(&mut document, edit)
        .expect("apply");
    assert!(applied.result.reindexed);
    assert_eq!(document.source(), "# A\nnew body\n\n# B\n");
}
