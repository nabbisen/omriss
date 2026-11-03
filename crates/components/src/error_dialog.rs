//! General-purpose error dialog (RFC-039).
//!
//! Shows a structured message: what happened, the technical cause, and the
//! primary recovery action. The optional `on_secondary` handler enables a
//! second action (e.g. "Save As" alongside "Dismiss").

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

#[component]
pub fn ErrorDialog(
    locale: Signal<Locale>,
    /// Short description of what went wrong (i18n key or plain string).
    title: String,
    /// Technical reason text (plain string, shown as a detail).
    cause: String,
    /// Label for the primary dismiss button.
    dismiss_label: String,
    /// Optional label + handler for a secondary recovery action.
    secondary_label: Option<String>,
    on_dismiss: EventHandler<()>,
    on_secondary: Option<EventHandler<()>>,
) -> Element {
    let lang = *locale.read();
    // Resolve title through i18n if it matches a known key.
    let title_text = if title.contains('.') {
        t(lang, &title).to_string()
    } else {
        title.clone()
    };

    rsx! {
        div {
            class: "modal-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "err-title",
            "aria-describedby": "err-cause",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "err-title", class: "error-title", {title_text} }
                if !cause.is_empty() {
                    p { id: "err-cause", class: "error-cause", {cause.clone()} }
                }
                div { class: "modal-actions",
                    button {
                        autofocus: true,
                        onclick: move |_| on_dismiss.call(()),
                        {dismiss_label.clone()}
                    }
                    if let (Some(label), Some(handler)) = (secondary_label.clone(), on_secondary) {
                        button {
                            class: "primary",
                            onclick: move |_| handler.call(()),
                            {label}
                        }
                    }
                }
            }
        }
    }
}
