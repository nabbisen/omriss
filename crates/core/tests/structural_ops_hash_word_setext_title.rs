//! RFC-065 §2.8: a setext heading whose title *starts with* `#` is not an ATX
//! heading, so rename and join must take their setext branch.
//!
//! `rename_section` and `merge_with_prev_sibling` used to guard their ATX path
//! with `line.starts_with('#')`. For `#x\n---` that sent a setext title through
//! `atx_title_range`, which stripped the `#` as if it were a heading marker.

use omriss_core::Document;

fn id_of(doc: &Document, title: &str) -> omriss_core::NodeId {
    doc.outline()
        .iter()
        .find(|n| n.title == title)
        .unwrap_or_else(|| panic!("no node titled {title:?}"))
        .id
}

fn titles(doc: &Document) -> Vec<String> {
    doc.outline().iter().map(|n| n.title.clone()).collect()
}

#[test]
fn renaming_a_hash_word_setext_heading_replaces_the_whole_title_line() {
    let mut doc = Document::parse("# One\n\n#x\n---\nrest\n".to_string()).unwrap();
    assert_eq!(titles(&doc), ["", "One", "#x"]);
    let id = id_of(&doc, "#x");

    let rev = doc.revision();
    doc.rename_section(id, "Foo", rev).unwrap();

    assert_eq!(
        doc.source(),
        "# One\n\nFoo\n---\nrest\n",
        "the title `#x` is replaced as a whole; the `#` is not a heading marker"
    );
    assert_eq!(titles(&doc), ["", "One", "Foo"]);
}

#[test]
fn joining_a_hash_word_setext_heading_keeps_every_byte_of_its_title() {
    let mut doc = Document::parse("## P\n\ntext\n\n#x\n---\nrest\n".to_string()).unwrap();
    let id = id_of(&doc, "#x");

    let rev = doc.revision();
    doc.merge_with_prev_sibling(id, rev).unwrap();

    assert_eq!(
        doc.source(),
        "## P\n\ntext\n\n#x\nrest\n",
        "join removes the setext underline only; the user's `#` must survive"
    );
}

#[test]
fn a_real_atx_heading_is_still_renamed_and_joined_through_the_atx_branch() {
    // The fix swaps the guard, so the ATX branch must be unaffected.
    let mut doc = Document::parse("# One\n\n## Two\nbody\n".to_string()).unwrap();
    let id = id_of(&doc, "Two");
    let rev = doc.revision();
    doc.rename_section(id, "Deux", rev).unwrap();
    assert_eq!(doc.source(), "# One\n\n## Deux\nbody\n");

    let mut doc = Document::parse("## P\n\ntext\n\n## Q\nrest\n".to_string()).unwrap();
    let id = id_of(&doc, "Q");
    let rev = doc.revision();
    doc.merge_with_prev_sibling(id, rev).unwrap();
    assert_eq!(doc.source(), "## P\n\ntext\n\nQ\nrest\n");
}
