use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

use crate::draft_sync::commit_or_block;

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
            for (item_idx , is_current) in display.iter() {
                {
                    let item_idx = *item_idx;
                    let is_current = *is_current;
                    if item_idx == usize::MAX {
                        rsx! {
                            span { class: "sep", "aria-hidden": "true", " › " }
                            span { class: "ellipsis", "aria-label": "collapsed sections", "…" }
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
                                    "aria-current": "page",
                                    "{label}"
                                }
                            } else {
                                button {
                                    class: "crumb-btn",
                                    onclick: move |_| {
                                        if !commit_or_block(&mut session, &mut draft, &mut status) {
                                            return;
                                        }
                                        if item.level.is_none() {
                                            session.write().show_outline();
                                            draft.set(String::new());
                                        } else {
                                            let _ = session.write().focus(item.id);
                                            let body = session.read()
                                                .current_snapshot()
                                                .map(|s| s.body)
                                                .unwrap_or_default();
                                            draft.set(body);
                                        }
                                    },
                                    "{label}"
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
                    if !commit_or_block(&mut session, &mut draft, &mut status) {
                        return;
                    }
                    session.write().zoom_out();
                    let body = session
                        .read()
                        .current_snapshot()
                        .map(|s| s.body)
                        .unwrap_or_default();
                    draft.set(body);
                },
                "↑"
            }
        }
    }
}
