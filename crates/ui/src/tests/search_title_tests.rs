//! RFC-077: search covers titles — a Markdown heading's title and a JSON object
//! member's key — as one more kind of result on the one search walk (RFC-085).

use omriss_core::DocumentFormat;

use crate::file::file_profile::FileTextProfile;
use crate::session::EditorSession;
use crate::{NodeItem, SearchMatch, SearchMatchKind};

fn md(source: &str) -> EditorSession {
    EditorSession::open(source.to_string(), None).unwrap()
}

fn json(source: &str) -> EditorSession {
    EditorSession::open_detected(
        source.to_string(),
        None,
        FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap()
}

fn kinds(hits: &[SearchMatch]) -> Vec<SearchMatchKind> {
    hits.iter().map(|h| h.kind).collect()
}

fn last_title(hit: &SearchMatch) -> &str {
    hit.path
        .last()
        .map(|p: &NodeItem| p.title.as_str())
        .unwrap()
}

use SearchMatchKind::{Heading, Key, Text};

// ── Markdown ────────────────────────────────────────────────────────────────

#[test]
fn a_heading_the_map_shows_is_found_and_opens_that_section() {
    let mut s = md("# Alpha\nbody\n\n## Gamma ray\nx\n\n# Delta\ny\n");
    let hits = s.search_document("gamma");
    assert_eq!(kinds(&hits), [Heading]);
    assert_eq!(last_title(&hits[0]), "Gamma ray");
    // No offset inside a body: the range is where the section starts, empty.
    assert!(hits[0].range.start == hits[0].range.end);
    // It opens that section.
    s.focus(hits[0].containing_node).unwrap();
    assert_eq!(s.focused_path().last().unwrap().title, "Gamma ray");
}

#[test]
fn heading_and_body_in_one_section_are_two_results_and_the_title_is_first() {
    let s = md("# Fox\nthe fox runs\n\n# Other\nno\n");
    let hits = s.search_document("fox");
    assert_eq!(kinds(&hits), [Heading, Text]);
    assert_eq!(hits[0].containing_node, hits[1].containing_node);
}

#[test]
fn results_follow_the_document() {
    let s = md("# One\nx one\n\n## One more\nnothing\n\n# Two\nx one\n");
    let hits = s.search_document("one");
    let order: Vec<_> = hits.iter().map(|h| (h.kind, last_title(h))).collect();
    assert_eq!(
        order,
        [
            (Heading, "One"),
            (Text, "One"),
            (Heading, "One more"),
            (Text, "Two"),
        ]
    );
}

#[test]
fn a_setext_underline_is_not_title_text() {
    let s = md("Title\n-----\nbody\n\nOther\n=====\nmore\n");
    assert!(s.search_document("---").is_empty(), "no setext underline");
    assert!(s.search_document("===").is_empty());
    let hits = s.search_document("title");
    assert_eq!(kinds(&hits), [Heading]);
}

#[test]
fn case_follows_the_body_rule() {
    let s = md("# Straße ÜBER\nplain\n");
    assert_eq!(kinds(&s.search_document("über")), [Heading]);
    assert_eq!(kinds(&s.search_document("ÜBER")), [Heading]);
    assert_eq!(
        kinds(&s.search_document("straSSE")),
        [],
        "exactly the body rule: no folding of ß"
    );
}

#[test]
fn the_body_results_of_a_document_are_what_they_were() {
    // No heading contains the query: the same matches, order and targets as before titles.
    let s = md("# A\nthe quick brown fox\n\n# B\nfox jumps\n");
    let hits = s.search_document("fox");
    assert_eq!(kinds(&hits), [Text, Text]);
    assert_eq!(last_title(&hits[0]), "A");
    assert_eq!(last_title(&hits[1]), "B");
}

#[test]
fn the_section_scope_includes_the_focused_title_and_the_titles_below() {
    let mut s = md("# Part\ntext\n\n## Part two\nx\n\n# Outside part\ny\n");
    let part = s.outline_items()[0].id;
    s.focus(part).unwrap();
    let hits = s.search_section("part");
    let order: Vec<_> = hits.iter().map(|h| (h.kind, last_title(h))).collect();
    assert_eq!(
        order,
        [(Heading, "Part"), (Heading, "Part two")],
        "{order:?}"
    );
}

// ── JSON ────────────────────────────────────────────────────────────────────

#[test]
fn a_json_key_is_found_and_opens_its_node() {
    let mut s = json("{\"name\": \"x\", \"nested\": {\"deep\": 1}}\n");
    let hits = s.search_document("deep");
    assert_eq!(kinds(&hits), [Key]);
    assert_eq!(
        hits[0]
            .path
            .iter()
            .map(|p| p.title.as_str())
            .collect::<Vec<_>>(),
        ["", "nested", "deep"]
    );
    s.focus(hits[0].containing_node).unwrap();
    assert_eq!(s.focused_path().last().unwrap().title, "deep");
}

#[test]
fn a_lists_positional_labels_are_not_searchable() {
    let s = json("{\"list\": [\"x\", \"y\", {\"k\": 1}]}\n");
    assert!(
        s.search_document("Item").is_empty(),
        "the word the app makes up"
    );
    assert!(
        s.search_document("1").iter().all(|h| h.kind != Key),
        "an index is not a key"
    );
    assert!(s.search_document("2").is_empty());
    // A key inside a list member is a key from the file: found.
    assert_eq!(kinds(&s.search_document("k")), [Key]);
    // And the list's own key.
    assert_eq!(kinds(&s.search_document("list")), [Key]);
}

#[test]
fn a_root_list_has_no_searchable_titles_at_all() {
    let s = json("[\"a\", \"b\"]\n");
    assert!(s.search_document("Item").is_empty());
}

#[test]
fn a_key_and_its_value_are_two_results_and_both_open_the_node() {
    let s = json("{\"name\": \"name of it\"}\n");
    let hits = s.search_document("name");
    assert_eq!(kinds(&hits), [Key, Text]);
    assert_eq!(hits[0].containing_node, hits[1].containing_node);
}

#[test]
fn a_decoded_key_is_what_the_map_shows() {
    let s = json("{\"caf\\u00e9\": 1}\n");
    assert_eq!(kinds(&s.search_document("café")), [Key]);
    assert!(s.search_document("u00e9").is_empty());
}

#[test]
fn the_json_section_scope_includes_the_focused_key_and_the_keys_below() {
    let mut s = json("{\"outer\": {\"inner\": {\"leaf\": \"v\"}}, \"other\": 1}\n");
    let outer = s.outline_items()[0].id;
    s.focus(outer).unwrap();
    let hits = s.search_section("e");
    let order: Vec<_> = hits.iter().map(|h| (h.kind, last_title(h))).collect();
    assert_eq!(
        order,
        [(Key, "outer"), (Key, "inner"), (Key, "leaf")],
        "'other' and its value are outside the focused node: {order:?}"
    );
}

#[test]
fn nothing_is_a_title_in_a_file_that_cannot_be_read() {
    let s = json("{\"a\": ");
    assert!(s.search_document("a").iter().all(|h| h.kind == Text));
}

#[test]
fn titles_are_cached_with_the_texts_by_revision() {
    use std::rc::Rc;
    let mut s = json("{\"name\": \"a\", \"other\": \"b\"}\n");
    let before = s.pane_texts();
    for _ in 0..3 {
        let _ = s.search_document("name");
        let _ = s.search_section("name");
    }
    assert!(Rc::ptr_eq(&before, &s.pane_texts()));
    let id = s.outline_items()[0].id;
    s.focus(id).unwrap();
    s.commit_structured_draft("changed").unwrap();
    assert!(!Rc::ptr_eq(&before, &s.pane_texts()));
    assert_eq!(
        kinds(&s.search_document("name")),
        [Key],
        "the key is still found after the edit"
    );
}
