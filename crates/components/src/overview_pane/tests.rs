//! RFC-085 §3.3 item 3, level 2: the overview's cards, per format and per case.
//! Markdown, valid JSON, an empty JSON container and an unreadable `.json`.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::Locale;
use omriss_ui::{EditorSession, FileTextProfile};

use super::OverviewPane;
use crate::test_texts::texts_of;

const MD: &str = "# One\nbody\n\n## Child\nx\n\n# Two\ny\n";
const JSON: &str = "{\"name\": \"omriss\", \"tags\": [\"a\", \"b\"], \"n\": 3, \"on\": true, \"nested\": {\"k\": 1}}\n";

thread_local! {
    static CASE: std::cell::Cell<(&'static str, DocumentFormat, Locale)> =
        const { std::cell::Cell::new(("", DocumentFormat::Markdown, Locale::En)) };
}

fn host() -> Element {
    let (source, format, locale) = CASE.with(|c| c.get());
    let session = use_signal(|| {
        EditorSession::open_detected(
            source.to_string(),
            None,
            FileTextProfile::detect(source, false),
            format,
        )
        .unwrap()
    });
    rsx! {
        OverviewPane {
            session,
            locale: use_signal(move || locale),
            draft: use_signal(String::new),
            status: use_signal(String::new),
            selected_card: use_signal(|| 0usize),
        }
    }
}

fn cards(source: &'static str, format: DocumentFormat, locale: Locale) -> Vec<String> {
    CASE.with(|c| c.set((source, format, locale)));
    texts_of(host)
}

fn has(texts: &[String], wanted: &str) -> bool {
    texts.iter().any(|t| t == wanted)
}

#[test]
fn markdown_cards_are_unchanged() {
    let t = cards(MD, DocumentFormat::Markdown, Locale::En);
    for wanted in ["H1", "One", "Two", "1 subsections"] {
        assert!(has(&t, wanted), "{wanted:?} in {t:?}");
    }
    assert!(!has(&t, "Child"), "a child is not a top-level card: {t:?}");
}

#[test]
fn valid_json_shows_its_members_with_the_kind_of_each() {
    let t = cards(JSON, DocumentFormat::Json, Locale::En);
    for wanted in ["name", "tags", "n", "on", "nested"] {
        assert!(has(&t, wanted), "{wanted:?} in {t:?}");
    }
    // What each value is, and the item count of a list or group.
    for wanted in ["Text", "Number", "On/Off", "2 items", "1 item"] {
        assert!(has(&t, wanted), "{wanted:?} in {t:?}");
    }
    assert!(!has(&t, "Document"), "not one Document card: {t:?}");
}

#[test]
fn json_cards_are_in_japanese_too() {
    let t = cards(JSON, DocumentFormat::Json, Locale::Ja);
    assert!(has(&t, "2件"), "{t:?}");
    assert!(!has(&t, "2 items"));
}

#[test]
fn an_empty_json_container_is_one_card_for_the_whole_document() {
    let t = cards("{}\n", DocumentFormat::Json, Locale::En);
    assert!(has(&t, "Document"), "{t:?}");
}

#[test]
fn an_unreadable_json_file_is_one_card_for_the_whole_document() {
    let t = cards("{\"a\": ", DocumentFormat::Json, Locale::En);
    assert!(has(&t, "Document"), "{t:?}");
}
