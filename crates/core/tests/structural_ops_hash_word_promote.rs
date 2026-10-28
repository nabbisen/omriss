//! RFC-065 §2.9: `promote`/`demote` must not mistake a setext heading whose
//! title starts with `#` for an ATX heading and overwrite its title text.
//!
//! The guard was `source[heading_start] == '#'`; for `#x\n---` the marker
//! edit then replaced the first `level` bytes of the user's title.

use omriss_core::{Document, StructuralEditError};

fn id_of(doc: &Document, title: &str) -> omriss_core::NodeId {
    doc.outline()
        .iter()
        .find(|n| n.title == title)
        .unwrap_or_else(|| panic!("no node titled {title:?}"))
        .id
}

/// The error of a result that must be an error, or a failure that shows what
/// the operation wrongly did to the user's text.
fn refused<T>(
    result: Result<T, StructuralEditError>,
    doc: &Document,
    op: &str,
) -> StructuralEditError {
    match result {
        Err(e) => e,
        Ok(_) => panic!(
            "{op} must be refused for a setext heading, but it succeeded and the source is now {:?}",
            doc.source()
        ),
    }
}

#[test]
fn promoting_a_hash_word_setext_heading_is_refused_and_changes_nothing() {
    let src = "# One\n\n#x\n---\nrest\n";
    let mut doc = Document::parse(src.to_string()).unwrap();
    let id = id_of(&doc, "#x");
    let rev = doc.revision();

    let err = refused(doc.promote_section(id, rev), &doc, "promote");

    assert!(
        matches!(err, StructuralEditError::UnsupportedHeadingStyle),
        "setext cannot be promoted, even when its title starts with `#`: {err:?}"
    );
    assert_eq!(doc.source(), src, "a refused promote must not touch a byte");
}

#[test]
fn promoting_a_hash_word_with_a_longer_title_does_not_eat_a_letter() {
    // The worse shape: a character vanishes from the middle of a word and the
    // result still reads as a plausible title.
    let src = "# One\n\n#hashtag word\n---\nrest\n";
    let mut doc = Document::parse(src.to_string()).unwrap();
    let id = id_of(&doc, "#hashtag word");
    let rev = doc.revision();

    let err = refused(doc.promote_section(id, rev), &doc, "promote");

    assert!(
        matches!(err, StructuralEditError::UnsupportedHeadingStyle),
        "{err:?}"
    );
    assert_eq!(
        doc.source(),
        src,
        "`#hashtag word` must not become `#ashtag word`"
    );
}

#[test]
fn demoting_a_hash_word_setext_heading_is_refused_too() {
    let src = "# One\n\n#x\n---\nrest\n";
    let mut doc = Document::parse(src.to_string()).unwrap();
    let id = id_of(&doc, "#x");
    let rev = doc.revision();
    let err = refused(doc.demote_section(id, rev), &doc, "demote");
    assert!(
        matches!(err, StructuralEditError::UnsupportedHeadingStyle),
        "{err:?}"
    );
    assert_eq!(doc.source(), src);
}

#[test]
fn control_an_ordinary_setext_heading_is_still_refused() {
    let src = "# One\n\nplain\n---\nrest\n";
    let mut doc = Document::parse(src.to_string()).unwrap();
    let id = id_of(&doc, "plain");
    let rev = doc.revision();
    let err = refused(doc.promote_section(id, rev), &doc, "promote");
    assert!(
        matches!(err, StructuralEditError::UnsupportedHeadingStyle),
        "{err:?}"
    );
    assert_eq!(doc.source(), src);
}

#[test]
fn control_an_indented_atx_heading_still_promotes_and_demotes() {
    // `heading_range.start` points at the `#`, not at the indentation, so the
    // predicate's three-space allowance cannot misfire here (RFC-065 §2.9).
    let src = "# One\n\n  ## x\nbody\n";
    let doc = Document::parse(src.to_string()).unwrap();
    let node = doc.outline().iter().find(|n| n.title == "x").unwrap();
    assert_eq!(
        &src[node.heading_range.start..node.heading_range.start + 2],
        "##",
        "heading_range must start at the `#`, past the indentation"
    );

    let mut doc = Document::parse(src.to_string()).unwrap();
    let id = id_of(&doc, "x");
    let rev = doc.revision();
    doc.promote_section(id, rev).unwrap();
    assert_eq!(doc.source(), "# One\n\n  # x\nbody\n");

    let mut doc = Document::parse(src.to_string()).unwrap();
    let id = id_of(&doc, "x");
    let rev = doc.revision();
    doc.demote_section(id, rev).unwrap();
    assert_eq!(doc.source(), "# One\n\n  ### x\nbody\n");
}
