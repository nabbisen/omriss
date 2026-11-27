//! RFC-085 §3.3 item 4, level 2: the breadcrumb follows the structure's parents,
//! in every format. Markdown, valid JSON, an empty JSON container, an unreadable
//! `.json` (nothing to focus: no crumbs).

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::Locale;
use omriss_ui::{DocumentMapNode, EditorSession, FileTextProfile, node_id_from_raw};

use super::Breadcrumb;
use crate::test_texts::texts_of;

const MD: &str = "# One\nbody\n\n## Child\nx\n";
const JSON: &str = "{\"nested\": {\"k\": 1}, \"list\": [true]}\n";

thread_local! {
    static CASE: std::cell::Cell<(&'static str, DocumentFormat, &'static [&'static str], Locale)> =
        const { std::cell::Cell::new(("", DocumentFormat::Markdown, &[], Locale::En)) };
}

/// Opens `source` and focuses, one by one, the node titled each of `focus`
/// (each looked up among the children of the one before).
fn host() -> Element {
    let (source, format, focus, locale) = CASE.with(|c| c.get());
    let mut session = use_signal(|| {
        EditorSession::open_detected(
            source.to_string(),
            None,
            FileTextProfile::detect(source, false),
            format,
        )
        .unwrap()
    });
    use_hook(|| {
        for title in focus {
            fn find(node: &DocumentMapNode, title: &str) -> Option<u64> {
                node.children
                    .iter()
                    .find(|c| c.title == title)
                    .map(|c| c.id)
            }
            // Walk down from the node currently focused (or the root).
            let map = session.read().document_map_nodes();
            let from = session
                .read()
                .focused_path()
                .last()
                .map(|p| p.id.0)
                .unwrap_or(map.id);
            fn locate(node: &DocumentMapNode, id: u64) -> Option<&DocumentMapNode> {
                if node.id == id {
                    return Some(node);
                }
                node.children.iter().find_map(|c| locate(c, id))
            }
            let parent = locate(&map, from).unwrap();
            let id = find(parent, title).unwrap_or_else(|| panic!("no {title:?}"));
            session.write().focus(node_id_from_raw(id)).unwrap();
        }
    });
    rsx! {
        Breadcrumb {
            session,
            locale: use_signal(move || locale),
            draft: use_signal(String::new),
            status: use_signal(String::new),
        }
    }
}

fn crumbs(
    source: &'static str,
    format: DocumentFormat,
    focus: &'static [&'static str],
    locale: Locale,
) -> Vec<String> {
    CASE.with(|c| c.set((source, format, focus, locale)));
    texts_of(host)
        .into_iter()
        .filter(|t| !t.trim().is_empty() && t.trim() != "›" && t != "…" && t != "↑")
        .collect()
}

#[test]
fn markdown_crumbs_are_unchanged() {
    let t = crumbs(MD, DocumentFormat::Markdown, &["One", "Child"], Locale::En);
    assert_eq!(t, ["Document", "One", "Child"], "{t:?}");
}

#[test]
fn json_crumbs_follow_the_structures_parents() {
    let t = crumbs(JSON, DocumentFormat::Json, &["nested", "k"], Locale::En);
    assert_eq!(t, ["Document", "nested", "k"], "{t:?}");
    let t = crumbs(JSON, DocumentFormat::Json, &["list", "1"], Locale::En);
    assert_eq!(t, ["Document", "list", "Item 1"], "{t:?}");
}

#[test]
fn a_top_level_json_node_has_the_whole_document_above_it() {
    let t = crumbs(JSON, DocumentFormat::Json, &["nested"], Locale::En);
    assert_eq!(t, ["Document", "nested"], "{t:?}");
}

#[test]
fn the_whole_document_crumb_is_localized() {
    let t = crumbs(JSON, DocumentFormat::Json, &["nested"], Locale::Ja);
    assert_eq!(t[0], omriss_ui::i18n::t(Locale::Ja, "breadcrumb.root"));
}

#[test]
fn with_nothing_focused_or_no_structure_there_are_no_crumbs() {
    assert!(crumbs(JSON, DocumentFormat::Json, &[], Locale::En).is_empty());
    assert!(crumbs("{}\n", DocumentFormat::Json, &[], Locale::En).is_empty());
    assert!(crumbs("{\"a\": ", DocumentFormat::Json, &[], Locale::En).is_empty());
}

#[test]
fn a_list_members_crumb_is_in_the_users_language() {
    let en = crumbs(JSON, DocumentFormat::Json, &["list", "1"], Locale::En);
    assert_eq!(en, ["Document", "list", "Item 1"], "{en:?}");
    let ja = crumbs(JSON, DocumentFormat::Json, &["list", "1"], Locale::Ja);
    assert_eq!(ja[2], "項目 1", "{ja:?}");
    assert_eq!(ja[1], "list", "a key is shown as written");
}
