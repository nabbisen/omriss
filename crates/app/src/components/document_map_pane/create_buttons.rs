//! Selected-section creation buttons for the Document Map panel.

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{DocumentMapNode, EditorSession, node_id_from_raw};

use super::draft::{commit_draft_if_dirty, sync_draft};
use super::node_tree::capability_title;

#[component]
pub(super) fn SelectedSectionCreateButtons(
    node: DocumentMapNode,
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
) -> Element {
    let lang = *locale.read();
    let caps = node.capabilities.clone();
    let node_id = node_id_from_raw(node.id);

    rsx! {
        if !caps.can_add_inside.is_hidden() {
            button {
                class: "document-map-create-action",
                disabled: !caps.can_add_inside.is_allowed(),
                title: capability_title(
                    lang,
                    &caps.can_add_inside,
                    "document_map.action.add_inside",
                ),
                "aria-label": t(lang, "document_map.action.add_inside"),
                onclick: move |ev| {
                    ev.stop_propagation();
                    if !commit_draft_if_dirty(
                        &mut session.clone(),
                        &mut draft.clone(),
                        &mut status.clone(),
                    ) {
                        return;
                    }
                    let _ = session.write().focus(node_id);
                    sync_draft(&session, &mut draft.clone());
                    status.clone().set("struct.add_inside.pending".into());
                },
                "+ "
                {t(lang, "document_map.action.add_inside_short")}
            }
        }
        if !caps.can_add_after.is_hidden() {
            button {
                class: "document-map-create-action",
                disabled: !caps.can_add_after.is_allowed(),
                title: capability_title(
                    lang,
                    &caps.can_add_after,
                    "document_map.action.add_after",
                ),
                "aria-label": t(lang, "document_map.action.add_after"),
                onclick: move |ev| {
                    ev.stop_propagation();
                    if !commit_draft_if_dirty(
                        &mut session.clone(),
                        &mut draft.clone(),
                        &mut status.clone(),
                    ) {
                        return;
                    }
                    let _ = session.write().focus(node_id);
                    sync_draft(&session, &mut draft.clone());
                    status.clone().set("struct.add_after.pending".into());
                },
                "+ "
                {t(lang, "document_map.action.add_after_short")}
            }
        }
    }
}
