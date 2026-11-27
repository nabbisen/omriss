//! The dialog for a file that could not be opened (RFC-039, RFC-073 §8–§9).
//!
//! What it shows is decided here, in one place, so the shell and its tests mount
//! the same thing: a title that says what happened and only that, the message —
//! what happened in detail and what to do, said once — as body text, and a "Close"
//! button that has its own words.

use dioxus::prelude::*;
use omriss_components::ErrorDialog;
use omriss_ui::i18n::{Locale, t};

use crate::file::failure::OpenFailure;
use crate::shell::failure_text::open_failure_message;

/// The dialog for `failure`, in the current language (read at render, so a language
/// change while it is open is honoured).
#[component]
pub(crate) fn OpenErrorDialog(
    locale: Signal<Locale>,
    failure: OpenFailure,
    on_dismiss: EventHandler<()>,
) -> Element {
    let lang = *locale.read();
    rsx! {
        ErrorDialog {
            locale,
            // "This file could not be opened." — what happened, and no advice: what to
            // do is the message's to say, once.
            title: "error.open_failed".to_string(),
            message: open_failure_message(lang, &failure),
            // There is nothing to cancel: the dialog is closed, so its button says Close.
            dismiss_label: t(lang, "dialog.close").to_string(),
            secondary_label: None,
            on_dismiss: move |()| on_dismiss.call(()),
            on_secondary: None,
        }
    }
}
