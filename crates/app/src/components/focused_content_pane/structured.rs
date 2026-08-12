//! Right-panel rendering and editing for a focused non-Markdown (JSON)
//! node (RFC-054 §4.4–§4.7, §7.4 — `focused_content_pane.rs` dispatches
//! here instead of its Markdown body/preview/children path).
//!
//! J7a built the read half (`focused_structured_content()`, no input
//! anywhere). J7b adds the write half: a text/number input, an on/off
//! toggle, a container raw-text editor, and `DraftState` (RFC-053 §9.3)
//! blocking — apply-on-navigation, the same interaction pattern RFC-050
//! already established for Markdown's body textarea, rather than the
//! RFC-054 §4.4/§4.5 mockup's separate `[Done]` button. Consistency with
//! the one interaction pattern this app already uses everywhere else, and
//! `DraftState`'s own purpose (blocking *navigation* away from an invalid
//! draft), both point the same way: commit on blur/navigate, not on a
//! second explicit action. On/off is the one case with no natural "blur"
//! — a toggle click commits immediately, matching §4.6's mockup exactly.

use dioxus::prelude::*;
use omriss_core::{FocusedContent, ValueKind};
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{EditorSession, StructureErrorKindCatalogKey};

use crate::shell::draft_sync::json_edit_error_key;

#[component]
pub fn StructuredFocusView(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
) -> Element {
    let lang = *locale.read();
    let mut raw_open = use_signal(|| false);
    let Some(content) = session.read().focused_structured_content() else {
        return rsx! {};
    };

    match content {
        FocusedContent::StructuredValue {
            title, value_kind, ..
        } => {
            let draft_state = session.read().structured_draft_state(&draft.read());
            let is_invalid = draft_state == omriss_core::DraftState::InvalidUncommitted;

            let do_blur = move |_: Event<FocusData>| {
                commit_value_draft(session, draft, status);
            };

            rsx! {
                h1 { class: "focus-title", "{title}" }
                p { class: "structured-kind-label", "{kind_label(lang, value_kind)}" }
                match value_kind {
                    ValueKind::NoValue => rsx! {
                        p {
                            class: "focused-content-empty-hint",
                            {t(lang, "focused_content.json.empty_value_hint")}
                        }
                    },
                    ValueKind::OnOff => rsx! {
                        div { class: "structured-onoff",
                            button {
                                class: if *draft.read() == "true" { "btn-onoff active" } else { "btn-onoff" },
                                onclick: move |_| commit_onoff(session, draft, status, "true"),
                                {t(lang, "focused_content.json.onoff.on")}
                            }
                            button {
                                class: if *draft.read() == "false" { "btn-onoff active" } else { "btn-onoff" },
                                onclick: move |_| commit_onoff(session, draft, status, "false"),
                                {t(lang, "focused_content.json.onoff.off")}
                            }
                        }
                    },
                    _ => rsx! {
                        input {
                            r#type: "text",
                            class: if is_invalid { "structured-value-input invalid" } else { "structured-value-input" },
                            "aria-label": "{kind_label(lang, value_kind)}",
                            "aria-invalid": if is_invalid { "true" } else { "false" },
                            value: "{draft}",
                            oninput: move |ev| draft.set(ev.value()),
                            onblur: do_blur,
                        }
                        if is_invalid {
                            p { class: "structured-error", role: "alert",
                                {t(lang, value_error_key(value_kind))}
                            }
                        }
                    },
                }
            }
        }
        FocusedContent::StructuredGroup {
            title,
            child_count,
            summary,
            raw_text_available,
        } => {
            let draft_state = session.read().structured_draft_state(&draft.read());
            let is_invalid = draft_state == omriss_core::DraftState::InvalidUncommitted;
            let do_blur = move |_: Event<FocusData>| {
                commit_raw_draft(session, draft, status);
            };

            rsx! {
                h1 { class: "focus-title", "{title}" }
                p { class: "focused-content-empty-hint", {t(lang, "focused_content.json.group_hint")} }
                p {
                    class: "structured-item-count",
                    "{t(lang, \"focused_content.json.item_count_label\")}: {child_count}"
                }
                if *raw_open.read() {
                    textarea {
                        class: if is_invalid { "structured-raw-editor invalid" } else { "structured-raw-editor" },
                        "aria-label": t(lang, "focused_content.json.raw_text_label"),
                        "aria-invalid": if is_invalid { "true" } else { "false" },
                        value: "{draft}",
                        oninput: move |ev| draft.set(ev.value()),
                        onblur: do_blur,
                    }
                    if is_invalid {
                        p { class: "structured-error", role: "alert",
                            {t(lang, "focused_content.json.error.invalid_raw_value")}
                        }
                    }
                } else {
                    if raw_text_available && !summary.is_empty() {
                        pre { class: "structured-group-summary", "{summary}" }
                    }
                    if raw_text_available {
                        button {
                            class: "btn-show-raw",
                            onclick: move |_| raw_open.set(true),
                            {t(lang, "focused_content.json.show_as_text")}
                        }
                    }
                }
            }
        }
        FocusedContent::MarkdownSection { .. } => rsx! {},
        FocusedContent::Unsupported { reason, .. } => rsx! {
            p { class: "focused-content-empty-hint", "{t(lang, reason.catalog_key())}" }
        },
    }
}

