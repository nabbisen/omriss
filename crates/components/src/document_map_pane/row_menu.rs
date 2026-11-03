//! The Document Map row action menu (rename, move, join, delete).

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{DocumentMapNode, EditorSession, node_id_from_raw};

use super::draft::{commit_draft_if_dirty, sync_draft};
use super::menu_focus::{
    ROW_MENU_FOCUS_FIRST_ITEM, ROW_MENU_FOCUS_LAST_ITEM, ROW_MENU_FOCUS_NEXT,
    ROW_MENU_FOCUS_PREVIOUS, focus_open_row_menu, move_open_row_menu_focus,
};
use super::node_tree::{capability_title, disabled_title};

#[component]
pub(super) fn NodeRowMenu(
    node: DocumentMapNode,
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
    mut menu_open_for: Signal<Option<u64>>,
) -> Element {
    let lang = *locale.read();
    let caps = node.capabilities.clone();
    let node_id = node_id_from_raw(node.id);

    use_effect(move || {
        focus_open_row_menu(node.id);
    });

    rsx! {
        div {
            class: "row-menu",
            role: "menu",
            "aria-label": t(lang, "document_map.actions"),
            "data-row-menu-id": "{node.id}",
            tabindex: "-1",
            onclick: move |ev| ev.stop_propagation(),
            onkeydown: move |ev| {
                let activates_item = matches!(ev.key(), Key::Enter)
                    || matches!(ev.key(), Key::Character(ref ch) if ch == " ");
                if activates_item {
                    ev.stop_propagation();
                    return;
                }
                let focus_script = match ev.key() {
                    Key::Tab if ev.modifiers().shift() => Some(ROW_MENU_FOCUS_PREVIOUS),
                    Key::Tab => Some(ROW_MENU_FOCUS_NEXT),
                    Key::ArrowDown => Some(ROW_MENU_FOCUS_NEXT),
                    Key::ArrowUp => Some(ROW_MENU_FOCUS_PREVIOUS),
                    Key::Home => Some(ROW_MENU_FOCUS_FIRST_ITEM),
                    Key::End => Some(ROW_MENU_FOCUS_LAST_ITEM),
                    Key::Escape => {
                        ev.stop_propagation();
                        ev.prevent_default();
                        menu_open_for.set(None);
                        return;
                    }
                    _ => None,
                };
                if let Some(script) = focus_script {
                    ev.stop_propagation();
                    ev.prevent_default();
                    move_open_row_menu_focus(script);
                }
            },

            if !caps.can_move_up.is_hidden() {
                button {
                    class: "row-menu-item",
                    role: "menuitem",
                    disabled: !caps.can_move_up.is_allowed(),
                    title: disabled_title(lang, &caps.can_move_up),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        if session.write().move_focused_up().is_ok() {
                            sync_draft(&session, &mut draft.clone());
                        }
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.move_up")}
                }
            }
            if !caps.can_move_down.is_hidden() {
                button {
                    class: "row-menu-item",
                    role: "menuitem",
                    disabled: !caps.can_move_down.is_allowed(),
                    title: disabled_title(lang, &caps.can_move_down),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        if session.write().move_focused_down().is_ok() {
                            sync_draft(&session, &mut draft.clone());
                        }
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.move_down")}
                }
            }
            if !caps.can_move_inside_previous.is_hidden() {
                button {
                    class: "row-menu-item",
                    role: "menuitem",
                    disabled: !caps.can_move_inside_previous.is_allowed(),
                    title: disabled_title(lang, &caps.can_move_inside_previous),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        if session.write().demote_focused().is_ok() {
                            sync_draft(&session, &mut draft.clone());
                        }
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.move_inside_previous")}
                }
            }
            if !caps.can_move_out_one_level.is_hidden() {
                button {
                    class: "row-menu-item",
                    role: "menuitem",
                    disabled: !caps.can_move_out_one_level.is_allowed(),
                    title: disabled_title(lang, &caps.can_move_out_one_level),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        if session.write().promote_focused().is_ok() {
                            sync_draft(&session, &mut draft.clone());
                        }
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.move_out_one_level")}
                }
            }

            if !caps.can_rename.is_hidden() {
                button {
                    class: "row-menu-item",
                    role: "menuitem",
                    disabled: !caps.can_rename.is_allowed(),
                    title: disabled_title(lang, &caps.can_rename),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        sync_draft(&session, &mut draft.clone());
                        status.clone().set("struct.rename.pending".into());
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.rename")}
                }
            }
            if !caps.can_join_with_previous.is_hidden() {
                button {
                    class: "row-menu-item",
                    role: "menuitem",
                    disabled: !caps.can_join_with_previous.is_allowed(),
                    title: capability_title(
                        lang,
                        &caps.can_join_with_previous,
                        "document_map.action.join_with_previous.title",
                    ),
                    "aria-label": t(lang, "document_map.action.join_with_previous.title"),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        if session.write().merge_focused_up().is_ok() {
                            sync_draft(&session, &mut draft.clone());
                        }
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.join_with_previous")}
                }
            }

            if !caps.can_delete.is_hidden() {
                button {
                    class: "row-menu-item row-menu-item--danger",
                    role: "menuitem",
                    disabled: !caps.can_delete.is_allowed(),
                    onclick: move |_| {
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        let _ = session.write().focus(node_id);
                        sync_draft(&session, &mut draft.clone());
                        status.clone().set("struct.delete.pending".into());
                        menu_open_for.set(None);
                    },
                    {t(lang, "document_map.action.delete")}
                }
            }

        }
    }
}
