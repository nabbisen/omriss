//! Plain file text view (RFC-017/RFC-048, editable per RFC-076): transparent
//! inspection of the canonical text, the recovery path when the structure
//! projection is incomplete, and — by explicit choice — a way to edit it.
//!
//! The view always opens **read-only**. Someone who opens it to look is not one
//! keystroke from changing the file; editing starts only when they choose
//! **Edit**. While editing, the one live draft is the raw text, held in the same
//! `draft` signal the Writing Area uses, so the raw draft and a section draft
//! can never both exist (RFC-076 §3.3): the section draft is committed before
//! the view opens, and the raw draft before it closes. A raw commit replaces
//! text across the whole document and is one undo step; focus returns to the
//! outline rather than being guessed at, and the structure-change notice says
//! what changed.

use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

use crate::draft_sync::{commit_and_resync, commit_or_block, sync};

#[component]
pub fn RawSourceView(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
) -> Element {
    let lang = *locale.read();
    let editing = session.read().raw_editing();
    // Snapshot the source once to avoid repeated borrows inside rsx!
    let source = session.read().source().to_string();
    let line_count = source.lines().count();

    let toggle_editing = move |_| {
        if !editing {
            let text = session.write().begin_raw_edit();
            draft.set(text);
            return;
        }
        // Commit first; a refused commit keeps the user in the editor.
        if !commit_or_block(
            &mut session.clone(),
            &mut draft.clone(),
            &mut status.clone(),
        ) {
            return;
        }
        session.write().end_raw_edit();
        sync(&session, &mut draft.clone());
    };
    // Apply on blur, like every other draft (RFC-050).
    let apply = move |_: Event<FocusData>| {
        commit_and_resync(
            &mut session.clone(),
            &mut draft.clone(),
            &mut status.clone(),
        );
    };

    rsx! {
        main { class: "main-pane raw-source",
            div { class: "raw-toolbar",
                h2 { {t(lang, "raw.title")} }
                button {
                    class: "raw-edit-toggle",
                    onclick: toggle_editing,
                    if editing {
                        {t(lang, "raw.stop_editing")}
                    } else {
                        {t(lang, "raw.edit")}
                    }
                }
                span { class: "raw-meta hint-text",
                    "{line_count} lines"
                }
            }
            p {
                class: "raw-notice hint-text",
                "aria-live": "polite",
                if editing {
                    {t(lang, "raw.editing_notice")}
                } else {
                    {t(lang, "raw.readonly_notice")}
                }
            }
            if editing {
                textarea {
                    class: "raw-editor",
                    "aria-label": t(lang, "raw.editor_label"),
                    spellcheck: "false",
                    autofocus: true,
                    value: "{draft}",
                    oninput: move |event| draft.set(event.value()),
                    onblur: apply,
                }
            } else {
                pre {
                    class: "raw-pre",
                    // role="region" + label so screen readers can browse the source.
                    role: "region",
                    "aria-label": t(lang, "raw.title"),
                    code { class: "raw-code", {source} }
                }
            }
        }
    }
}
