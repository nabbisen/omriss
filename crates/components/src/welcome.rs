//! Welcome / start screen (RFC-010, RFC-041 onboarding).
//!
//! Shows a five-step onboarding hint, primary actions, and the recent-files
//! list so returning users can quickly re-open their last document.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

#[component]
pub fn WelcomeScreen(
    locale: Signal<Locale>,
    /// Recent file paths (most recent first). Stale paths are filtered by
    /// the caller before being passed here.
    recent_files: Signal<Vec<String>>,
    on_open: EventHandler<()>,
    on_new: EventHandler<()>,
    /// Open a specific recent file by path.
    on_open_recent: EventHandler<String>,
) -> Element {
    let lang = *locale.read();
    let recents = recent_files.read().clone();

    rsx! {
        main { class: "welcome",
            div { class: "welcome-inner",
                h1 { class: "welcome-title", "Omriss" }
                p { class: "welcome-tagline", {t(lang, "welcome.tagline")} }

                div { class: "welcome-actions",
                    button {
                        class: "primary welcome-btn",
                        autofocus: true,
                        onclick: move |_| on_open.call(()),
                        {t(lang, "menu.file.open")}
                    }
                    button {
                        class: "welcome-btn",
                        onclick: move |_| on_new.call(()),
                        {t(lang, "menu.file.new")}
                    }
                }

                if !recents.is_empty() {
                    section { class: "recent-files",
                        h2 { class: "recent-title", "Recent Files" }
                        for path in recents.iter() {
                            {
                                let p = path.clone();
                                let display = std::path::Path::new(path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(path.as_str())
                                    .to_string();
                                let dir = std::path::Path::new(path)
                                    .parent()
                                    .and_then(|d| d.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                rsx! {
                                    button {
                                        class: "recent-item",
                                        key: "{p}",
                                        onclick: move |_| on_open_recent.call(p.clone()),
                                        span { class: "recent-name", "{display}" }
                                        span { class: "recent-dir hint-text", "{dir}" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    p { class: "hint-text welcome-hint", {t(lang, "welcome.hint")} }
                }
            }
        }
    }
}