/// Commits `draft` as the focused value's new text (Text/Number), no-op
/// if unchanged. Refreshes `draft` to the freshly committed
/// `editable_text` on success (e.g. a Text draft is re-decoded through
/// the same escaping `focused_content` used to show it); leaves `draft`
/// as-is on failure so the user's invalid input stays visible to fix.
fn commit_value_draft(
    mut session: Signal<EditorSession>,
    mut draft: Signal<String>,
    mut status: Signal<String>,
) {
    let d = draft.read().clone();
    if !session.read().structured_draft_differs(&d) {
        return;
    }
    let result = session.write().commit_structured_draft(&d);
    match result {
        Ok(_) => {
            if let Some(FocusedContent::StructuredValue { editable_text, .. }) =
                session.read().focused_structured_content()
            {
                draft.set(editable_text);
            }
        }
        Err(kind) => status.set(json_edit_error_key(kind).into()),
    }
}

/// On/off commits immediately on click — there is no draft to leave
/// mid-edit for a two-button toggle (RFC-054 §4.6).
fn commit_onoff(
    mut session: Signal<EditorSession>,
    mut draft: Signal<String>,
    mut status: Signal<String>,
    literal: &'static str,
) {
    draft.set(literal.to_string());
    let result = session.write().commit_structured_draft(literal);
    if let Err(kind) = result {
        status.set(json_edit_error_key(kind).into());
    }
}

/// Commits `draft` as the focused container's new raw text (RFC-054
/// §7.4), no-op if unchanged. Mirrors `commit_value_draft`'s
/// refresh-on-success / preserve-on-failure shape.
fn commit_raw_draft(
    mut session: Signal<EditorSession>,
    mut draft: Signal<String>,
    mut status: Signal<String>,
) {
    let d = draft.read().clone();
    if !session.read().structured_draft_differs(&d) {
        return;
    }
    let result = session.write().commit_structured_draft(&d);
    match result {
        Ok(_) => {
            if let Some(text) = session.read().focused_structured_raw_text() {
                draft.set(text);
            }
        }
        Err(kind) => status.set(json_edit_error_key(kind).into()),
    }
}

fn kind_label(lang: Locale, kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::Text => t(lang, "focused_content.json.kind.text"),
        ValueKind::Number => t(lang, "focused_content.json.kind.number"),
        ValueKind::OnOff => t(lang, "focused_content.json.kind.onoff"),
        ValueKind::NoValue => t(lang, "focused_content.json.kind.empty"),
        // Unused by any adapter today (see omriss-core's own doc comment
        // on the variant); a container's raw text is not modeled as a
        // ValueKind at all, since it comes from StructuredGroup, not
        // StructuredValue. Falls back to the Text label rather than
        // panicking if a future adapter ever does construct one.
        ValueKind::RawText => t(lang, "focused_content.json.kind.text"),
    }
}

/// RFC-054 §10's context-specific message for a `StructuredValue` commit
/// failure, chosen with the value kind this call site actually knows —
/// more specific than `json_edit_error_key`'s generic fallback. Only
/// `Number` and `OnOff` ever reach this (`Text` always encodes
/// successfully; `NoValue` is never editable).
fn value_error_key(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::Number => "focused_content.json.error.invalid_number",
        _ => "focused_content.json.error.invalid_raw_value",
    }
}
