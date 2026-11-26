use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

use crate::draft_sync::{open_node, open_overview};

/// Maximum number of segments shown before collapsing the middle.
const MAX_VISIBLE: usize = 4;

#[component]
pub fn Breadcrumb(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
) -> Element {
    let lang = *locale.read();
    let Some(snapshot) = session.read().current_snapshot() else {
        return rsx! {};
    };

    let path = snapshot.path.clone();
    let total = path.len();

    let collapsed = total > MAX_VISIBLE;
    let display: Vec<(usize, bool)> = if collapsed {
        let mut v: Vec<(usize, bool)> = Vec::with_capacity(MAX_VISIBLE);
        v.push((0, false));
        v.push((usize::MAX, false));
        v.push((total - 2, false));
        v.push((total - 1, true));
        v
    } else {
        (0..total).map(|i| (i, i + 1 == total)).collect()
    };

    rsx! {
        nav {
            class: "breadcrumb",
            "aria-label": t(lang, "aria.breadcrumb"),
            // The path reads from the left, on one line, cut with an ellipsis
            // (RFC-013); the up button stays outside it, at the right edge.
            div { class: "breadcrumb-crumbs",
                for (item_idx , is_current) in display.iter() {
                    {
                        let item_idx = *item_idx;
                        let is_current = *is_current;
                        if item_idx == usize::MAX {
                            rsx! {
                                span { class: "sep", "aria-hidden": "true", " › " }
                                span { class: "ellipsis", "aria-label": t(lang, "breadcrumb.collapsed"), "…" }
                            }
                        } else {
                            let item = path[item_idx].clone();
                            let label = if item.title.is_empty() {
                                t(lang, "breadcrumb.root").to_string()
                            } else {
                                item.title.clone()
                            };
                            rsx! {
                                if item_idx > 0 {
                                    span { class: "sep", "aria-hidden": "true", " › " }
                                }
                                if is_current {
                                    span {
                                        class: "here",
                                        title: "{label}",
                                        "aria-current": "page",
                                        "{label}"
                                    }
                                } else {
                                    button {
                                        class: "crumb-btn",
                                        title: "{label}",
                                        onclick: move |_| {
                                            if item.level.is_none() {
                                                open_overview(&mut session, &mut draft, &mut status);
                                            } else {
                                                open_node(&mut session, &mut draft, &mut status, item.id);
                                            }
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Up one level — icon button at the right edge of the breadcrumb bar.
            button {
                class: "breadcrumb-up",
                title: t(lang, "nav.up"),
                "aria-label": t(lang, "nav.up"),
                onclick: move |_| {
                    crate::draft_sync::zoom_out(&mut session, &mut draft, &mut status);
                },
                "↑"
            }
        }
    }
}

#[cfg(test)]
mod tests;
