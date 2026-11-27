//! RFC-085 §3.3: every pane reads the document's structure, in every format.
//! The surfaces that used to read the Markdown heading outline of a JSON file's
//! text — cards, keys, breadcrumb, Esc and Up, search, statistics — on a JSON
//! fixture: each result's path and target, the counts.

use omriss_core::{DocumentFormat, NodeId, StructureNodeKind, ValueKind};

use crate::editor::stats::StatsUnit;
use crate::editor::view_state::ViewMode;
use crate::file::file_profile::FileTextProfile;
use crate::session::EditorSession;

const JSON: &str = r#"{
  "name": "omriss",
  "tags": ["alpha", "beta"],
  "n": 3,
  "on": true,
  "none": null,
  "nested": {"deep": "needle here", "other": "no hit"}
}
"#;

fn json(source: &str) -> EditorSession {
    EditorSession::open_detected(
        source.to_string(),
        Some("x.json".into()),
        FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap()
}

fn top(s: &EditorSession, title: &str) -> NodeId {
    s.outline_items()
        .into_iter()
        .find(|i| i.title == title)
        .unwrap_or_else(|| panic!("no top-level {title:?}"))
        .id
}

fn titles(items: &[crate::NodeItem]) -> Vec<&str> {
    items.iter().map(|i| i.title.as_str()).collect()
}

// ── cards and keys ──────────────────────────────────────────────────────────

#[test]
fn the_overview_lists_the_top_level_members_not_one_document_card() {
    let s = json(JSON);
    let items = s.current_children();
    assert_eq!(
        titles(&items),
        ["name", "tags", "n", "on", "none", "nested"]
    );
    // The label each card carries: what a value is, or its item count.
    let kinds: Vec<_> = items
        .iter()
        .map(|i| (i.kind, i.value_kind, i.child_count))
        .collect();
    assert_eq!(
        kinds,
        [
            (StructureNodeKind::Value, Some(ValueKind::Text), 0),
            (StructureNodeKind::List, None, 2),
            (StructureNodeKind::Value, Some(ValueKind::Number), 0),
            (StructureNodeKind::Value, Some(ValueKind::OnOff), 0),
            (StructureNodeKind::Value, Some(ValueKind::NoValue), 0),
            (StructureNodeKind::Group, None, 2),
        ]
    );
    assert!(items.iter().all(|i| i.level.is_none() && !i.is_root));
}

#[test]
fn inside_a_node_the_cards_are_its_children() {
    let mut s = json(JSON);
    let nested = top(&s, "nested");
    s.focus(nested).unwrap();
    assert_eq!(titles(&s.current_children()), ["deep", "other"]);
    let tags = top(&s, "tags");
    s.focus(tags).unwrap();
    assert_eq!(titles(&s.current_children()), ["Item 1", "Item 2"]);
    // A value has no children.
    let n = top(&s, "n");
    s.focus(n).unwrap();
    assert!(s.current_children().is_empty());
}

#[test]
fn a_json_container_with_no_members_is_one_card_for_the_whole_document() {
    for source in ["{}\n", "[]\n"] {
        let s = json(source);
        let items = s.outline_items();
        assert_eq!(items.len(), 1, "{source:?}");
        assert!(items[0].is_root);
        assert_eq!(items[0].child_count, 0);
    }
}

#[test]
fn an_unreadable_json_file_has_the_whole_document_as_its_one_card() {
    let s = json("{\"a\": ");
    assert!(!s.structure_readable());
    let items = s.outline_items();
    assert_eq!(items.len(), 1);
    assert!(items[0].is_root);
}

// ── breadcrumb, Esc and Up ──────────────────────────────────────────────────

#[test]
fn the_breadcrumb_is_the_path_of_the_focused_node() {
    let mut s = json(JSON);
    assert!(s.focused_path().is_empty(), "nothing focused, no path");
    let nested = top(&s, "nested");
    s.focus(nested).unwrap();
    let deep = s.current_children()[0].id;
    s.focus(deep).unwrap();
    let path = s.focused_path();
    assert_eq!(titles(&path), ["", "nested", "deep"]);
    assert!(path[0].is_root, "the first crumb is the whole document");
    assert!(!path[1].is_root && !path[2].is_root);
}

#[test]
fn esc_climbs_one_level_and_a_top_level_node_goes_to_the_whole_document() {
    let mut s = json(JSON);
    let nested = top(&s, "nested");
    s.focus(nested).unwrap();
    let deep = s.current_children()[0].id;
    s.focus(deep).unwrap();

    s.zoom_out();
    assert_eq!(s.view_mode(), ViewMode::Focus(nested), "to the parent");
    s.zoom_out();
    assert_eq!(
        s.view_mode(),
        ViewMode::Outline,
        "a top-level node goes up to the overview"
    );
    s.zoom_out();
    assert_eq!(
        s.view_mode(),
        ViewMode::Outline,
        "nothing above the overview"
    );
}

// ── search ──────────────────────────────────────────────────────────────────

#[test]
fn search_finds_a_value_and_names_its_path_and_target() {
    let s = json(JSON);
    let hits = s.search_document("needle");
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];
    assert_eq!(titles(&hit.path), ["", "nested", "deep"]);
    assert_eq!(hit.containing_node, hit.path.last().unwrap().id);
    assert!(hit.preview.contains("needle here"));
    // The target opens that node, not the root.
    assert_ne!(
        hit.containing_node,
        s.document_map_nodes().id.into_node_id()
    );
}

