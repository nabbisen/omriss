//! Unsaved changes confirmation dialog (RFC-016).
//!
//! Shown before any action that would discard uncommitted edits: open another
//! file, create a new document, or quit the application.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

/// The user's choice in the unsaved-changes dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChoice {
    /// Commit the pending draft, save the file, then proceed.
    Save,
    /// Discard all unsaved work and proceed without saving.
    Discard,
    /// Return to the editor; do not proceed with the action.
    Cancel,
}

/// Modal dialog displayed when an action would lose unsaved changes.
///
/// The caller is responsible for rendering this component only when needed
/// (e.g. `if show_unsaved_dialog { UnsavedDialog { … } }`).
#[component]
pub fn UnsavedDialog(locale: Signal<Locale>, on_choice: EventHandler<UnsavedChoice>) -> Element {
    let lang = *locale.read();
    rsx! {
        // Accessible modal overlay (RFC-028 focus trap expected by M6).
        div {
            class: "modal-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "unsaved-title",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "unsaved-title", {t(lang, "dialog.unsaved.title")} }
                p { {t(lang, "dialog.unsaved.body")} }
                div { class: "modal-actions",
                    button {
                        class: "primary",
                        autofocus: true,
                        onclick: move |_| on_choice.call(UnsavedChoice::Save),
                        {t(lang, "dialog.unsaved.save")}
                    }
                    button {
                        onclick: move |_| on_choice.call(UnsavedChoice::Discard),
                        {t(lang, "dialog.unsaved.discard")}
                    }
                    button {
                        onclick: move |_| on_choice.call(UnsavedChoice::Cancel),
                        {t(lang, "dialog.unsaved.cancel")}
                    }
                }
            }
        }
    }
}
