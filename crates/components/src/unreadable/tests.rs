//! RFC-085 §7 PR 5, level 2: the Writing Area beside a file whose structure cannot
//! be shown says what the Document Map says (one catalog key) and offers the text.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::{Locale, t, t_args};
use omriss_ui::{EditorSession, FileTextProfile};

use super::UnreadablePane;
use crate::test_texts::texts_of;

thread_local! {
    static CASE: std::cell::Cell<(&'static str, DocumentFormat, Locale)> =
        const { std::cell::Cell::new(("", DocumentFormat::Json, Locale::En)) };
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
        UnreadablePane {
            session,
            locale: use_signal(move || locale),
            draft: use_signal(String::new),
            status: use_signal(String::new),
        }
    }
}

fn pane(source: &'static str, format: DocumentFormat, locale: Locale) -> Vec<String> {
    CASE.with(|c| c.set((source, format, locale)));
    texts_of(host)
}

#[test]
fn unreadable_json_says_the_maps_sentence_and_offers_the_text() {
    for locale in [Locale::En, Locale::Ja] {
        let t_ = pane("{\"a\": ", DocumentFormat::Json, locale);
        let sentence = t_args(locale, "document_map.unreadable", &[("format", "JSON")]);
        assert!(t_.contains(&sentence), "{locale:?}: {t_:?}");
        assert!(
            t_.iter()
                .any(|x| x == t(locale, "document_map.action.show_plain_text")),
            "the button to the text: {t_:?}"
        );
        // Never an empty card or an empty view.
        assert!(
            !t_.iter().any(|x| x == t(locale, "breadcrumb.root")),
            "{t_:?}"
        );
    }
}

#[test]
fn other_formats_say_their_own_sentence() {
    let toml = pane("a = 1\n", DocumentFormat::Toml, Locale::En);
    assert!(toml.contains(&t_args(
        Locale::En,
        "document_map.unsupported_format",
        &[("format", "TOML")]
    )));
    let plain = pane("plain\n", DocumentFormat::PlainText, Locale::En);
    assert!(plain.contains(&t(Locale::En, "document_map.plain_text").to_string()));
}

#[test]
fn the_sentence_is_the_maps_not_a_copy() {
    // One function builds both, from one catalog key per format.
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/document_map_pane.rs"
    ))
    .unwrap();
    assert!(src.contains("unreadable_message("));
    assert!(!src.contains("fn unreadable_message"));
}
