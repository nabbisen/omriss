//! RFC-085 §3.4, level 2: what the Document Map says when it has no rows, per
//! format and per case — and that no JSON file is ever told to add `#` headings.
//!
//! The pane fills its tree in an effect, so each case is rendered once, the
//! effects run, and it is rendered again.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::{Locale, t, t_args};
use omriss_ui::{EditorSession, FileTextProfile};

use super::DocumentMapPane;
use crate::structure_request::StructureRequest;
use crate::test_texts::texts_of_settled;

thread_local! {
    static CASE: std::cell::Cell<(&'static str, DocumentFormat, Locale)> =
        const { std::cell::Cell::new(("", DocumentFormat::Markdown, Locale::En)) };
}

fn host() -> Element {
    let (source, format, locale) = CASE.with(|c| c.get());
    let session = use_signal(|| {
        let mut s = EditorSession::open_detected(
            source.to_string(),
            None,
            FileTextProfile::detect(source, false),
            format,
        )
        .unwrap();
        s.settle_on_open();
        s
    });
    let request: Signal<Option<StructureRequest>> = use_signal(|| None);
    rsx! {
        DocumentMapPane {
            session,
            locale: use_signal(move || locale),
            draft: use_signal(String::new),
            status: use_signal(String::new),
            request,
        }
    }
}

fn texts(source: &'static str, format: DocumentFormat, locale: Locale) -> Vec<String> {
    CASE.with(|c| c.set((source, format, locale)));
    texts_of_settled(host)
}

fn says(texts: &[String], wanted: &str) -> bool {
    texts.iter().any(|t| t == wanted)
}

const HINT_KEY: &str = "document_map.no_headings_hint";

#[test]
fn markdown_without_headings_keeps_its_hint() {
    let t = texts("just text\n", DocumentFormat::Markdown, Locale::En);
    assert!(
        says(&t, "Add headings with # to begin organizing."),
        "{t:?}"
    );
}

#[test]
fn an_unreadable_json_file_gets_the_sentence_and_not_the_markdown_hint() {
    let en = texts("{\"a\": ", DocumentFormat::Json, Locale::En);
    let sentence = t_args(Locale::En, "document_map.unreadable", &[("format", "JSON")]);
    assert!(says(&en, &sentence), "{en:?}");
    assert!(
        !says(&en, t(Locale::En, HINT_KEY)),
        "no # hint for JSON: {en:?}"
    );
    let ja = texts("{\"a\": ", DocumentFormat::Json, Locale::Ja);
    let sentence = t_args(Locale::Ja, "document_map.unreadable", &[("format", "JSON")]);
    assert!(says(&ja, &sentence), "{ja:?}");
    assert!(!says(&ja, t(Locale::Ja, HINT_KEY)));
}

#[test]
fn valid_json_with_no_members_says_so() {
    for source in ["{}\n", "[]\n"] {
        let t = texts(source, DocumentFormat::Json, Locale::En);
        assert!(
            says(&t, "This JSON has no keys or items to show."),
            "{source:?}: {t:?}"
        );
        assert!(!says(&t, t_str(HINT_KEY)), "{t:?}");
    }
}

#[test]
fn valid_json_with_members_has_rows_and_no_empty_message() {
    let t = texts("{\"a\": 1, \"b\": [2]}\n", DocumentFormat::Json, Locale::En);
    assert!(says(&t, "a") && says(&t, "b"), "rows: {t:?}");
    assert!(!says(&t, "This JSON has no keys or items to show."));
    assert!(!says(&t, t_str(HINT_KEY)));
}

#[test]
fn other_formats_have_their_own_words() {
    let toml = texts("a = 1\n", DocumentFormat::Toml, Locale::En);
    assert!(
        says(
            &toml,
            &t_args(
                Locale::En,
                "document_map.unsupported_format",
                &[("format", "TOML")]
            )
        ),
        "{toml:?}"
    );
    let plain = texts("plain\n", DocumentFormat::PlainText, Locale::En);
    assert!(
        says(&plain, t(Locale::En, "document_map.plain_text")),
        "{plain:?}"
    );
}

fn t_str(key: &'static str) -> &'static str {
    t(Locale::En, key)
}

#[test]
fn the_maps_rows_for_a_list_say_item_n_in_the_users_language() {
    let en = texts("[true, false]\n", DocumentFormat::Json, Locale::En);
    assert!(says(&en, "Item 1") && says(&en, "Item 2"), "{en:?}");
    let ja = texts("[true, false]\n", DocumentFormat::Json, Locale::Ja);
    assert!(says(&ja, "項目 1") && says(&ja, "項目 2"), "{ja:?}");
}