#[test]
fn search_reads_the_decoded_text_of_a_value() {
    let s = json("{\"k\": \"caf\\u00e9\"}\n");
    assert_eq!(s.search_document("café").len(), 1);
    assert!(s.search_document("u00e9").is_empty());
}

#[test]
fn a_group_or_list_has_no_text_of_its_own_but_its_members_do() {
    let s = json(JSON);
    // "alpha" and "beta" are the list's two members; the list itself has none.
    let alpha = s.search_document("alpha");
    assert_eq!(alpha.len(), 1);
    assert_eq!(titles(&alpha[0].path), ["", "tags", "Item 1"]);
    assert_eq!(
        s.search_document("a")
            .iter()
            .filter(|h| h.path.len() == 2 && h.kind == crate::SearchMatchKind::Text)
            .count(),
        0
    );
}

#[test]
fn the_section_scope_is_the_focused_node_and_what_is_under_it() {
    let mut s = json(JSON);
    let nested = top(&s, "nested");
    s.focus(nested).unwrap();
    // "o" occurs in "omriss" (outside) and in "no hit" / "other"-values (inside).
    let inside = s.search_section("hit");
    assert_eq!(inside.len(), 1);
    assert_eq!(titles(&inside[0].path), ["", "nested", "other"]);
    assert!(
        s.search_section("omriss").is_empty(),
        "outside the focused node"
    );
    assert_eq!(s.search_document("omriss").len(), 1);
}

#[test]
fn the_markdown_search_is_unchanged_in_what_it_finds() {
    let s = EditorSession::open(
        "# A\nthe fox\n\n## B\nanother fox\n\n# C\nfox\n".into(),
        None,
    )
    .unwrap();
    let hits = s.search_document("fox");
    assert_eq!(hits.len(), 3);
    assert_eq!(titles(&hits[1].path), ["", "A", "B"]);
}

// ── statistics ──────────────────────────────────────────────────────────────

#[test]
fn json_statistics_count_values_and_the_words_of_their_text() {
    let s = json(JSON);
    let stats = s.stats();
    assert_eq!(stats.unit, StatsUnit::Values);
    // name, alpha, beta, n, on, none, deep, other: eight leaves.
    assert_eq!(stats.section_count, 8);
    // omriss(1) alpha(1) beta(1) 3(1) true(1) ""(0) "needle here"(2) "no hit"(2).
    assert_eq!(stats.total_words, 9);
    assert_eq!(stats.focused_words, 0);
}

#[test]
fn json_focused_words_cover_the_focused_node_and_what_is_under_it() {
    let mut s = json(JSON);
    let nested = top(&s, "nested");
    s.focus(nested).unwrap();
    assert_eq!(s.stats().focused_words, 4);
    let name = top(&s, "name");
    s.focus(name).unwrap();
    assert_eq!(s.stats().focused_words, 1);
}

