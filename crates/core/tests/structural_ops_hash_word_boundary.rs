//! RFC-065 §2.7: a line that merely *starts with* `#` is not an ATX heading.
//!
//! `joining_separator` once treated any `#`-initial line as one and granted the
//! one-break shortcut, so a following `#x` + setext underline was captured as
//! the last line of the body just written.

use omriss_core::{Document, ReplaceSectionBody};

fn titles(doc: &Document) -> Vec<String> {
    doc.outline().iter().map(|n| n.title.clone()).collect()
}

#[test]
fn writing_a_body_before_a_hash_word_setext_heading_leaves_that_heading_alone() {
    let mut doc = Document::parse("# One\n#x\n---\nrest\n".to_string()).unwrap();
    assert_eq!(titles(&doc), ["", "One", "#x"]);

    let one = doc.outline().iter().find(|n| n.title == "One").unwrap().id;
    let rev = doc.revision();
    doc.replace_section_body(ReplaceSectionBody {
        node_id: one,
        base_revision: rev,
        new_body: "hello".to_string(),
    })
    .unwrap();

    assert_eq!(
        titles(&doc),
        ["", "One", "#x"],
        "typing into One must not rename Two (source: {:?})",
        doc.source()
    );
    assert!(
        doc.source().ends_with("#x\n---\nrest\n"),
        "the following heading's bytes must be untouched: {:?}",
        doc.source()
    );
    assert_eq!(doc.section_body(one).unwrap().trim_end(), "hello");
}

#[test]
fn a_real_atx_heading_still_gets_the_one_break_shortcut() {
    // Deliberate and correct (RFC-065 §2.7 / constraint 2): no blank line is
    // forced before an ATX heading the user already had directly below.
    let mut doc = Document::parse("# One\n# Two\nrest\n".to_string()).unwrap();
    let one = doc.outline().iter().find(|n| n.title == "One").unwrap().id;
    let rev = doc.revision();
    doc.replace_section_body(ReplaceSectionBody {
        node_id: one,
        base_revision: rev,
        new_body: "hello".to_string(),
    })
    .unwrap();
    assert_eq!(doc.source(), "# One\nhello\n# Two\nrest\n");
    assert_eq!(titles(&doc), ["", "One", "Two"]);
}
