//! Read-only right-panel rendering for a focused non-Markdown (JSON) node
//! (RFC-054 J7a — the read half of app wiring; `focused_content_pane.rs`
//! dispatches here instead of its Markdown body/preview/children path).
//!
//! **This half mutates nothing** — the same framing RFC-053 S4a used for
//! `build_structure`. No input commits a value; that is J7b, which adds
//! the actual text/number/on-off/container editors, `DraftState`
//! blocking, and save/undo. This component only renders what
//! `EditorSession::focused_structured_content()` (a read-only accessor
//! over `JsonAdapter::focused_content`) already returns.

use dioxus::prelude::*;
use omriss_core::{FocusedContent, ValueKind};
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{EditorSession, StructureErrorKindCatalogKey};

#[component]
pub fn StructuredFocusView(session: Signal<EditorSession>, locale: Signal<Locale>) -> Element {
    let lang = *locale.read();
    let Some(content) = session.read().focused_structured_content() else {
        return rsx! {};
    };

    match content {
        FocusedContent::StructuredValue {
            title,
            value_kind,
            display_text,
            ..
        } => rsx! {
            h1 { class: "focus-title", "{title}" }
            p { class: "structured-kind-label", "{kind_label(lang, value_kind)}" }
            if value_kind == ValueKind::NoValue {
                p {
                    class: "focused-content-empty-hint",
                    {t(lang, "focused_content.json.empty_value_hint")}
                }
            } else {
                div { class: "structured-value-display", "{display_text}" }
            }
        },
        FocusedContent::StructuredGroup {
            title,
            child_count,
            summary,
            raw_text_available,
        } => rsx! {
            h1 { class: "focus-title", "{title}" }
            p { class: "focused-content-empty-hint", {t(lang, "focused_content.json.group_hint")} }
            p {
                class: "structured-item-count",
                "{t(lang, \"focused_content.json.item_count_label\")}: {child_count}"
            }
            if raw_text_available && !summary.is_empty() {
                pre { class: "structured-group-summary", "{summary}" }
            }
        },
        FocusedContent::MarkdownSection { .. } => rsx! {},
        FocusedContent::Unsupported { reason, .. } => rsx! {
            p { class: "focused-content-empty-hint", "{t(lang, reason.catalog_key())}" }
        },
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
