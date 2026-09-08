//! RFC-065 regression tests: structural operation boundary integrity.
//!
//! New tests only — the `structural_ops*` and `source_preservation` golden
//! suites are protected and untouched by this file. Each test here targets
//! one of RFC-065 §2's defects directly, using the exact (or a minimally
//! adapted) shape from the RFC's own reproduction.
//!
//! Two of RFC-065's defects are **not** regression-tested here, and that is
//! itself the finding this task's review request reports in detail:
//!
//! - §2.1 (typing into a bare trailing heading) is fixed, but two protected
//!   tests assert the *pre-fix* behavior as correct
//!   (`crates/core/src/tests/edit_tests.rs::replacement_is_stored_verbatim_without_normalization`,
//!   `crates/core/tests/source_preservation.rs::replacing_any_body_preserves_all_unrelated_bytes`
//!   against `setext.md`). Per the handoff's own escalation trigger ("a
//!   golden test must change to pass"), those were not touched here.
//! - §2.5 (CRLF split — separate from the above) *is* tested below and
//!   passes.
//! - The `split_section` offset-validation defect this task's testing
//!   uncovered (inserting inside a descendant's own heading line) is
//!   reported, not fixed — out of RFC-065's six named defects.

use omriss_core::{Document, MoveTarget, NodeId, ReplaceSectionBody};

fn doc(md: &str) -> Document {
    Document::parse(md.to_string()).unwrap()
}

fn node_ids(d: &Document) -> Vec<NodeId> {
    d.outline().iter().map(|n| n.id).collect()
}

fn titles(d: &Document) -> Vec<String> {
    d.outline()
        .iter()
        .filter(|n| !n.is_root())
        .map(|n| n.title.clone())
        .collect()
}

// ── §2.1: typing into a bare trailing heading ───────────────────────────────

#[test]
fn typing_into_a_bare_trailing_heading_no_longer_writes_into_the_heading() {
    let path = format!(
        "{}/tests/fixtures/heading_only_no_trailing_newline.md",
        env!("CARGO_MANIFEST_DIR")
    );
    let src = std::fs::read_to_string(&path).unwrap();
    assert!(
        !src.ends_with('\n'),
        "fixture must have no trailing newline"
    );
    let mut d = doc(&src);

    let last_id = *d.outline().root().children.last().unwrap();
    assert_eq!(d.outline().node(last_id).unwrap().title, "Last");

    d.replace_section_body(ReplaceSectionBody {
        node_id: last_id,
        base_revision: d.revision(),
        new_body: "typed text".to_string(),
    })
    .unwrap();

    assert_eq!(
        d.outline().node(last_id).unwrap().title,
        "Last",
        "the heading title must not absorb the typed text"
    );
    assert_eq!(
        d.section_body(last_id).unwrap(),
        "typed text",
        "the body must read back exactly what was written"
    );
    assert!(
        d.source().contains("# Last"),
        "the heading marker must survive verbatim: {:?}",
        d.source()
    );
}

// ── §2.1 mirror case (found by RFC-066 P3): the right edge, via root ───────
//
// Per this task's own investigation: the root is reachable through the GUI
// (Overview pane's "Document" card) *only* when it has zero children, i.e.
// no headings at all — so this exact "root's body edit corrupts a following
// heading" shape is not reachable through the current GUI (there is no
// following heading to corrupt when root has none). It remains a real
// `omriss-core` public-API correctness issue, since `replace_section_body`
// has no such restriction. Documented here as a targeted regression test,
// not a GUI-reachable severity claim.

#[test]
fn editing_the_root_body_no_longer_destroys_the_following_heading() {
    let mut d = doc("intro\n\n# H\nbody\n");
    let root_id = d.outline().root_id();

    d.replace_section_body(ReplaceSectionBody {
        node_id: root_id,
        base_revision: d.revision(),
        new_body: "intro2".to_string(),
    })
    .unwrap();

    assert_eq!(
        titles(&d),
        vec!["H".to_string()],
        "the 'H' heading must survive"
    );
    assert!(
        d.source().contains("# H"),
        "the heading marker must survive verbatim: {:?}",
        d.source()
    );
}

/// The degenerate case: root has an *empty* body (the document starts
/// immediately with a heading) and is edited at offset 0 — the smallest
/// possible instance of the same defect, and RFC-066 P3's own shrunk
/// counterexample (`"a\n===", new_body="a"`).
///
/// **Not asserting exact body read-back here** — that is the escalation
/// this task's review request reports: `replace_section_body`'s necessary
/// right-edge separator becomes part of the freshly re-derived
/// `body_range` once the document is re-parsed, so `section_body` reads
/// back `new_body` plus that separator, not `new_body` alone. What *is*
/// asserted, and what matters for RFC-065 §2.1: the following heading
/// survives as a distinct, correctly-titled node.
#[test]
fn editing_an_empty_root_body_before_the_first_heading_preserves_that_heading() {
    let mut d = doc("A\n===");
    let root_id = d.outline().root_id();
    assert_eq!(
        d.outline().node(root_id).unwrap().body_range.len(),
        0,
        "root's body must be empty when the document starts immediately with a heading"
    );

    d.replace_section_body(ReplaceSectionBody {
        node_id: root_id,
        base_revision: d.revision(),
        new_body: "A".to_string(),
    })
    .unwrap();

    assert_eq!(
        titles(&d),
        vec!["A".to_string()],
        "the setext heading must survive as one node, not merge with the new root body"
    );
}

