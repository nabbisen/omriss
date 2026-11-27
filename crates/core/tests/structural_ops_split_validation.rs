//! RFC-070 §4.1 and §8: `split_section` takes the title rule `rename_section`
//! has, and refuses an offset that cuts a character or one line ending in two.

use omriss_core::{Document, HeadingLevel, StructuralEditError};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

/// Titles and what `rename_section` did with each *before* RFC-070 moved its check
/// (run on the code at `eae0d5e`-era `rename.rs`, which this table was written
/// from): `Some(written)` for an accepted one, `None` for `InvalidTitle`.
const TITLES: &[(&str, Option<&str>)] = &[
    ("New", Some("New")),
    ("  padded  ", Some("padded")),
    ("\tTabbed\t", Some("Tabbed")),
    ("two words", Some("two words")),
    ("日本語", Some("日本語")),
    ("# looks like a marker", Some("# looks like a marker")),
    ("", None),
    ("   ", None),
    ("\t \t", None),
    ("a\nb", None),
    ("a\r\nb", None),
    ("a\rb", None),
    ("Evil\n# Injected H1\n\ntext", None),
    ("\n", None),
    ("trailing\n", Some("trailing")), // trim removes an outer line break first
    ("\n leading", Some("leading")),
];

#[test]
fn rename_accepts_and_refuses_the_titles_it_did_before() {
    for (title, want) in TITLES {
        let mut d = doc("# A\nbody\n");
        let a = d.outline().root().children[0];
        let r = d.rename_section(a, title, d.revision());
        match want {
            Some(written) => {
                r.unwrap_or_else(|e| panic!("{title:?}: {e:?}"));
                assert_eq!(d.source(), format!("# {written}\nbody\n"), "{title:?}");
            }
            None => {
                assert_eq!(
                    r.unwrap_err(),
                    StructuralEditError::InvalidTitle,
                    "{title:?}"
                );
                assert_eq!(d.source(), "# A\nbody\n", "{title:?}: nothing written");
            }
        }
    }
}

#[test]
fn split_refuses_every_title_rename_refuses_and_writes_nothing() {
    for (title, want) in TITLES.iter().filter(|(_, w)| w.is_none()) {
        assert!(want.is_none());
        let mut d = doc("# A\nbody\n");
        let a = d.outline().root().children[0];
        let rev = d.revision();
        let err = d
            .split_section(a, 0, title, HeadingLevel::H2, rev)
            .unwrap_err();
        assert_eq!(err, StructuralEditError::InvalidTitle, "{title:?}");
        assert_eq!(d.source(), "# A\nbody\n", "{title:?}: nothing written");
        assert_eq!(d.revision(), rev, "{title:?}: no new revision");
    }
}

#[test]
fn split_writes_the_title_rename_would_write() {
    for (title, want) in TITLES.iter().filter_map(|(t, w)| w.map(|w| (t, w))) {
        let mut d = doc("# A\nbody\n");
        let a = d.outline().root().children[0];
        d.split_section(a, 0, title, HeadingLevel::H2, d.revision())
            .unwrap_or_else(|e| panic!("{title:?}: {e:?}"));
        assert!(
            d.source().contains(&format!("\n## {want}\n")),
            "{title:?}: {:?}",
            d.source()
        );
    }
}

#[test]
fn an_injected_heading_in_the_title_adds_no_outline_node() {
    let mut d = doc("# A\nbody\n");
    let a = d.outline().root().children[0];
    let before = d.outline().len();
    assert!(
        d.split_section(
            a,
            0,
            "Evil\n# Injected H1\n\ntext",
            HeadingLevel::H2,
            d.revision()
        )
        .is_err()
    );
    assert_eq!(d.outline().len(), before);
    assert_eq!(d.source(), "# A\nbody\n");
}

#[test]
fn an_empty_title_no_longer_makes_a_heading_with_a_trailing_space() {
    let mut d = doc("# A\nbody\n");
    let a = d.outline().root().children[0];
    assert_eq!(
        d.split_section(a, 0, "", HeadingLevel::H2, d.revision())
            .unwrap_err(),
        StructuralEditError::InvalidTitle
    );
}

// ── Offsets (RFC-070 §8.3) ────────────────────────────────────────────────

#[test]
fn an_offset_inside_a_character_is_refused_and_does_not_panic() {
    // Measured on the code before RFC-070: `split_section` already refuses it with
    // `InvalidSplitOffset` (the `is_char_boundary` check was there); this pins it.
    let mut d = doc("# A\n日本語\n");
    let a = d.outline().root().children[0];
    for inside in [1, 2, 4, 5] {
        let err = d
            .split_section(a, inside, "X", HeadingLevel::H2, d.revision())
            .unwrap_err();
        assert_eq!(
            err,
            StructuralEditError::InvalidSplitOffset,
            "offset {inside}"
        );
    }
    assert_eq!(d.source(), "# A\n日本語\n");
    // On a boundary it splits.
    d.split_section(a, 3, "X", HeadingLevel::H2, d.revision())
        .unwrap();
    assert!(d.source().contains("日\n## X\n"), "{:?}", d.source());
}

#[test]
fn an_offset_between_cr_and_lf_is_refused() {
    let src = "# A\r\none\r\ntwo\r\n";
    let mut d = doc(src);
    let a = d.outline().root().children[0];
    // Body is "one\r\ntwo\r\n"; byte 4 is between the CR and the LF after "one".
    let err = d
        .split_section(a, 4, "X", HeadingLevel::H2, d.revision())
        .unwrap_err();
    assert_eq!(err, StructuralEditError::InvalidSplitOffset);
    assert_eq!(d.source(), src);
    // Before the CR, and after the LF, both split.
    for ok in [3, 5] {
        let mut d = doc(src);
        let a = d.outline().root().children[0];
        d.split_section(a, ok, "X", HeadingLevel::H2, d.revision())
            .unwrap_or_else(|e| panic!("offset {ok}: {e:?}"));
    }
}

#[test]
fn a_lone_cr_or_lone_lf_is_not_a_pair() {
    // A lone CR followed by text, and a lone LF preceded by text: splitting there
    // cuts no line ending in two.
    for src in ["# A\rone\rtwo\r", "# A\none\ntwo\n"] {
        let mut d = doc(src);
        let a = d.outline().root().children[0];
        d.split_section(a, 4, "X", HeadingLevel::H2, d.revision())
            .unwrap_or_else(|e| panic!("{src:?}: {e:?}"));
    }
}
