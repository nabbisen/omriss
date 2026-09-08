//! RFC-004/008: source-preserving replacement semantics, the validation
//! sequence (revision check → resolve → validate → apply → re-index), and
//! verbatim replacement storage.

use crate::{Document, DocumentRevision, EditError, NodeId, ReplaceSectionBody};

fn doc(source: &str) -> Document {
    Document::parse(source.to_string()).expect("parse")
}

fn replace(
    document: &mut Document,
    id: NodeId,
    new_body: &str,
) -> Result<crate::EditResult, EditError> {
    document.replace_section_body(ReplaceSectionBody {
        node_id: id,
        base_revision: document.revision(),
        new_body: new_body.to_string(),
    })
}

#[test]
fn replacing_one_body_preserves_every_unrelated_byte() {
    let source = "# A\nA body\n\n## A.1\nchild\n\n# B\nB body\n";
    let mut document = doc(source);
    let a = document.outline().root().children[0];
    let body = document.outline().node(a).unwrap().body_range;

    replace(&mut document, a, "NEW BODY\n\n").unwrap();

    let after = document.source();
    assert_eq!(
        &after[..body.start],
        &source[..body.start],
        "prefix changed"
    );
    let suffix_before = &source[body.end..];
    assert_eq!(
        &after[after.len() - suffix_before.len()..],
        suffix_before,
        "suffix changed"
    );
    assert_eq!(after, "# A\nNEW BODY\n\n## A.1\nchild\n\n# B\nB body\n");
}

#[test]
fn heading_line_and_children_are_preserved() {
    let mut document = doc("# A\nold\n## Child\nchild body\n");
    let a = document.outline().root().children[0];
    replace(&mut document, a, "new\n").unwrap();
    assert_eq!(document.source(), "# A\nnew\n## Child\nchild body\n");
}

#[test]
fn replacement_is_stored_verbatim_without_normalization() {
    // No blank-line normalization, no reflow, no trimming (RFC-004
    // whitespace policy). But per RFC-004's 2026-09-01 amendment (RFC-065),
    // core does insert the *minimum* separator needed to keep block
    // structure intact at a body boundary — a replacement is never allowed
    // to weld onto the next heading and turn it into plain text. Here the
    // replacement has no trailing newline of its own, and "# B" follows
    // immediately with none between them, so exactly one line break is
    // inserted (not two: "# B" is ATX-shaped, which CommonMark always
    // recognizes as a fresh block after a single break).
    let mut document = doc("# A\nold body\n\n# B\n");
    let a = document.outline().root().children[0];
    replace(&mut document, a, "no trailing newline").unwrap();
    assert_eq!(document.source(), "# A\nno trailing newline\n# B\n");
}

#[test]
fn stale_revision_is_rejected_before_any_mutation() {
    let mut document = doc("# A\nbody\n");
    let a = document.outline().root().children[0];
    let stale = document.revision();
    replace(&mut document, a, "first edit\n").unwrap();

    let err = document
        .replace_section_body(ReplaceSectionBody {
            node_id: a,
            base_revision: stale,
            new_body: "second edit\n".to_string(),
        })
        .unwrap_err();
    assert!(matches!(err, EditError::RevisionMismatch { .. }));
    assert_eq!(document.source(), "# A\nfirst edit\n");
}

#[test]
fn unknown_node_is_a_stale_node_error() {
    let mut document = doc("# A\nbody\n");
    let err = replace(&mut document, NodeId(0xdead_beef), "x").unwrap_err();
    assert_eq!(err, EditError::StaleNode(NodeId(0xdead_beef)));
}

#[test]
fn revision_increments_exactly_once_per_successful_edit() {
    let mut document = doc("# A\nbody\n");
    let a = document.outline().root().children[0];
    assert_eq!(document.revision(), DocumentRevision::INITIAL);
    let result = replace(&mut document, a, "x\n").unwrap();
    assert_eq!(result.old_revision, DocumentRevision(0));
    assert_eq!(result.new_revision, DocumentRevision(1));
    assert_eq!(document.revision(), DocumentRevision(1));
}

#[test]
fn edit_result_reports_pre_and_post_edit_ranges() {
    let mut document = doc("# A\nold\n");
    let a = document.outline().root().children[0];
    let body = document.outline().node(a).unwrap().body_range;
    let result = replace(&mut document, a, "longer body\n").unwrap();
    assert_eq!(result.replaced_range, body);
    assert_eq!(result.new_range.start, body.start);
    assert_eq!(result.new_range.len(), "longer body\n".len());
    assert!(result.reindexed);
}

#[test]
fn node_id_survives_body_only_edits() {
    // RFC-006 stability requirement: body replacement that does not touch
    // heading lines keeps IDs stable after re-index.
    let mut document = doc("# A\nshort\n# B\nbody b\n");
    let ids_before = document.outline().root().children.clone();
    replace(
        &mut document,
        ids_before[0],
        "a much, much longer body than before\n",
    )
    .unwrap();
    let ids_after = document.outline().root().children.clone();
    assert_eq!(ids_before, ids_after);
    // The later section's body is still addressable through its old ID.
    assert_eq!(document.section_body(ids_after[1]).unwrap(), "body b\n");
}

#[test]
fn editing_root_preface_preserves_sections() {
    let mut document = doc("preface\n\n# A\nbody\n");
    let root = document.outline().root_id();
    replace(&mut document, root, "new preface\n\n").unwrap();
    assert_eq!(document.source(), "new preface\n\n# A\nbody\n");
}

#[test]
fn multibyte_bodies_replace_safely() {
    let mut document = doc("# 見出し\n日本語の本文です。\n\n# 次\n保持\n");
    let first = document.outline().root().children[0];
    replace(&mut document, first, "編集後の本文 🎉\n\n").unwrap();
    assert_eq!(
        document.source(),
        "# 見出し\n編集後の本文 🎉\n\n# 次\n保持\n"
    );
}

#[test]
fn structural_body_edit_that_adds_headings_reindexes() {
    let mut document = doc("# A\nbody\n");
    let a = document.outline().root().children[0];
    replace(&mut document, a, "intro\n\n## New Child\nchild body\n").unwrap();
    let outline = document.outline();
    let a_node = outline.children(outline.root_id()).unwrap()[0];
    let children = outline.children(a_node.id).unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].title, "New Child");
}
