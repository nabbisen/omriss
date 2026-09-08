//! Golden integration tests (RFC-005..008, RFC-044).
//!
//! For every fixture document and every section in it, replacing that
//! section's body must leave each byte outside the replaced range untouched,
//! and a subsequent undo must restore the original source byte-for-byte.

use omriss_core::{Document, NodeId, ReplaceSectionBody};

const FIXTURES: &[(&str, &str)] = &[
    ("nested_atx.md", include_str!("fixtures/nested_atx.md")),
    (
        "duplicate_titles.md",
        include_str!("fixtures/duplicate_titles.md"),
    ),
    ("japanese.md", include_str!("fixtures/japanese.md")),
    ("code_fences.md", include_str!("fixtures/code_fences.md")),
    (
        "skipped_levels.md",
        include_str!("fixtures/skipped_levels.md"),
    ),
    (
        "no_trailing_newline.md",
        include_str!("fixtures/no_trailing_newline.md"),
    ),
    ("setext.md", include_str!("fixtures/setext.md")),
    (
        "yaml_front_matter.md",
        include_str!("fixtures/yaml_front_matter.md"),
    ),
    (
        "toml_front_matter.md",
        include_str!("fixtures/toml_front_matter.md"),
    ),
    ("crlf.md", include_str!("fixtures/crlf.md")),
    ("html_content.md", include_str!("fixtures/html_content.md")),
    ("empty_bodies.md", include_str!("fixtures/empty_bodies.md")),
    ("no_headings.md", include_str!("fixtures/no_headings.md")),
];

/// Bodies whose replacement must not introduce new structure, so the outline
/// shape (and thus every untouched byte) can be compared mechanically.
const MARKER: &str = "REPLACED-BODY-MARKER\n";

fn node_ids(doc: &Document) -> Vec<NodeId> {
    doc.outline().iter().map(|node| node.id).collect()
}

/// Every non-root title, as a multiset (order-independent, duplicate
/// titles counted). A body-only edit must never change this — if it does,
/// some heading was corrupted or destroyed by the edit, even if the
/// edited range's own bytes look correct in isolation.
fn title_multiset(doc: &Document) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for node in doc.outline().iter().filter(|n| !n.is_root()) {
        *counts.entry(node.title.clone()).or_insert(0) += 1;
    }
    counts
}

#[test]
fn every_fixture_indexes_with_valid_invariants() {
    for (name, source) in FIXTURES {
        let doc = Document::parse((*source).to_string())
            .unwrap_or_else(|err| panic!("{name}: parse failed: {err}"));
        doc.outline()
            .validate(doc.source())
            .unwrap_or_else(|err| panic!("{name}: invariants violated: {err}"));
        assert_eq!(doc.source(), *source, "{name}: source must be verbatim");
    }
}

#[test]
fn replacing_any_body_preserves_all_unrelated_bytes() {
    for (name, source) in FIXTURES {
        let pristine = Document::parse((*source).to_string()).unwrap();
        let before_titles = title_multiset(&pristine);
        for id in node_ids(&pristine) {
            let mut doc = Document::parse((*source).to_string()).unwrap();
            let before = doc.source().to_string();
            let result = doc
                .replace_section_body(ReplaceSectionBody {
                    node_id: id,
                    base_revision: doc.revision(),
                    new_body: MARKER.to_string(),
                })
                .unwrap_or_else(|err| panic!("{name}: edit on {id:?} failed: {err}"));

            let replaced = result.replaced_range;
            let after = doc.source();

            // Prefix bytes are untouched.
            assert_eq!(
                &after[..replaced.start],
                &before[..replaced.start],
                "{name}: prefix changed when editing {id:?}"
            );
            // Suffix bytes are untouched (shifted by the length delta).
            assert_eq!(
                &after[result.new_range.end..],
                &before[replaced.end..],
                "{name}: suffix changed when editing {id:?}"
            );
            // The replaced span holds the marker verbatim, optionally
            // padded on either side by *only* line-break characters — the
            // minimum separator RFC-004's 2026-09-01 amendment (RFC-065)
            // authorises core to insert at a body boundary so the marker
            // can never weld onto an adjacent heading. Anything else in
            // this span would mean the marker was not stored verbatim.
            let region = &after[result.new_range.as_range()];
            let marker_at = region.find(MARKER).unwrap_or_else(|| {
                panic!("{name}: marker not found verbatim in {id:?}'s replaced region: {region:?}")
            });
            let (prefix, rest) = region.split_at(marker_at);
            let suffix = &rest[MARKER.len()..];
            assert!(
                prefix.chars().all(|c| c == '\n' || c == '\r'),
                "{name}: bytes before the marker must be only line breaks for {id:?}: {region:?}"
            );
            assert!(
                suffix.chars().all(|c| c == '\n' || c == '\r'),
                "{name}: bytes after the marker must be only line breaks for {id:?}: {region:?}"
            );

            // The title multiset must be unchanged: a body-only edit must
            // never corrupt, merge, or destroy any heading, including ones
            // far from the edited range. This is the check the pre-RFC-065
            // version of this test lacked — it verified only byte
            // positions, which stayed internally consistent even while
            // `setext.md`'s following heading was silently flattened into
            // the marker's own paragraph. See RFC-004 §Whitespace Policy's
            // amendment and RFC-065 §2.1/§2.6 for the full history.
            assert_eq!(
                title_multiset(&doc),
                before_titles,
                "{name}: title multiset changed when editing {id:?} — a heading was corrupted or destroyed by this body-only edit"
            );
        }
    }
}

