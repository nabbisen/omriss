//! Focused Content Area — the right panel (RFC-050).
//!
//! This panel has one job: edit, preview, and show save status for the
//! selected content. It contains:
//!
//! - breadcrumb path + section title;
//! - the section body textarea;
//! - Preview toggle;
//! - save status feedback;
//! - read-only child section navigation links (navigation only, no structure controls).
//!
//! **Structure controls (move in/out, move up/down, join, delete, add section)
//! are NOT here.** They live in `DocumentMapPane`. Any structural operation
//! triggered here (e.g. from keyboard shortcuts) is routed through the session
//! by the parent `App`, not this component.
//!
//! Apply-on-navigation (RFC-053 DraftState): the draft is committed whenever
//! the user navigates, saves, opens preview, or the component commits via
//! blur. There is no explicit "Done" primary action.

use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

use super::{Breadcrumb, PreviewPane};

#[component]
pub fn FocusedContentPane(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
    preview_open: Signal<bool>,
) -> Element {
    let lang = *locale.read();
    let Some(snapshot) = session.read().current_snapshot() else {
        return rsx! { main { class: "focused-content-pane" } };
    };

    let local_dirty = *draft.read() != snapshot.body;

    // Commit on blur (apply-on-navigation pattern: RFC-050).
    let blur_base = snapshot.clone();
    let do_blur = move |_: Event<FocusData>| {
        let body = draft.read().clone();
        if body == blur_base.body {
            return;
        }
        let result = session.write().commit_focused_body(&blur_base, body);
        match result {
            Ok(_) => {
                let refreshed = session
                    .read()
                    .current_snapshot()
                    .map(|s| s.body)
                    .unwrap_or_default();
                draft.set(refreshed);
            }
            Err(_) => {
                status.clone().set("error.stale_edit".into());
            }
        }
    };

    rsx! {
        main {
            class: "focused-content-pane",
            "aria-label": t(lang, "focused_content.title"),

            Breadcrumb { session, locale, draft, status }

            // ── Section title ─────────────────────────────────────────────
            h1 {
                class: "focus-title",
                if snapshot.title.is_empty() {
                    {t(lang, "breadcrumb.root")}
                } else {
                    "{snapshot.title}"
                }
                if local_dirty {
                    span { class: "local-dirty", " \u{25cf}" }
                }
            }

            // ── Preview or editor ─────────────────────────────────────────
            if *preview_open.read() {
                PreviewPane {
                    session,
                    locale,
                    on_close: move |()| {
                        let mut po = preview_open;
                        po.set(false);
                    },
                }
            } else {
                // Empty section guidance
                if snapshot.body.is_empty() && !local_dirty {
                    p {
                        class: "focused-content-empty-hint",
                        {t(lang, "focused_content.empty_section_hint")}
                    }
                }

                textarea {
                    class: "body-editor",
                    "aria-label": t(lang, "aria.editor"),
                    placeholder: t(lang, "editor.body.placeholder"),
                    autofocus: true,
                    value: "{draft}",
                    oninput: move |event| draft.set(event.value()),
                    onblur: do_blur,
                }

                // ── Editor footer: preview toggle + save status ───────────
                div { class: "focused-content-footer",
                    button {
                        class: if *preview_open.read() { "btn-preview active" } else { "btn-preview" },
                        title: t(lang, "editor.preview"),
                        onclick: move |_| {
                            let currently_open = *preview_open.read();
                            if !currently_open {
                                // Commit draft before preview (apply-on-navigation).
                                let snap = session.read().current_snapshot();
                                if let Some(s) = snap {
                                    let d = draft.read().clone();
                                    if d != s.body
                                        && session.write().commit_focused_body(&s, d).is_err()
                                    {
                                        status.clone().set("error.stale_edit".into());
                                        return;
                                    }
                                }
                            }
                            let mut po = preview_open;
                            po.set(!currently_open);
                        },
                        {t(lang, "editor.preview")}
                    }

                    // Save status (right-aligned via CSS flex)
                    span {
                        class: "save-status",
                        "aria-live": "polite",
                        if session.read().is_dirty() {
                            {t(lang, "status.unsaved")}
                        } else {
                            {t(lang, "status.saved")}
                        }
                    }
                }
            }

            // ── Read-only child navigation ────────────────────────────────
            // Children are shown as navigation links only; structure ops
            // (add, delete, rename, move) belong to the Document Map.
            if !snapshot.children.is_empty() {
                section {
                    class: "focused-content-children",
                    "aria-label": t(lang, "focus.children"),
                    for child in snapshot.children.clone() {
                        button {
                            class: "child-card",
                            key: "{child.id.0}",
                            onclick: move |_| {
                                // Commit before zooming in.
                                let snap = session.read().current_snapshot();
                                if let Some(s) = snap {
                                    let d = draft.read().clone();
                                    if d != s.body
                                        && session.write().commit_focused_body(&s, d).is_err()
                                    {
                                        status.clone().set("error.stale_edit".into());
                                        return;
                                    }
                                }
                                let _ = session.write().focus(child.id);
                                let body = session
                                    .read()
                                    .current_snapshot()
                                    .map(|s| s.body)
                                    .unwrap_or_default();
                                draft.set(body);
                            },
                            "{child.title}"
                            if child.child_count > 0 {
                                span { class: "count", " ({child.child_count})" }
                            }
                        }
                    }
                }
            }
        }
    }
}
