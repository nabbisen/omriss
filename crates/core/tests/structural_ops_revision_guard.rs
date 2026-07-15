//! Revision-mismatch guard tests (RFC-026).

use omriss_core::{Document, StructuralEditError};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

#[test]
fn structural_revision_mismatch_rejected() {
    let mut d = doc("# A\n");
    let a = d.outline().root().children[0];
    let stale_rev = d.revision();
    d.replace_section_body(omriss_core::ReplaceSectionBody {
        node_id: a,
        base_revision: d.revision(),
        new_body: "changed\n".into(),
    })
    .unwrap();
    assert!(matches!(
        d.demote_section(a, stale_rev),
        Err(StructuralEditError::RevisionMismatch { .. })
    ));
}
