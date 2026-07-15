//! Markdown preview pane (RFC-045): renders the focused section body as HTML
//! using the pulldown-cmark HTML renderer. Read-only — never modifies source.

use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

#[component]
pub fn PreviewPane(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    on_close: EventHandler<()>,
) -> Element {
    let lang = *locale.read();
    let html = session.read().preview_html();
    let title = session
        .read()
        .current_snapshot()
        .map(|s| s.title)
        .unwrap_or_default();

    use_effect(move || {
        spawn(async move {
            let _ = document::eval(
                "requestAnimationFrame(() => requestAnimationFrame(() => document.querySelector('.preview-back')?.focus()))",
            );
        });
    });

    rsx! {
        div {
            class: "preview-pane",
            role: "region",
            "aria-label": t(lang, "editor.preview"),
            div { class: "preview-header",
                h2 { class: "preview-title", "{title}" }
                button {
                    class: "preview-back",
                    autofocus: true,
                    onclick: move |_| on_close.call(()),
                    "aria-label": t(lang, "editor.source"),
                    "\u{2190} {t(lang, \"editor.source\")}"
                }
            }
            if html.trim().is_empty() {
                p { class: "hint-text", {t(lang, "focus.empty_body")} }
            } else {
                div {
                    class: "preview-body",
                    // RFC-045: read-only HTML rendering; source is unchanged.
                    dangerous_inner_html: "{html}",
                }
            }
        }
    }
}
