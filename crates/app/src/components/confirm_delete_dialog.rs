//! Delete-section confirmation dialog (RFC-025).
//!
//! Shown before removing a section subtree, which cannot be recovered
//! without Undo. The dialog displays the section title and child count
//! so the user understands the scope of the deletion.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmDeleteChoice {
    Delete,
    Cancel,
}

#[component]
pub fn ConfirmDeleteDialog(
    locale: Signal<Locale>,
    /// Title of the section to be deleted.
    section_title: String,
    /// Number of immediate child sections.
    child_count: usize,
    on_choice: EventHandler<ConfirmDeleteChoice>,
) -> Element {
    let lang = *locale.read();
    rsx! {
        div {
            class: "modal-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "del-title",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "del-title", {t(lang, "dialog.confirm_delete.title")} }
                p {
                    if section_title.is_empty() {
                        {t(lang, "dialog.confirm_delete.body")}
                    } else {
                        "\u{201c}{section_title}\u{201d} "
                        {t(lang, "dialog.confirm_delete.body")}
                    }
                    if child_count > 0 {
                        span { class: "hint-text", " ({child_count} subsections)" }
                    }
                }
                div { class: "modal-actions",
                    button {
                        class: "btn-danger",
                        autofocus: true,
                        onclick: move |_| on_choice.call(ConfirmDeleteChoice::Delete),
                        {t(lang, "dialog.confirm_delete.confirm")}
                    }
                    button {
                        onclick: move |_| on_choice.call(ConfirmDeleteChoice::Cancel),
                        {t(lang, "dialog.confirm_delete.cancel")}
                    }
                }
            }
        }
    }
}
