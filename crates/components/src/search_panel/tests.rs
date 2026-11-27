//! RFC-077 §8.2, level 2: a title result carries the word for what matched — "Heading"
//! for a Markdown heading, "Key" for a JSON key — in both locales; a text result
//! carries none; and a key and its value, which start at the same place, are two
//! rows.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{EditorSession, FileTextProfile};

use super::SearchResultList;
use crate::test_texts::texts_of;

thread_local! {
    static CASE: std::cell::Cell<(&'static str, DocumentFormat, &'static str, Locale)> =
        const { std::cell::Cell::new(("", DocumentFormat::Markdown, "", Locale::En)) };
}

fn host() -> Element {
    let (source, format, query, locale) = CASE.with(|c| c.get());
    let session = EditorSession::open_detected(
        source.to_string(),
        None,
        FileTextProfile::detect(source, false),
        format,
    )
    .unwrap();
    let results = session.search_document(query);
    rsx! {
        SearchResultList {
            results,
            locale: use_signal(move || locale),
            on_navigate: move |_| {},
            on_close: move |_| {},
        }
    }
}

fn rows(
    source: &'static str,
    format: DocumentFormat,
    query: &'static str,
    locale: Locale,
) -> Vec<String> {
    CASE.with(|c| c.set((source, format, query, locale)));
    texts_of(host)
}

fn count(texts: &[String], word: &str) -> usize {
    texts.iter().filter(|t| t.as_str() == word).count()
}

#[test]
fn a_markdown_heading_result_says_heading_and_a_body_result_says_nothing() {
    for locale in [Locale::En, Locale::Ja] {
        let t_ = rows(
            "# Fox\nthe fox runs\n",
            DocumentFormat::Markdown,
            "fox",
            locale,
        );
        assert_eq!(
            count(&t_, t(locale, "search.kind.heading")),
            1,
            "{locale:?}: {t_:?}"
        );
        assert_eq!(count(&t_, t(locale, "search.kind.key")), 0, "{t_:?}");
    }
}

#[test]
fn a_json_key_result_says_key_and_a_key_and_its_value_are_two_rows() {
    for locale in [Locale::En, Locale::Ja] {
        let t_ = rows(
            "{\"name\": \"name of it\"}\n",
            DocumentFormat::Json,
            "name",
            locale,
        );
        assert_eq!(
            count(&t_, t(locale, "search.kind.key")),
            1,
            "{locale:?}: {t_:?}"
        );
        assert_eq!(count(&t_, t(locale, "search.kind.heading")), 0);
        // Both rows are there: the preview of the key and of the value.
        assert!(t_.iter().any(|x| x == "name"), "{t_:?}");
        assert!(t_.iter().any(|x| x == "name of it"), "{t_:?}");
    }
}

#[test]
fn the_words_differ_in_the_two_locales_and_exist() {
    assert_eq!(t(Locale::En, "search.kind.heading"), "Heading");
    assert_eq!(t(Locale::En, "search.kind.key"), "Key");
    assert_ne!(t(Locale::Ja, "search.kind.heading"), "search.kind.heading");
    assert_ne!(t(Locale::Ja, "search.kind.key"), "search.kind.key");
}

#[test]
fn a_search_path_through_a_list_names_its_member_in_the_users_language() {
    let en = rows(
        "{\"tags\": [\"alpha\"]}\n",
        DocumentFormat::Json,
        "alpha",
        Locale::En,
    );
    assert!(en.iter().any(|x| x.ends_with("tags › Item 1")), "{en:?}");
    let ja = rows(
        "{\"tags\": [\"alpha\"]}\n",
        DocumentFormat::Json,
        "alpha",
        Locale::Ja,
    );
    assert!(ja.iter().any(|x| x.ends_with("tags › 項目 1")), "{ja:?}");
    assert!(!ja.iter().any(|x| x.contains("Item")), "{ja:?}");
}
