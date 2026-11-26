//! RFC-084 §11.3: every place in core that finds the end of a line uses the
//! three-ending rule (`\r\n`, `\n`, lone `\r`). The structural edits that read a
//! heading's line — rename, join (merge), delete, split — run on one document in
//! each ending; the expected bytes are written with `\n` and converted to the
//! ending under test, so each ending must give the same edit.

use crate::{Document, NodeId};

const DOC: &str = "# A\nbody a\n\n# Bee\nbody b\n\n# C\nbody c\n";

fn with(ending: &str, s: &str) -> String {
    s.replace('\n', ending)
}

fn id_of(document: &Document, title: &str) -> NodeId {
    let outline = document.outline();
    outline
        .root()
        .children
        .iter()
        .copied()
        .find(|&id| {
            let node = outline.node(id).unwrap();
            document.source()[node.heading_range.as_range()]
                .trim_start_matches('#')
                .trim()
                == title
        })
        .unwrap_or_else(|| panic!("no section {title:?} in {:?}", document.source()))
}

fn each_ending(check: impl Fn(&str)) {
    for ending in ["\n", "\r\n", "\r"] {
        check(ending);
    }
}

fn open(ending: &str) -> Document {
    Document::parse(with(ending, DOC)).unwrap()
}

#[test]
fn rename_replaces_the_title_and_nothing_of_the_line_ending() {
    each_ending(|e| {
        let mut d = open(e);
        let id = id_of(&d, "Bee");
        d.rename_section(id, "Wasp", d.revision()).unwrap();
        assert_eq!(
            d.source(),
            with(e, "# A\nbody a\n\n# Wasp\nbody b\n\n# C\nbody c\n"),
            "ending {e:?}"
        );
        d.undo().unwrap();
        assert_eq!(d.source(), with(e, DOC));
    });
}

#[test]
fn rename_a_setext_heading_keeps_its_underline() {
    each_ending(|e| {
        let source = with(e, "Bee\n---\nbody\n");
        let mut d = Document::parse(source).unwrap();
        let id = d.outline().root().children[0];
        d.rename_section(id, "Wasp", d.revision()).unwrap();
        assert_eq!(d.source(), with(e, "Wasp\n---\nbody\n"), "ending {e:?}");
    });
}

#[test]
fn join_removes_the_marker_and_keeps_the_line_ending() {
    each_ending(|e| {
        let mut d = open(e);
        let id = id_of(&d, "Bee");
        d.merge_with_prev_sibling(id, d.revision()).unwrap();
        assert_eq!(
            d.source(),
            with(e, "# A\nbody a\n\nBee\nbody b\n\n# C\nbody c\n"),
            "ending {e:?}"
        );
        d.undo().unwrap();
        assert_eq!(d.source(), with(e, DOC));
    });
}

#[test]
fn join_a_setext_heading() {
    each_ending(|e| {
        let source = with(e, "A\n===\nbody a\n\nBee\n===\nbody b\n");
        let mut d = Document::parse(source).unwrap();
        let id = id_of_setext(&d);
        d.merge_with_prev_sibling(id, d.revision()).unwrap();
        assert_eq!(
            d.source(),
            with(e, "A\n===\nbody a\n\nBee\nbody b\n"),
            "ending {e:?}"
        );
    });
}

fn id_of_setext(document: &Document) -> NodeId {
    document.outline().root().children[1]
}

#[test]
fn delete_removes_the_section_and_writes_no_foreign_ending() {
    each_ending(|e| {
        let mut d = open(e);
        let id = id_of(&d, "Bee");
        d.delete_section(id, d.revision()).unwrap();
        assert_eq!(
            d.source(),
            with(e, "# A\nbody a\n\n# C\nbody c\n"),
            "ending {e:?}"
        );
        d.undo().unwrap();
        assert_eq!(d.source(), with(e, DOC));
    });
}

#[test]
fn split_inserts_a_heading_in_the_documents_ending() {
    each_ending(|e| {
        let mut d = open(e);
        let id = id_of(&d, "A");
        // After "body a" and its line break.
        let offset = with(e, "body a\n").len();
        d.split_section(id, offset, "Mid", crate::HeadingLevel::H2, d.revision())
            .unwrap();
        let after = d.source().to_string();
        assert!(
            after.contains(&with(e, "## Mid\n")),
            "ending {e:?}: {after:?}"
        );
        match e {
            "\r" => assert!(
                !after.contains('\n'),
                "an LF was written into a CR file: {after:?}"
            ),
            "\n" => assert!(!after.contains('\r')),
            _ => assert_eq!(after.matches('\n').count(), after.matches("\r\n").count()),
        }
        d.undo().unwrap();
        assert_eq!(d.source(), with(e, DOC));
    });
}
