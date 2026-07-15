//! Quick Actions overlay (RFC-022/RFC-048): filter and execute any registered
//! command by title. Keyboard help and the palette share one source of truth.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{COMMANDS, filter_commands};

/// An opaque command id dispatched back to the app shell for execution.
pub type CommandId = &'static str;

const PALETTE_FOCUS_FIRST: &str = r#"
(() => {
  const root = document.querySelector('.palette-overlay[aria-modal="true"]');
  if (!root) return;
  const focusables = Array.from(root.querySelectorAll(
    'input:not([disabled]), button:not([disabled])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  (focusables[0] || root).focus();
})()
"#;

const PALETTE_FOCUS_LAST: &str = r#"
(() => {
  const root = document.querySelector('.palette-overlay[aria-modal="true"]');
  if (!root) return;
  const focusables = Array.from(root.querySelectorAll(
    'input:not([disabled]), button:not([disabled])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  (focusables[focusables.length - 1] || root).focus();
})()
"#;

fn move_palette_focus(script: &'static str) {
    spawn(async move {
        let _ = document::eval(script);
    });
}

pub(crate) fn focus_active_palette() {
    spawn(async move {
        let _ = document::eval(
            "requestAnimationFrame(() => requestAnimationFrame(() => document.querySelector('.palette-input')?.focus()))",
        );
    });
}

#[component]
pub fn CommandPalette(
    locale: Signal<Locale>,
    on_close: EventHandler<()>,
    on_execute: EventHandler<CommandId>,
) -> Element {
    let lang = *locale.read();
    let mut query = use_signal(String::new);
    let mut active_index = use_signal(|| 0usize);

    let t_fn = |key: &'static str| t(lang, key).to_string();
    let filtered = filter_commands(COMMANDS, &query.read(), &t_fn);
    let active_index_value = (*active_index.read()).min(filtered.len().saturating_sub(1));

    use_effect(move || {
        focus_active_palette();
    });

    rsx! {
        div {
            class: "palette-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": t(lang, "palette.title"),
            onkeydown: move |evt| {
                match evt.key() {
                    Key::Escape => {
                        evt.stop_propagation();
                        evt.prevent_default();
                        on_close.call(());
                    }
                    Key::ArrowDown => {
                        evt.stop_propagation();
                        evt.prevent_default();
                        if !filtered.is_empty() {
                            let next = (active_index_value + 1) % filtered.len();
                            active_index.set(next);
                        }
                    }
                    Key::ArrowUp => {
                        evt.stop_propagation();
                        evt.prevent_default();
                        if !filtered.is_empty() {
                            let next = if active_index_value == 0 {
                                filtered.len() - 1
                            } else {
                                active_index_value - 1
                            };
                            active_index.set(next);
                        }
                    }
                    Key::Home => {
                        evt.stop_propagation();
                        evt.prevent_default();
                        active_index.set(0);
                    }
                    Key::End => {
                        evt.stop_propagation();
                        evt.prevent_default();
                        if !filtered.is_empty() {
                            active_index.set(filtered.len() - 1);
                        }
                    }
                    Key::Enter => {
                        evt.stop_propagation();
                        evt.prevent_default();
                        if let Some(cmd) = filtered.get(active_index_value) {
                            on_execute.call(cmd.id);
                            on_close.call(());
                        }
                    }
                    Key::Tab => {
                        evt.stop_propagation();
                    }
                    _ => {}
                }
            },
            span {
                class: "palette-sentinel",
                tabindex: 0,
                onfocus: move |_| {
                    move_palette_focus(PALETTE_FOCUS_LAST);
                },
                "Quick Actions focus boundary"
            }
            div { class: "palette-inner",
                input {
                    class: "palette-input",
                    r#type: "search",
                    placeholder: t(lang, "palette.placeholder"),
                    autofocus: true,
                    value: "{query}",
                    oninput: move |evt| {
                        query.set(evt.value());
                        active_index.set(0);
                    },
                }
                div { class: "palette-list", "aria-live": "polite",
                    if filtered.is_empty() {
                        p { class: "palette-empty hint-text",
                            {t(lang, "palette.no_results")}
                        }
                    } else {
                        for (idx, cmd) in filtered.iter().enumerate() {
                            {
                                let id = cmd.id;
                                let title = t(lang, cmd.title_key).to_string();
                                let shortcut = cmd.shortcut.unwrap_or("");
                                let class = if idx == active_index_value {
                                    "palette-item palette-item--active"
                                } else {
                                    "palette-item"
                                };
                                rsx! {
                                    button {
                                        key: "{id}",
                                        class,
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
            span {
                class: "palette-sentinel",
                tabindex: 0,
                onfocus: move |_| {
                    move_palette_focus(PALETTE_FOCUS_FIRST);
                },
                "Quick Actions focus boundary"
            }
        }
    }
}
