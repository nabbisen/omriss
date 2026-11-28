//! RFC-070 §10: renaming a setext heading replaces its whole title.
//!
//! A setext title may span lines; the Document Map shows them as one title, so a
//! rename to "New" must read "New", not "New second line". Every title line becomes
//! the one requested line; the underline, the body and every line ending stay.

use omriss_core::{Document, StructuralEditError};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

fn rename_first(src: &str, to: &str) -> (Document, String) {
    let mut d = doc(src);
    let id = d.outline().root().children[0];
    d.rename_section(id, to, d.revision()).unwrap();
    let title = d
        .outline()
        .node(d.outline().root().children[0])
        .unwrap()
        .title
        .clone();
    (d, title)
}

#[test]
fn a_two_line_title_becomes_the_requested_line() {
    let (d, title) = rename_first("para\nsecond line\n===\nbody\n", "New");
    assert_eq!(d.source(), "New\n===\nbody\n");
    assert_eq!(title, "New");
}

#[test]
fn a_three_line_title_becomes_the_requested_line() {
    let (d, title) = rename_first("one\ntwo\nthree\n---\nbody\n\n# Next\n", "New");
    assert_eq!(d.source(), "New\n---\nbody\n\n# Next\n");
    assert_eq!(title, "New");
}

#[test]
fn line_endings_are_kept_in_crlf_cr_and_mixed() {
    // The line break after the title is the one that stood before the underline.
    for (src, want) in [
        (
            "para\r\nsecond\r\n===\r\nbody\r\n",
            "New\r\n===\r\nbody\r\n",
        ),
        ("para\rsecond\r===\rbody\r", "New\r===\rbody\r"),
        ("para\r\nsecond\n===\r\nbody\n", "New\n===\r\nbody\n"),
    ] {
        let (d, title) = rename_first(src, "New");
        assert_eq!(d.source(), want, "{src:?}");
        assert_eq!(title, "New", "{src:?}");
    }
}

#[test]
fn a_heading_at_the_end_of_the_file_with_no_final_line_break() {
    let (d, title) = rename_first("para\nsecond\n===", "New");
    assert_eq!(d.source(), "New\n===");
    assert_eq!(title, "New");
}

#[test]
fn a_one_line_setext_title_and_an_atx_heading_are_as_before() {
    let (d, _) = rename_first("Old\n===\nbody\n", "New");
    assert_eq!(d.source(), "New\n===\nbody\n");
    let (d, _) = rename_first("# Old ##\nbody\n", "New");
    assert_eq!(d.source(), "# New ##\nbody\n");
}

#[test]
fn a_setext_title_starting_with_a_hash_is_still_a_setext_title() {
    // RFC-065 §2.8's shape, over two lines.
    let (d, title) = rename_first("#x\nsecond\n---\nbody\n", "New");
    assert_eq!(d.source(), "New\n---\nbody\n");
    assert_eq!(title, "New");
}

#[test]
fn only_the_renamed_heading_changes() {
    let src = "# A\nbody\n\npara\nsecond\n===\ntext\n\n## C\n";
    let mut d = doc(src);
    let id = d.outline().root().children[1];
    d.rename_section(id, "New", d.revision()).unwrap();
    assert_eq!(d.source(), "# A\nbody\n\nNew\n===\ntext\n\n## C\n");
}

#[test]
fn a_refused_title_changes_nothing() {
    let src = "para\nsecond\n===\nbody\n";
    let mut d = doc(src);
    let id = d.outline().root().children[0];
    for bad in ["", "  ", "a\nb"] {
        assert_eq!(
            d.rename_section(id, bad, d.revision()).unwrap_err(),
            StructuralEditError::InvalidTitle
        );
        assert_eq!(d.source(), src);
    }
}

#[test]
fn undo_and_redo_are_exact() {
    for src in [
        "para\nsecond line\n===\nbody\n",
        "one\r\ntwo\r\nthree\r\n---\r\nbody\r\n",
        "para\nsecond\n===",
    ] {
        let mut d = doc(src);
        let id = d.outline().root().children[0];
        d.rename_section(id, "New", d.revision()).unwrap();
        let renamed = d.source().to_string();
        assert_ne!(renamed, src);
        d.undo().unwrap();
        assert_eq!(d.source(), src, "undo of {src:?}");
        d.redo().unwrap();
        assert_eq!(d.source(), renamed, "redo of {src:?}");
    }
}