// ── §2.2: moving a section with no trailing newline ─────────────────────────

#[test]
fn moving_a_section_with_no_trailing_newline_no_longer_destroys_a_heading() {
    let mut d = doc("# A\nalpha\n# B\nbeta");
    let ids = node_ids(&d);
    let a = ids[1];
    let b = ids[2];

    d.move_section(b, MoveTarget::Before(a), d.revision())
        .unwrap();

    assert_eq!(
        titles(&d),
        vec!["B".to_string(), "A".to_string()],
        "both headings must survive the move, B now before A"
    );
}

// ── §2.3: promote on a root-parented section with a following sibling ──────

#[test]
fn promote_on_a_root_parented_section_with_a_following_sibling_stays_in_place() {
    let mut d = doc("## A\nalpha\n\n## B\nbeta\n");
    let ids = node_ids(&d);
    let a = ids[1];

    d.promote_section(a, d.revision()).unwrap();

    assert_eq!(
        titles(&d),
        vec!["A".to_string(), "B".to_string()],
        "A must stay before B, not relocate to end of file"
    );
    assert!(
        d.source().starts_with("# A\n"),
        "A must be promoted in place, at the start of the document: {:?}",
        d.source()
    );
}

/// A more severe variant this task's property testing found: relocating
/// into a position that collides with a setext underline used to lose
/// *two* nodes, not one.
#[test]
fn promote_whose_relocation_destination_touches_a_setext_underline_loses_no_nodes() {
    let mut d = doc("a\n===\n#### A\na\n---");
    let before = titles(&d);
    let ids = node_ids(&d);
    // The H4 "A" node: has a following sibling (the H2 "a" setext) under
    // the same parent, so promoting it triggers the relocation branch.
    let h4 = ids
        .iter()
        .copied()
        .find(|id| d.outline().node(*id).unwrap().title == "A")
        .unwrap();

    d.promote_section(h4, d.revision()).unwrap();

    assert_eq!(
        titles(&d).len(),
        before.len(),
        "promote must not change the total node count: before={before:?}, after={:?}",
        titles(&d)
    );
}

// ── §2.4: join preserves heading source ─────────────────────────────────────

#[test]
fn join_preserves_link_urls_code_spans_and_emphasis() {
    let mut d = doc("## One\nalpha\n\n## **bold** and [link](http://x) and `code`\nbeta\n");
    let ids = node_ids(&d);
    let second = ids[2];

    d.merge_with_prev_sibling(second, d.revision()).unwrap();

    assert!(
        d.source()
            .contains("**bold** and [link](http://x) and `code`"),
        "merge must preserve the heading's markup verbatim, not its flattened title: {:?}",
        d.source()
    );
    assert!(
        !d.source().contains("bold and link and code\n"),
        "merge must not have used the flattened plain-text title"
    );
}

#[test]
fn join_preserves_setext_heading_markup() {
    let mut d = doc("## One\nalpha\n\n[a link](http://x)\n---\nbeta\n");
    let ids = node_ids(&d);
    let second = ids[2];

    d.merge_with_prev_sibling(second, d.revision()).unwrap();

    assert!(
        d.source().contains("[a link](http://x)"),
        "merge must preserve the setext title's markup verbatim: {:?}",
        d.source()
    );
    assert!(
        !d.source().contains("==="),
        "the underline itself must be dropped, not preserved"
    );
}

// ── §2.5: inserted headings derive the newline from the document ──────────

#[test]
fn split_on_a_crlf_document_inserts_a_crlf_heading() {
    let mut d = doc("# A\r\nbody\r\n");
    let ids = node_ids(&d);
    let a = ids[1];
    let node = d.outline().node(a).unwrap();
    let offset = node.body_range.len(); // append at the end of A's body

    d.split_section(a, offset, "B", omriss_core::HeadingLevel::H1, d.revision())
        .unwrap();

    assert!(
        !d.source().contains("\n\n")
            || d.source().matches("\r\n").count() >= d.source().matches('\n').count(),
        "no bare LF should be introduced into a CRLF document: {:?}",
        d.source()
    );
    assert!(
        d.source().contains("\r\n# B\r\n\r\n"),
        "the inserted heading must use the document's own CRLF convention: {:?}",
        d.source()
    );
}

// ── §2.6: delete does not weld its neighbours ───────────────────────────────

#[test]
fn deleting_a_section_does_not_weld_its_neighbours_into_a_corrupted_heading() {
    // RFC-065 §2.6's exact reproduction.
    let mut d = doc("a\n===\n0\n## A\nA\n===");
    let ids = node_ids(&d);
    let atx_a = ids
        .iter()
        .copied()
        .find(|id| {
            let n = d.outline().node(*id).unwrap();
            n.level == Some(omriss_core::HeadingLevel::H2)
        })
        .unwrap();

    d.delete_section(atx_a, d.revision()).unwrap();

    let after = titles(&d);
    assert_eq!(
        after,
        vec!["a".to_string(), "A".to_string()],
        "the two remaining headings must stay distinct, not merge into \"0 A\": {after:?}"
    );
}

// ── Root reachability note (not a test — see review request) ───────────────
//
// This file deliberately does not assert anything about whether root is
// reachable through the GUI in a document *with* headings — it is not, per
// this task's own investigation of `crates/ui/src/session.rs::outline_items`
// and `crates/app/src/components/document_map_pane.rs`. That is a `crates/ui`/
// `crates/app` finding, out of this `crates/core`-only slice's file scope,
// and is reported in the review request instead.
