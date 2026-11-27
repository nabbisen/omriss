//! A file whose structure cannot be shown (RFC-085 §3.4, §7 PR 5).
//!
//! The Document Map says why it has no rows; the Writing Area, outside plain file
//! text, says the same thing — one catalog key, not a copy — and offers the way to
//! the text. It never shows an empty "Document" card or an empty view: there is
//! nothing to open but the text.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t, t_args};

use crate::draft_sync::open_raw;

/// The sentence for a file whose structure cannot be shown. It says what
/// happened, where the text is, and what makes the structure appear; it does not
/// blame the user (RFC-080 §5.4 rule 8). The words follow the format.
pub(crate) fn unreadable_message(lang: Locale, format: DocumentFormat) -> String {
    match format {
        DocumentFormat::Json => t_args(lang, "document_map.unreadable", &[("format", "JSON")]),
        DocumentFormat::Toml => t_args(
            lang,
            "document_map.unsupported_format",
            &[("format", "TOML")],
        ),
        DocumentFormat::YamlExperimental => t_args(
            lang,
            "document_map.unsupported_format",
            &[("format", "YAML")],
        ),
        DocumentFormat::Markdown | DocumentFormat::PlainText | DocumentFormat::Unsupported => {
            t(lang, "document_map.plain_text").to_string()
        }
    }
}

/// The Writing Area for a file whose structure cannot be shown, when plain file
/// text is not open: the map's sentence and a button to the text.
#[component]
pub fn UnreadablePane(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
) -> Element {
    let lang = *locale.read();
    let message = unreadable_message(lang, session.read().format());
    rsx! {
        main { class: "main-pane unreadable-pane",
            p { class: "unreadable-sentence", "{message}" }
            button {
                class: "unreadable-show-raw",
                onclick: move |_| {
                    open_raw(&mut session.clone(), &mut draft.clone(), &mut status.clone());
                },
                {t(lang, "document_map.action.show_plain_text")}
            }
        }
    }
}

#[cfg(test)]
mod tests;
