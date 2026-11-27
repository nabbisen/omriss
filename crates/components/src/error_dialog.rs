//! General-purpose error dialog (RFC-039).
//!
//! Shows what happened (the title, a catalog key or plain text), the message — in
//! words a person reads, as body text: what happened in detail and what to do —
//! and the primary action. The optional `on_secondary` handler enables a second
//! action (e.g. "Save As" alongside "Dismiss").
//!
//! The message is **not** a "technical cause": since RFC-073 nothing the operating
//! system said reaches it (RFC-060 §5), so it is drawn like any other sentence the
//! application says to a person, not as a code excerpt.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

#[component]
pub fn ErrorDialog(
    locale: Signal<Locale>,
    /// Short description of what went wrong (i18n key or plain string).
    title: String,
    /// The message: what happened and what to do, already in the user's language,
    /// shown as body text and read out as the dialog's description.
    message: String,
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
            "data-focus-scope": "contain",
            "data-focus-arrows": "true",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "err-title",
            "aria-describedby": "err-message",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "err-title", class: "error-title", {title_text} }
                if !message.is_empty() {
                    p { id: "err-message", class: "error-message", {message.clone()} }
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
