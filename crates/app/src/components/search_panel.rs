//! Search panel overlay (RFC-021): whole-document or current-section search
//! with section-grouped results and one-click focus navigation.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{EditorSession, SearchMatch};

#[component]
pub fn SearchPanel(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    on_close: EventHandler<()>,
    on_navigate: EventHandler<omriss_core::NodeId>,
) -> Element {
    let lang = *locale.read();
    let mut query = use_signal(String::new);
    let mut whole_doc = use_signal(|| true);

    // Derive results from query + scope.
    let results: Vec<SearchMatch> = {
        let q = query.read().clone();
        let wd = *whole_doc.read();
        let sess = session.read();
        if wd {
            sess.search_document(&q)
        } else {
            sess.search_section(&q)
        }
    };

    rsx! {
        div {
            class: "search-overlay",
            role: "search",
            "aria-label": t(lang, "search.title"),
            div { class: "search-header",
                h2 { {t(lang, "search.title")} }
                button {
                    class: "search-close",
                    onclick: move |_| on_close.call(()),
                    "aria-label": t(lang, "search.close"),
                    "\u{00D7}"
                }
            }
            div { class: "search-controls",
                input {
                    class: "search-input",
                    r#type: "search",
                    placeholder: t(lang, "search.placeholder"),
                    autofocus: true,
                    value: "{query}",
                    oninput: move |evt| query.set(evt.value()),
                }
                div { class: "search-scope",
                    label {
                        input {
                            r#type: "radio",
                            name: "scope",
                            checked: *whole_doc.read(),
                            onchange: move |_| whole_doc.set(true),
                        }
                        {t(lang, "search.scope.document")}
                    }
                    label {
                        input {
                            r#type: "radio",
                            name: "scope",
                            checked: !*whole_doc.read(),
                            onchange: move |_| whole_doc.set(false),
                        }
                        {t(lang, "search.scope.section")}
                    }
                }
            }
            div { class: "search-results",
                "aria-live": "polite",
                if query.read().trim().is_empty() {
                    // No query entered yet — show nothing.
                } else if results.is_empty() {
                    p { class: "search-empty hint-text", {t(lang, "search.no_results")} }
                } else {
                    p { class: "search-count hint-text",
                        "{results.len()} {t(lang, \"search.results\")}"
                    }
                    for result in results.iter() {
                        {
                            let path_label: String = result
                                .path
                                .iter()
                                .map(|p| if p.title.is_empty() { t(lang, "breadcrumb.root").to_string() } else { p.title.clone() })
                                .collect::<Vec<_>>()
                                .join(" › ");
                            let preview = result.preview.clone();
                            let node_id = result.containing_node;
                            rsx! {
                                button {
                                    class: "search-result",
                                    key: "{result.range.start}",
                                    onclick: move |_| {
                                        on_navigate.call(node_id);
                                        on_close.call(());
                                    },
                                    span { class: "result-path hint-text", "{path_label}" }
                                    span { class: "result-preview", "{preview}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
