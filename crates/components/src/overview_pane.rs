//! Overview canvas: shown in the main pane when the user is in outline mode.
//! Displays the document's current-level children as clickable heading cards
//! (RFC-011) with keyboard-selected highlighting driven by `selected_card`.

use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

use crate::draft_sync::open_node;

#[component]
pub fn OverviewPane(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
    selected_card: Signal<usize>,
) -> Element {
    let mut draft = draft;
    let mut status = status;
    let mut session = session;
    let lang = *locale.read();
    let items = session.read().current_children();
    let sel = *selected_card.read();

    rsx! {
        main { class: "main-pane",
            if items.is_empty() {
                p { class: "outline-empty", {t(lang, "outline.empty")} }
                p { class: "outline-hint", {t(lang, "outline.no-headings.hint")} }
            } else {
                div { class: "overview-cards",
                    for (idx , item) in items.iter().enumerate() {
                        {
                            let id = item.id;
                            let count = item.child_count;
                            let title = item.title.clone();
                            let level_label = item.level
                                .map(|l| format!("H{}", l.as_u8()))
                                .unwrap_or_default();
                            let is_selected = idx == sel;
                            rsx! {
                                div {
                                    key: "{id.0}",
                                    role: "button",
                                    tabindex: if is_selected { "0" } else { "-1" },
                                    class: if is_selected {
                                        "overview-card overview-card--selected"
                                    } else {
                                        "overview-card"
                                    },
                                    onclick: move |_| {
                                        selected_card.set(idx);
                                        open_node(&mut session, &mut draft, &mut status, id);
                                    },
                                    div { class: "card-header",
                                        span { class: "card-level", "{level_label}" }
                                        span { class: "card-title",
                                            if title.is_empty() {
                                                {t(lang, "breadcrumb.root")}
                                            } else {
                                                "{title}"
                                            }
                                        }
                                    }
                                    if count > 0 {
                                        div { class: "card-meta",
                                            span { class: "count", "{count} subsections" }
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