#[test]
fn markdown_statistics_count_sections_as_before() {
    let s = EditorSession::open("# A\none two\n\n## A.1\nx\n\n# B\ny\n".into(), None).unwrap();
    let stats = s.stats();
    assert_eq!(stats.unit, StatsUnit::Sections);
    assert_eq!(stats.section_count, 3);
}

#[test]
fn the_statistics_messages_exist_in_both_locales_with_the_same_placeholders() {
    use crate::i18n::{Locale, t_args};
    for key in [
        "stats.message.sections",
        "stats.message.sections_focused",
        "stats.message.values",
        "stats.message.values_focused",
    ] {
        for locale in [Locale::En, Locale::Ja] {
            let out = t_args(
                locale,
                key,
                &[("focused", "11"), ("words", "22"), ("count", "33")],
            );
            assert!(out.contains("22") && out.contains("33"), "{key}: {out}");
            assert!(!out.contains('{'), "{key} left a placeholder: {out}");
            assert_eq!(
                out.contains("11"),
                key.ends_with("_focused"),
                "{key}: {out}"
            );
        }
    }
}

/// The map's root id is a raw `u64`; panes use `NodeId`.
trait IntoNodeId {
    fn into_node_id(self) -> NodeId;
}
impl IntoNodeId for u64 {
    fn into_node_id(self) -> NodeId {
        crate::interface::document_map::node_id_from_raw(self)
    }
}

// ── opening and empty states (RFC-085 §3.3 items 1–2, §3.4) ─────────────────

use crate::session::EmptyMap;

fn opened(source: &str, format: DocumentFormat) -> EditorSession {
    let mut s = EditorSession::open_detected(
        source.to_string(),
        Some("x".into()),
        FileTextProfile::detect(source, false),
        format,
    )
    .unwrap();
    s.settle_on_open();
    s
}

/// The Document Map's selected row and the Writing Area's node: the same node.
fn highlighted(s: &EditorSession) -> Option<u64> {
    fn find(n: &crate::DocumentMapNode) -> Option<u64> {
        if n.is_selected {
            return Some(n.id);
        }
        n.children.iter().find_map(find)
    }
    find(&s.document_map_nodes())
}

#[test]
fn a_valid_json_file_opens_on_its_first_member_and_the_map_highlights_that_row() {
    let s = opened(JSON, DocumentFormat::Json);
    let focused = match s.view_mode() {
        ViewMode::Focus(id) => id,
        other => panic!("expected a focused node, got {other:?}"),
    };
    assert_eq!(s.focused_path().last().unwrap().title, "name");
    assert_eq!(highlighted(&s), Some(focused.0));
    assert!(!s.is_raw());
}

#[test]
fn a_markdown_file_opens_as_before() {
    let s = opened("# One\nx\n\n# Two\ny\n", DocumentFormat::Markdown);
    assert_eq!(s.focused_path().last().unwrap().title, "One");
    let headingless = opened("just text\n", DocumentFormat::Markdown);
    // The whole document is one layer; the root is that layer.
    assert!(matches!(headingless.view_mode(), ViewMode::Focus(_)));
    assert_eq!(headingless.empty_map(), Some(EmptyMap::NoHeadings));
}

#[test]
fn a_json_container_with_no_members_opens_on_the_whole_document() {
    for source in ["{}\n", "[]\n"] {
        let s = opened(source, DocumentFormat::Json);
        assert_eq!(s.empty_map(), Some(EmptyMap::NoMembers), "{source:?}");
        assert!(!s.is_raw(), "readable: not plain file text");
        // The root is not a row, so nothing is highlighted.
        assert_eq!(
            highlighted(&s).filter(|&id| id != s.document_map_nodes().id),
            None
        );
    }
}

#[test]
fn an_unreadable_file_opens_in_plain_file_text_and_the_map_says_why() {
    for (source, format) in [
        ("{\"a\": ", DocumentFormat::Json),
        ("not json at all", DocumentFormat::Json),
        ("", DocumentFormat::Json),
        ("a = 1\n", DocumentFormat::Toml),
        ("a: 1\n", DocumentFormat::YamlExperimental),
        ("plain\n", DocumentFormat::PlainText),
    ] {
        let s = opened(source, format);
        assert!(s.is_raw(), "{format:?} {source:?} opens in plain file text");
        assert_eq!(s.empty_map(), Some(EmptyMap::Unreadable), "{format:?}");
        // Lossless: opening changed nothing, and nothing is pending.
        assert_eq!(s.source(), source);
        assert!(!s.is_dirty());
    }
}

