//! Delete, split, and merge tests (RFC-025).

use omriss_core::{Document, HeadingLevel, StructuralEditError};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

// ── Delete ─────────────────────────────────────────────────────────────────

#[test]
fn delete_removes_full_range() {
    let src = "# A\nbody A\n\n# B\nbody B\n";
    let mut d = doc(src);
    let b = *d.outline().root().children.last().unwrap();
    d.delete_section(b, d.revision()).unwrap();
    assert_eq!(d.source(), "# A\nbody A\n\n");
}

#[test]
fn delete_with_children() {
    let src = "# A\n\n## A.1\ntext\n\n## A.2\ntext\n\n# B\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    d.delete_section(a, d.revision()).unwrap();
    assert_eq!(d.source(), "# B\n");
}

#[test]
fn delete_root_rejected() {
    let mut d = doc("# A\n");
    let root = d.outline().root_id();
    assert_eq!(
        d.delete_section(root, d.revision()),
        Err(StructuralEditError::CannotDeleteRoot)
    );
}

#[test]
fn delete_undo_round_trip() {
    let src = "# A\nbody\n\n# B\nkeep\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    d.delete_section(a, d.revision()).unwrap();
    d.undo().unwrap();
    assert_eq!(d.source(), src);
}

// ── Split ──────────────────────────────────────────────────────────────────

#[test]
fn split_at_body_end_appends_child() {
    let src = "# A\nbody text\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    let body_len = d.outline().node(a).unwrap().body_range.len();
    d.split_section(a, body_len, "Child", HeadingLevel::H2, d.revision())
        .unwrap();
    assert!(d.source().contains("## Child\n"), "{:?}", d.source());
    assert!(
        d.source().starts_with("# A\nbody text\n"),
        "{:?}",
        d.source()
    );
}

#[test]
fn split_at_zero_prepends_child() {
    let src = "# A\nbody\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    d.split_section(a, 0, "New", HeadingLevel::H2, d.revision())
        .unwrap();
    assert!(d.source().contains("## New\n"), "{:?}", d.source());
}

#[test]
fn split_invalid_offset_rejected() {
    let src = "# A\nbody\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    assert!(
        d.split_section(a, 9999, "X", HeadingLevel::H2, d.revision())
            .is_err()
    );
}

// ── Rename ─────────────────────────────────────────────────────────────────

#[test]
fn rename_atx_heading_preserves_body_children_and_closing_marker() {
    let src = "# Old ###\nbody\n\n## Child\nchild\n\n# Next\n";
    let mut d = doc(src);
    let old = d.outline().root().children[0];
    d.rename_section(old, "New", d.revision()).unwrap();
    assert_eq!(d.source(), "# New ###\nbody\n\n## Child\nchild\n\n# Next\n");
}

#[test]
fn rename_setext_heading_preserves_underline_and_body() {
    let src = "Old\n===\nbody\n\n# Next\n";
    let mut d = doc(src);
    let old = d.outline().root().children[0];
    d.rename_section(old, "New", d.revision()).unwrap();
    assert_eq!(d.source(), "New\n===\nbody\n\n# Next\n");
}

#[test]
fn rename_undo_round_trip() {
    let src = "# Old\nbody\n";
    let mut d = doc(src);
    let old = d.outline().root().children[0];
    d.rename_section(old, "New", d.revision()).unwrap();
    d.undo().unwrap();
    assert_eq!(d.source(), src);
}

// ── Merge ──────────────────────────────────────────────────────────────────

#[test]
fn merge_preserves_heading_title_as_plain_text() {
    let src = "# A\nbody A\n\n# B\nbody B\n";
    let mut d = doc(src);
    let b = *d.outline().root().children.last().unwrap();
    d.merge_with_prev_sibling(b, d.revision()).unwrap();
    assert_eq!(d.source(), "# A\nbody A\n\nB\nbody B\n");
    let root_children = &d.outline().root().children;
    assert_eq!(root_children.len(), 1);
    assert_eq!(d.outline().node(root_children[0]).unwrap().title, "A");
}

#[test]
fn merge_preserves_heading_title_when_section_has_no_body() {
    let src = "# A\nbody A\n\n# B\n";
    let mut d = doc(src);
    let b = *d.outline().root().children.last().unwrap();
    d.merge_with_prev_sibling(b, d.revision()).unwrap();
    assert_eq!(d.source(), "# A\nbody A\n\nB\n");
}

#[test]
fn merge_third_empty_section_into_second_empty_section() {
    let src = "# A.\n# B.\n# C.\n";
    let mut d = doc(src);
    let c = d.outline().root().children[2];
    d.merge_with_prev_sibling(c, d.revision()).unwrap();
    assert_eq!(d.source(), "# A.\n# B.\nC.\n");
    let root_children = &d.outline().root().children;
    assert_eq!(root_children.len(), 2);
    assert_eq!(d.outline().node(root_children[1]).unwrap().title, "B.");
}

#[test]
fn merge_preserves_setext_heading_title_as_plain_text() {
    let src = "A\n===\nbody A\n\nB\n===\nbody B\n";
    let mut d = doc(src);
    let b = *d.outline().root().children.last().unwrap();
    d.merge_with_prev_sibling(b, d.revision()).unwrap();
    assert_eq!(d.source(), "A\n===\nbody A\n\nB\nbody B\n");
}

#[test]
fn merge_preserves_heading_title_line_ending() {
    let src = "# A\r\nbody A\r\n\r\n# B\r\nbody B\r\n";
    let mut d = doc(src);
    let b = *d.outline().root().children.last().unwrap();
    d.merge_with_prev_sibling(b, d.revision()).unwrap();
    assert_eq!(d.source(), "# A\r\nbody A\r\n\r\nB\r\nbody B\r\n");
}

#[test]
fn merge_first_sibling_rejected() {
    let src = "# A\nbody\n\n# B\nbody\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    assert_eq!(
        d.merge_with_prev_sibling(a, d.revision()),
        Err(StructuralEditError::NoAdjacentSibling)
    );
}

#[test]
fn merge_rejects_when_previous_sibling_has_children() {
    let src = "# A\n\n## A.1\nchild\n\n# B\nbody B\n";
    let mut d = doc(src);
    let b = d.outline().root().children[1];
    assert_eq!(
        d.merge_with_prev_sibling(b, d.revision()),
        Err(StructuralEditError::UnsafePreservation)
    );
    assert_eq!(d.source(), src);
}

#[test]
fn merge_undo_round_trip() {
    let src = "# A\nbody A\n\n# B\nbody B\n";
    let mut d = doc(src);
    let b = *d.outline().root().children.last().unwrap();
    d.merge_with_prev_sibling(b, d.revision()).unwrap();
    d.undo().unwrap();
    assert_eq!(d.source(), src);
}
