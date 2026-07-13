//! Status bar (RFC-010, RFC-029).
//!
//! Shows: status message (polite/assertive live region) · dirty marker ·
//! file name. Statistics and line-ending details are omitted to reduce noise
//! for new users (less-is-more design principle).

use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

fn is_error_key(key: &str) -> bool {
    key.starts_with("error.")
}

#[component]
pub fn StatusBar(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    status: Signal<String>,
    on_save_as: EventHandler<()>,
) -> Element {
    let lang = *locale.read();
    let dirty = session.read().is_dirty();
    let file = session.read().file_name().unwrap_or_default().to_string();
    let raw = session.read().is_raw();

    let key = status.read().clone();
    let status_msg = t(lang, key.as_str()).to_string();
    let is_error = is_error_key(&key);
    let is_save_error = key == "error.save_failed";

    rsx! {
        footer { class: "statusbar",
            if is_error {
                span {
                    "aria-live": "assertive",
                    "aria-atomic": "true",
                    class: "status-error",
                    {status_msg}
                    if is_save_error {
                        " "
                        button {
                            class: "status-action",
                            onclick: move |_| on_save_as.call(()),
                            {t(lang, "menu.file.save_as")}
                        }
                    }
                }
            } else {
                span { "aria-live": "polite", "aria-atomic": "true", {status_msg} }
            }
            if raw {
                span { class: "raw-badge", {t(lang, "raw.title")} }
            }
            if dirty {
                span { class: "dirty", {t(lang, "status.unsaved")} }
            }
            if !file.is_empty() {
                span { class: "file", "{file}" }
            }
        }
    }
}