#[test]
fn a_valid_json_file_is_never_told_to_add_headings() {
    for source in [JSON, "{}\n", "[]\n", "3\n", "\"x\"\n"] {
        let s = opened(source, DocumentFormat::Json);
        assert_ne!(s.empty_map(), Some(EmptyMap::NoHeadings), "{source:?}");
    }
    assert_eq!(opened(JSON, DocumentFormat::Json).empty_map(), None);
}

#[test]
fn the_empty_map_messages_exist_in_both_locales() {
    use crate::i18n::{Locale, t, t_args};
    for locale in [Locale::En, Locale::Ja] {
        for key in ["document_map.no_members", "document_map.plain_text"] {
            assert_ne!(t(locale, key), key, "{key} is in the catalog");
        }
        for key in ["document_map.unreadable", "document_map.unsupported_format"] {
            let out = t_args(locale, key, &[("format", "JSON")]);
            assert!(out.contains("JSON") && !out.contains('{'), "{key}: {out}");
        }
    }
    // The English sentence of RFC-085 §3.4, word for word.
    assert_eq!(
        t_args(Locale::En, "document_map.unreadable", &[("format", "JSON")]),
        "This file could not be read as JSON, so its structure cannot be shown. You can view \
         and edit its text in Plain file text. When it is valid JSON, its structure is shown."
    );
    assert_eq!(
        t(Locale::En, "document_map.no_members"),
        "This JSON has no keys or items to show."
    );
}

// ── node texts are cached by revision (RFC-085 §8.3) ────────────────────────

#[test]
fn a_draft_does_not_rebuild_the_node_texts_and_a_commit_does() {
    use std::rc::Rc;
    let mut s = json(JSON);
    let before = s.pane_texts();
    // Everything that happens while typing, and everything that reads: no new revision, no rebuild.
    let name = top(&s, "name");
    s.focus(name).unwrap();
    for typed in ["o", "om", "omr", "omriss!"] {
        let _ = s.structured_draft_differs(typed);
        let _ = s.structured_draft_state(typed);
        let _ = s.validate_structured_draft(typed);
        let _ = s.search_document("alpha");
        let _ = s.stats();
        let _ = s.current_children();
    }
    assert!(
        Rc::ptr_eq(&before, &s.pane_texts()),
        "typing and reading must not rebuild the node texts"
    );

    s.commit_structured_draft("changed").unwrap();
    let after = s.pane_texts();
    assert!(
        !Rc::ptr_eq(&before, &after),
        "a commit bumps the revision: rebuilt"
    );
    assert!(after.all.iter().any(|t| t.text == "changed"));
    assert!(
        s.search_document("changed").len() == 1,
        "search sees the committed text"
    );

    // Undo is a new revision too; the texts follow the source.
    s.undo().unwrap();
    assert!(!Rc::ptr_eq(&after, &s.pane_texts()));
    assert!(s.search_document("changed").is_empty());
}

#[test]
fn a_markdown_draft_does_not_rebuild_the_node_texts_and_a_commit_does() {
    use std::rc::Rc;
    let mut s = EditorSession::open("# A\none\n\n# B\ntwo\n".into(), None).unwrap();
    let a = s.outline_items()[0].id;
    let snapshot = s.focus(a).unwrap();
    let before = s.pane_texts();
    let _ = s.body_draft_differs("one more\n");
    let _ = s.search_section("one");
    let _ = s.stats();
    assert!(Rc::ptr_eq(&before, &s.pane_texts()));
    s.commit_focused_body(&snapshot, "changed\n\n".into())
        .unwrap();
    assert!(!Rc::ptr_eq(&before, &s.pane_texts()));
    assert_eq!(s.search_document("changed").len(), 1);
}

#[test]
fn an_unreadable_file_has_one_text_and_no_cache_to_go_stale() {
    let s = json("{\"a\": ");
    let texts = s.pane_texts();
    assert_eq!(texts.all.len(), 1);
    assert_eq!(texts.all[0].text, "{\"a\": ");
}
