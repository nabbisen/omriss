//! Move section tests (RFC-024).

use omriss_core::{Document, MoveTarget, StructuralEditError};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

fn node_ids(d: &Document) -> Vec<omriss_core::NodeId> {
    d.outline().iter().map(|n| n.id).collect()
}

#[test]
fn move_before_sibling() {
    let src = "# A\n\n# B\n\n# C\n";
    let mut d = doc(src);
    let ids = node_ids(&d);
    let c = ids[3];
    let a = ids[1];
    d.move_section(c, MoveTarget::Before(a), d.revision())
        .unwrap();
    let new = d.source();
    let pos_c = new.find("# C\n").unwrap();
    let pos_a = new.find("# A\n").unwrap();
    assert!(pos_c < pos_a, "C should appear before A");
}

#[test]
fn move_after_sibling() {
    let src = "# A\n\n# B\n\n# C\n";
    let mut d = doc(src);
    let ids = node_ids(&d);
    let a = ids[1];
    let c = ids[3];
    d.move_section(a, MoveTarget::After(c), d.revision())
        .unwrap();
    let new = d.source();
    let pos_a = new.find("# A\n").unwrap();
    let pos_c = new.find("# C\n").unwrap();
    assert!(pos_c < pos_a, "A should appear after C");
}

#[test]
fn move_into_descendant_rejected() {
    let src = "# A\n\n## A.1\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    let a1 = d.outline().node(a).unwrap().children[0];
    assert_eq!(
        d.move_section(a, MoveTarget::Before(a1), d.revision()),
        Err(StructuralEditError::CannotMoveIntoDescendant)
    );
}

#[test]
fn move_self_rejected() {
    let src = "# A\n\n# B\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    assert_eq!(
        d.move_section(a, MoveTarget::Before(a), d.revision()),
        Err(StructuralEditError::CannotMoveSelf)
    );
}

#[test]
fn move_preserves_subtree_bytes() {
    let src = "# A\n\n## A.1\ntext\n\n# B\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    let b = *d.outline().root().children.last().unwrap();
    d.move_section(a, MoveTarget::After(b), d.revision())
        .unwrap();
    let new = d.source();
    assert!(new.starts_with("# B\n"), "{:?}", new);
    assert!(new.contains("## A.1\ntext\n"), "{:?}", new);
}

#[test]
fn move_undo_round_trip() {
    let src = "# A\n\n# B\n\n# C\n";
    let mut d = doc(src);
    let ids = node_ids(&d);
    let a = ids[1];
    let c = ids[3];
    d.move_section(a, MoveTarget::After(c), d.revision())
        .unwrap();
    d.undo().unwrap();
    assert_eq!(d.source(), src);
}
