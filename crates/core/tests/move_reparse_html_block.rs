//! Pins the triage of the RFC-066 P2 move failure (§3.1a).

use omriss_core::{Document, MoveTarget};

/// Moving a section whose body holds an unterminated HTML block can change
/// what parses as a heading. Every byte survives; CommonMark re-reads them.
/// Not corruption -- RFC-066 §3.1a, warned about by RFC-071 §2.1.
///
/// `<?` opens an HTML block of type 3, which ends only at `?>` and otherwise
/// runs to the end of the document. While it is the last thing in the file it
/// swallows nothing; once a setext heading follows it, the heading is part of
/// the block. This test fails loudly if `move_section` ever starts actually
/// deleting bytes, or if the outline stops being re-read this way.
#[test]
fn moving_a_section_with_an_unterminated_html_block_reparses_but_loses_no_bytes() {
    let before = "##### A\nA\n===\n#### a\n<?\n";
    let mut doc = Document::parse(before.to_string()).unwrap();
    assert_eq!(
        doc.outline().len(),
        4,
        "root + `##### A` + setext `A` + `a`"
    );

    let ids_by_title = |doc: &Document, title: &str| {
        doc.outline()
            .iter()
            .filter(|n| n.title == title)
            .map(|n| n.id)
            .collect::<Vec<_>>()
    };
    let a_lower = ids_by_title(&doc, "a")[0];
    let atx_a = ids_by_title(&doc, "A")[0]; // `##### A`, first in document order

    let rev = doc.revision();
    doc.move_section(a_lower, MoveTarget::After(atx_a), rev)
        .unwrap();

    assert_eq!(
        doc.source(),
        "##### A\n#### a\n<?\n\nA\n===\n",
        "the moved bytes are exactly the original ones, plus one separator"
    );
    assert!(
        doc.source().contains("A\n==="),
        "the setext heading's bytes must still be present"
    );
    assert_eq!(
        doc.outline().len(),
        3,
        "the node count drops 4 -> 3 because `<?` now swallows `A\\n===` as HTML"
    );
}
