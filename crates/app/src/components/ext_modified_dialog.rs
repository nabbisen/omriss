//! External file modification dialog (RFC-015).
//!
//! Shown when the file on disk has changed since it was last opened or saved,
//! before Omriss overwrites it with the in-memory canonical text.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

/// The user's choice when an external modification is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtModifiedChoice {
    /// Overwrite the disk file with in-memory canonical text.
    Overwrite,
    /// Open a Save As dialog to choose a different path.
    SaveAs,
    /// Cancel the save operation; do not touch the disk file.
    Cancel,
}

/// Modal dialog displayed when the file on disk changed externally.
#[component]
pub fn ExtModifiedDialog(
    locale: Signal<Locale>,
    on_choice: EventHandler<ExtModifiedChoice>,
) -> Element {
    let lang = *locale.read();
    rsx! {
        div {
            class: "modal-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "extmod-title",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "extmod-title", {t(lang, "dialog.ext_modified.title")} }
                p { {t(lang, "dialog.ext_modified.body")} }
                div { class: "modal-actions",
                    button {
                        class: "primary",
                        autofocus: true,
                        onclick: move |_| on_choice.call(ExtModifiedChoice::Overwrite),
                        {t(lang, "dialog.ext_modified.overwrite")}
                    }
                    button {
                        onclick: move |_| on_choice.call(ExtModifiedChoice::SaveAs),
                        {t(lang, "dialog.ext_modified.save_as")}
                    }
                    button {
                        onclick: move |_| on_choice.call(ExtModifiedChoice::Cancel),
                        {t(lang, "dialog.ext_modified.cancel")}
                    }
                }
            }
        }
    }
}
