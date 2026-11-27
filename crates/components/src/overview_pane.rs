//! Overview canvas: shown in the main pane when the user is in outline mode.
//! Displays the document's current-level children as clickable heading cards
//! (RFC-011) with keyboard-selected highlighting driven by `selected_card`.

use dioxus::prelude::*;
use omriss_core::StructureNodeKind;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t, t_args};

use crate::draft_sync::open_node;
use crate::focused_content_pane::structured::kind_label;

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
                            // The label each format already uses (RFC-085 §3.3 item 3): "H2"
                            // for a Markdown section; for JSON, what the value is. A group or
                            // list is told apart by its item count, below.
                            let level_label = match (item.level, item.value_kind) {
                                (Some(l), _) => format!("H{}", l.as_u8()),
                                (None, Some(kind)) => kind_label(lang, kind).to_string(),
                                (None, None) => String::new(),
                            };
                            let is_markdown = item.level.is_some()
                                || item.kind == StructureNodeKind::MarkdownSection
                                || item.kind == StructureNodeKind::DocumentRoot;
                            let meta = if is_markdown && count == 1 {
                                "overview.card.subsections_one"
                            } else if is_markdown {
                                "overview.card.subsections"
                            } else if count == 1 {
                                "overview.card.items_one"
                            } else {
                                "overview.card.items"
                            };
                            let count_text = t_args(lang, meta, &[("count", &count.to_string())]);
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
                                            span { class: "count", "{count_text}" }
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

#[cfg(test)]
mod tests;
