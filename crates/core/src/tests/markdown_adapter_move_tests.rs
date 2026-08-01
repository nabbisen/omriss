//! RFC-053 S4b: `MarkdownAdapter::structure_command` for `Move`,
//! `JoinWithPrevious`, `Rename`, and `Delete` — each proven byte-identical
//! to calling the shipped RFC-023/024/025 operation directly, plus undo.

use crate::{
    Document, DocumentFormatAdapter, MarkdownAdapter, MoveDirection, MoveTarget, StructureCommand,
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
fn move_inside_previous_wraps_demote_section() {
    let source = "# A\n\n# B\nbody\n\n# C\n";
    let structure = adapter().build_structure(source).expect("build");
    let b_id = find_id(&structure, "B");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::Move {
                target: b_id,
                direction: MoveDirection::InsidePrevious,
            },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped.demote_section(b_id, rev).expect("demote");

    assert_eq!(via_command.source(), via_shipped.source());
}

#[test]
fn move_out_one_level_wraps_promote_section() {
    let source = "# A\n\n## A1\nbody\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let a1_id = find_id(&structure, "A1");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::Move {
                target: a1_id,
                direction: MoveDirection::OutOneLevel,
            },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped.promote_section(a1_id, rev).expect("promote");

    assert_eq!(via_command.source(), via_shipped.source());
}

#[test]
fn move_up_wraps_move_section_before_previous_sibling() {
    let source = "# A\n\n# B\n\n# C\n";
    let structure = adapter().build_structure(source).expect("build");
    let b_id = find_id(&structure, "B");
    let a_id = find_id(&structure, "A");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::Move {
                target: b_id,
                direction: MoveDirection::Up,
            },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped
        .move_section(b_id, MoveTarget::Before(a_id), rev)
        .expect("move");

    assert_eq!(via_command.source(), via_shipped.source());
    assert_eq!(via_command.source(), "# B\n\n# A\n\n# C\n");
}

#[test]
fn move_down_wraps_move_section_after_next_sibling() {
    let source = "# A\n\n# B\n\n# C\n";
    let structure = adapter().build_structure(source).expect("build");
    let b_id = find_id(&structure, "B");
    let c_id = find_id(&structure, "C");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::Move {
                target: b_id,
                direction: MoveDirection::Down,
            },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped
        .move_section(b_id, MoveTarget::After(c_id), rev)
        .expect("move");

    assert_eq!(via_command.source(), via_shipped.source());
    // Byte-preserving move: B's own trailing blank line moves with it
    // rather than a separator being normalized at the new position.
    assert_eq!(via_command.source(), "# A\n\n# C\n# B\n\n");
}

#[test]
fn move_up_with_no_previous_sibling_is_an_error() {
    let source = "# A\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let a_id = find_id(&structure, "A");

    let mut document = doc(source);
    let err = adapter()
        .structure_command(
            &mut document,
            &structure,
            StructureCommand::Move {
                target: a_id,
                direction: MoveDirection::Up,
            },
        )
        .expect_err("no previous sibling");
    assert_eq!(
        err,
        StructureCommandError {
            kind: StructureErrorKind::UnsafeRange
        }
    );
    // The document is untouched on error.
    assert_eq!(document.source(), source);
}

#[test]
fn join_with_previous_wraps_merge_with_prev_sibling() {
    let source = "# A\n\n# B\nbody\n";
    let structure = adapter().build_structure(source).expect("build");
    let b_id = find_id(&structure, "B");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::JoinWithPrevious { target: b_id },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped
        .merge_with_prev_sibling(b_id, rev)
        .expect("merge");

    assert_eq!(via_command.source(), via_shipped.source());
}

#[test]
fn rename_wraps_rename_section() {
    let source = "# A\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let a_id = find_id(&structure, "A");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::Rename {
                target: a_id,
                new_name: "Renamed".to_string(),
            },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped
        .rename_section(a_id, "Renamed", rev)
        .expect("rename");

    assert_eq!(via_command.source(), via_shipped.source());
    assert_eq!(via_command.source(), "# Renamed\n\n# B\n");
}

#[test]
fn delete_wraps_delete_section() {
    let source = "# A\n\n# B\nbody\n\n# C\n";
    let structure = adapter().build_structure(source).expect("build");
    let b_id = find_id(&structure, "B");

    let mut via_command = doc(source);
    adapter()
        .structure_command(
            &mut via_command,
            &structure,
            StructureCommand::Delete { target: b_id },
        )
        .expect("command");

    let mut via_shipped = doc(source);
    let rev = via_shipped.revision();
    via_shipped.delete_section(b_id, rev).expect("delete");

    assert_eq!(via_command.source(), via_shipped.source());
    assert_eq!(via_command.source(), "# A\n\n# C\n");
}

#[test]
fn undo_after_a_structure_command_restores_the_original_source() {
    let source = "# A\n\n# B\n";
    let structure = adapter().build_structure(source).expect("build");
    let b_id = find_id(&structure, "B");

    let mut document = doc(source);
    adapter()
        .structure_command(
            &mut document,
            &structure,
            StructureCommand::Delete { target: b_id },
        )
        .expect("command");
    assert_ne!(document.source(), source);

    document.undo().expect("undo");
    assert_eq!(document.source(), source);
}