#[test]
fn undo_after_each_edit_restores_the_original_bytes() {
    for (name, source) in FIXTURES {
        let pristine = Document::parse((*source).to_string()).unwrap();
        for id in node_ids(&pristine) {
            let mut doc = Document::parse((*source).to_string()).unwrap();
            doc.replace_section_body(ReplaceSectionBody {
                node_id: id,
                base_revision: doc.revision(),
                new_body: MARKER.to_string(),
            })
            .unwrap();
            doc.undo()
                .unwrap_or_else(|err| panic!("{name}: undo on {id:?} failed: {err}"));
            assert_eq!(
                doc.source(),
                *source,
                "{name}: undo did not round-trip for {id:?}"
            );
        }
    }
}

#[test]
fn redo_after_undo_reapplies_the_exact_edit() {
    for (name, source) in FIXTURES {
        let pristine = Document::parse((*source).to_string()).unwrap();
        for id in node_ids(&pristine) {
            let mut doc = Document::parse((*source).to_string()).unwrap();
            doc.replace_section_body(ReplaceSectionBody {
                node_id: id,
                base_revision: doc.revision(),
                new_body: MARKER.to_string(),
            })
            .unwrap();
            let edited = doc.source().to_string();
            doc.undo().unwrap();
            doc.redo()
                .unwrap_or_else(|err| panic!("{name}: redo on {id:?} failed: {err}"));
            assert_eq!(
                doc.source(),
                edited,
                "{name}: redo did not restore edited source for {id:?}"
            );
        }
    }
}

#[test]
fn sequential_edits_to_every_section_then_full_unwind_round_trips() {
    for (name, source) in FIXTURES {
        let mut doc = Document::parse((*source).to_string()).unwrap();
        let mut edits = 0usize;
        loop {
            // Re-read ids each round: ranges shift after every edit.
            let ids = node_ids(&doc);
            let Some(&id) = ids.get(edits) else { break };
            doc.replace_section_body(ReplaceSectionBody {
                node_id: id,
                base_revision: doc.revision(),
                new_body: MARKER.to_string(),
            })
            .unwrap_or_else(|err| panic!("{name}: sequential edit failed: {err}"));
            edits += 1;
        }
        for _ in 0..edits {
            doc.undo().unwrap();
        }
        assert_eq!(
            doc.source(),
            *source,
            "{name}: full unwind did not restore original"
        );
        assert!(!doc.can_undo(), "{name}: history should be exhausted");
    }
}

// ── RFC-040 regression tests ─────────────────────────────────────────────

/// Empty document (no content at all) must not panic.
#[test]
fn empty_document_round_trip() {
    let d = Document::parse("".to_string()).unwrap();
    let original = d.source().to_string();
    // Root has no children — nothing to edit; just verify stability.
    assert!(d.outline().root().children.is_empty());
    assert_eq!(d.source(), original);
}

/// Document with only whitespace and no headings must be navigable.
#[test]
fn whitespace_only_document() {
    let src = "   \n\n   \n";
    let d = Document::parse(src.to_string()).unwrap();
    assert!(d.outline().root().children.is_empty());
    assert_eq!(d.source(), src);
}

/// Body edit of the LAST section must not touch the first section.
#[test]
fn edit_last_section_preserves_first() {
    let src = "# First\nbody one\n\n# Last\nbody two\n";
    let mut d = Document::parse(src.to_string()).unwrap();
    let first_id = d.outline().root().children[0];
    let last_id = *d.outline().root().children.last().unwrap();
    let first_range = d.outline().node(first_id).unwrap().full_range;
    let original_first = src[first_range.as_range()].to_string();
    d.replace_section_body(ReplaceSectionBody {
        node_id: last_id,
        base_revision: d.revision(),
        new_body: "replaced\n".to_string(),
    })
    .unwrap();
    let new_first = &d.source()[first_range.as_range()];
    assert_eq!(new_first, original_first.as_str());
}

/// UTF-8 multibyte body edit must produce valid UTF-8.
#[test]
fn utf8_multibyte_body_edit() {
    let src = "# 日本語\n元の本文\n\n# End\n";
    let mut d = Document::parse(src.to_string()).unwrap();
    let id = d.outline().root().children[0];
    d.replace_section_body(ReplaceSectionBody {
        node_id: id,
        base_revision: d.revision(),
        new_body: "新しい本文\n".to_string(),
    })
    .unwrap();
    assert!(std::str::from_utf8(d.source().as_bytes()).is_ok());
    assert!(d.source().contains("# End\n"));
}
