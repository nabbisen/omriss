//! Quick Actions overlay (RFC-022/RFC-048): filter and execute any registered
//! command by title. Keyboard help and the palette share one source of truth.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{COMMANDS, filter_commands};

/// An opaque command id dispatched back to the app shell for execution.
pub type CommandId = &'static str;

#[component]
pub fn CommandPalette(
    locale: Signal<Locale>,
    on_close: EventHandler<()>,
    on_execute: EventHandler<CommandId>,
) -> Element {
    let lang = *locale.read();
    let mut query = use_signal(String::new);

    let t_fn = |key: &'static str| t(lang, key).to_string();
    let filtered = filter_commands(COMMANDS, &query.read(), &t_fn);

    rsx! {
        div {
            class: "palette-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": t(lang, "palette.title"),
            div { class: "palette-inner",
                input {
                    class: "palette-input",
                    r#type: "search",
                    placeholder: t(lang, "palette.placeholder"),
                    autofocus: true,
                    value: "{query}",
                    oninput: move |evt| query.set(evt.value()),
                    onkeydown: move |evt| {
                        use keyboard_types::Code;
                        if evt.data().code() == Code::Escape {
                            on_close.call(());
                        }
                    },
                }
                div { class: "palette-list", "aria-live": "polite",
                    if filtered.is_empty() {
                        p { class: "palette-empty hint-text",
                            {t(lang, "palette.no_results")}
                        }
                    } else {
                        for cmd in filtered.iter() {
                            {
                                let id = cmd.id;
                                let title = t(lang, cmd.title_key).to_string();
                                let shortcut = cmd.shortcut.unwrap_or("");
                                rsx! {
                                    button {
                                        key: "{id}",
                                        class: "palette-item",
                                        onclick: move |_| {
                                            on_execute.call(id);
                                            on_close.call(());
                                        },
                                        span { class: "palette-title", "{title}" }
                                        if !shortcut.is_empty() {
                                            span { class: "palette-shortcut hint-text", "{shortcut}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
