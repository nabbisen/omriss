//! Promote / demote tests (RFC-023).

use omriss_core::{Document, StructuralEditError};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

fn node_ids(d: &Document) -> Vec<omriss_core::NodeId> {
    d.outline().iter().map(|n| n.id).collect()
}

#[test]
fn promote_h2_to_h1() {
    let src = "# A\n\n## B\nbody\n";
    let mut d = doc(src);
    let b = node_ids(&d)[2];
    d.promote_section(b, d.revision()).unwrap();
    assert!(d.source().contains("# B\n"), "got: {:?}", d.source());
    assert!(d.source().starts_with("# A\n"), "prefix changed");
}

#[test]
fn demote_h1_to_h2() {
    let src = "# A\nbody\n";
    let mut d = doc(src);
    let a = node_ids(&d)[1];
    d.demote_section(a, d.revision()).unwrap();
    assert_eq!(d.source(), "## A\nbody\n");
}

#[test]
fn promote_h1_rejected() {
    let mut d = doc("# A\n");
    let a = node_ids(&d)[1];
    assert_eq!(
        d.promote_section(a, d.revision()),
        Err(StructuralEditError::InvalidLevel)
    );
}

#[test]
fn demote_h6_rejected() {
    let mut d = doc("###### A\n");
    let a = node_ids(&d)[1];
    assert_eq!(
        d.demote_section(a, d.revision()),
        Err(StructuralEditError::InvalidLevel)
    );
}

#[test]
fn promote_preserves_unrelated_bytes() {
    let src = "# Root\n\n## Target\nbody\n\n## Sibling\nkeep\n";
    let mut d = doc(src);
    let target = node_ids(&d)[2];
    let old = d.source().to_string();
    d.promote_section(target, d.revision()).unwrap();
    assert_eq!(
        d.source(),
        "# Root\n\n## Sibling\nkeep\n# Target\nbody\n\n",
        "moving out a non-last child must not absorb following siblings"
    );
    assert_ne!(d.source(), old);
}

#[test]
fn promote_undo_round_trip() {
    let src = "# Root\n\n## B\nbody\n";
    let mut d = doc(src);
    let b = node_ids(&d)[2];
    d.promote_section(b, d.revision()).unwrap();
    d.undo().unwrap();
    assert_eq!(d.source(), src);
}

#[test]
fn promote_non_last_child_preserves_following_sibling_under_parent() {
    let src = "# One\n\n## One One\nbody\n\n## One Two\nkeep\n\n# Two\n";
    let mut d = doc(src);
    let one_one = node_ids(&d)[2];

    d.promote_section(one_one, d.revision()).unwrap();

    assert_eq!(
        d.source(),
        "# One\n\n## One Two\nkeep\n\n# One One\nbody\n\n# Two\n"
    );
    let root_titles: Vec<&str> = d
        .outline()
        .children(d.outline().root_id())
        .unwrap()
        .into_iter()
        .map(|node| node.title.as_str())
        .collect();
    assert_eq!(root_titles, vec!["One", "One One", "Two"]);
    let one = d.outline().root().children[0];
    let one_children: Vec<&str> = d
        .outline()
        .children(one)
        .unwrap()
        .into_iter()
        .map(|node| node.title.as_str())
        .collect();
    assert_eq!(one_children, vec!["One Two"]);
}

#[test]
fn promote_shifts_descendants_to_preserve_subtree() {
    let src = "# One\n\n## One One\nbody\n\n### Child\nchild\n\n# Two\n";
    let mut d = doc(src);
    let one_one = node_ids(&d)[2];

    d.promote_section(one_one, d.revision()).unwrap();

    assert_eq!(
        d.source(),
        "# One\n\n# One One\nbody\n\n## Child\nchild\n\n# Two\n"
    );
    let promoted = d.outline().root().children[1];
    let child_titles: Vec<&str> = d
        .outline()
        .children(promoted)
        .unwrap()
        .into_iter()
        .map(|node| node.title.as_str())
        .collect();
    assert_eq!(child_titles, vec!["Child"]);
}

#[test]
fn demote_shifts_descendants_to_preserve_subtree() {
    let src = "# One\n\n# Two\nbody\n\n## Child\nchild\n";
    let mut d = doc(src);
    let two = d.outline().root().children[1];

    d.demote_section(two, d.revision()).unwrap();

    assert_eq!(d.source(), "# One\n\n## Two\nbody\n\n### Child\nchild\n");
    let one = d.outline().root().children[0];
    let one_children: Vec<&str> = d
        .outline()
        .children(one)
        .unwrap()
        .into_iter()
        .map(|node| node.title.as_str())
        .collect();
    assert_eq!(one_children, vec!["Two"]);
    let demoted = d.outline().children(one).unwrap()[0].id;
    let child_titles: Vec<&str> = d
        .outline()
        .children(demoted)
        .unwrap()
        .into_iter()
        .map(|node| node.title.as_str())
        .collect();
    assert_eq!(child_titles, vec!["Child"]);
}
