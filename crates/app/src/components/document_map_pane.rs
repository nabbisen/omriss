//! Document Map panel — the single structure-organization surface (RFC-049).
//!
//! All structural editing actions (add, rename, move, join, delete) live here.
//! The right-side `FocusedContentPane` contains none of them.
//!
//! ## Reactive design
//!
//! `session` is subscribed exactly once, inside a `use_effect`. The effect
//! pushes two derived local signals: `item_tree` (for `ItemTreeView`) and
//! `map_root_sig` (the `DocumentMapNode` tree for row-menu rendering).
//! The component render body reads only local signals, so a `session` change
//! does not create a direct render-time subscription that would fight the
//! effect and cause a re-render loop.

use dioxus::prelude::*;
use dioxus_swdir_tree::item_tree::node::ItemNode;
use dioxus_swdir_tree::item_tree::node::NodeId as SwNodeId;
use dioxus_swdir_tree::{ItemTree, ItemTreeEvent, SelectionMode};
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{DocumentMapNode, EditorSession, MapCapability, ViewMode, node_id_from_raw};

// ── helpers ───────────────────────────────────────────────────────────────────

fn to_item_node(n: &DocumentMapNode) -> ItemNode<String> {
    let id = SwNodeId(n.id);
    let children: Vec<ItemNode<String>> = n.children.iter().map(to_item_node).collect();
    if children.is_empty() {
        ItemNode::leaf(id, n.title.clone())
    } else {
        ItemNode::branch(id, n.title.clone(), children)
    }
}

fn commit_draft_if_dirty(
    session: &mut Signal<EditorSession>,
    draft: &mut Signal<String>,
    status: &mut Signal<String>,
) -> bool {
    let snap = session.read().current_snapshot();
    let Some(s) = snap else { return true };
    let d = draft.read().clone();
    if d != s.body {
        if session.write().commit_focused_body(&s, d).is_err() {
            status.set("error.stale_edit".into());
            return false;
        }
    }
    true
}

fn sync_draft(session: &Signal<EditorSession>, draft: &mut Signal<String>) {
    let body = session
        .read()
        .current_snapshot()
        .map(|s| s.body)
        .unwrap_or_default();
    draft.set(body);
}

fn row_menu_focus_first_script(row_id: u64) -> String {
    format!(
        r#"
requestAnimationFrame(() => requestAnimationFrame(() => {{
  const root = document.querySelector('.row-menu[role="menu"][data-row-menu-id="{row_id}"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  (items[0] || root).focus();
}}))
"#
    )
}

const ROW_MENU_FOCUS_NEXT: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  if (!items.length) {
    root.focus();
    return;
  }
  const index = items.indexOf(document.activeElement);
  items[index < 0 ? 0 : (index + 1) % items.length].focus();
})()
"#;

const ROW_MENU_FOCUS_PREVIOUS: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  if (!items.length) {
    root.focus();
    return;
  }
  const index = items.indexOf(document.activeElement);
  items[index < 0 ? items.length - 1 : (index - 1 + items.length) % items.length].focus();
})()
"#;

const ROW_MENU_FOCUS_FIRST_ITEM: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  (items[0] || root).focus();
})()
"#;

const ROW_MENU_FOCUS_LAST_ITEM: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  (items[items.length - 1] || root).focus();
})()
"#;

fn focus_open_row_menu(row_id: u64) {
    let script = row_menu_focus_first_script(row_id);
    spawn(async move {
        let _ = document::eval(&script);
    });
}

fn move_open_row_menu_focus(script: &'static str) {
    spawn(async move {
        let _ = document::eval(script);
    });
}

// ── component ─────────────────────────────────────────────────────────────────

#[component]
pub fn DocumentMapPane(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
) -> Element {
    let lang = *locale.read();

    // Local signals — component body reads only these, never `session` directly.
    let mut item_tree = use_signal(|| ItemTree::new().with_display(|s: &String| s.clone()));
    let mut map_root_sig: Signal<Option<DocumentMapNode>> = use_signal(|| None);
    let mut menu_open_for: Signal<Option<u64>> = use_signal(|| None);
    let mut menu_node_sig: Signal<Option<DocumentMapNode>> = use_signal(|| None);
    let mut is_raw_sig = use_signal(|| false);

    // `session` is subscribed here only. Writing local signals inside this
    // effect is safe: Dioxus 0.7 tracks reactive deps by what is *read*
    // during the effect, not what is written. `item_tree` and `map_root_sig`
    // are therefore not deps of this effect and do not re-trigger it.
    use_effect(move || {
        let root_node = session.read().document_map_nodes();
        let view = session.read().view_mode();
        let is_raw = session.read().is_raw();

        item_tree.write().set_tree(to_item_node(&root_node));

        if let ViewMode::Focus(focused_id) = view {
            let focused_sw = SwNodeId(focused_id.0);

            // Keep the current editor focus visible in the map, including
            // focus changes initiated from breadcrumbs, child links, or search.
            let mut ancestors: Vec<SwNodeId> = Vec::new();
            collect_ancestors(&root_node, focused_sw, &mut ancestors);
            // ancestors is bottom-up (child before parent); reverse to expand
            // outermost first so each level becomes visible.
            ancestors.reverse();
            for id in ancestors {
                if item_tree.read().is_expanded(id) == Some(false) {
                    item_tree.write().on_toggled(id);
                }
            }

            // Sync visual selection for every focused-node change, including
            // navigation initiated outside the Document Map.
            item_tree
                .write()
                .on_selected(focused_sw, SelectionMode::Replace);
        }

        map_root_sig.set(Some(root_node));
        is_raw_sig.set(is_raw);
    });

    let mut on_event = move |ev: ItemTreeEvent| match ev {
        ItemTreeEvent::Toggled(id) => {
            item_tree.write().on_toggled(id);
        }
        ItemTreeEvent::Selected(id, mode) => {
            menu_open_for.set(None);
            let section_id = node_id_from_raw(id.0);
            if !commit_draft_if_dirty(
                &mut session.clone(),
                &mut draft.clone(),
                &mut status.clone(),
            ) {
                return;
            }
            if session.write().focus(section_id).is_ok() {
                item_tree.write().on_selected(id, mode);
                sync_draft(&session, &mut draft.clone());
            }
        }
        ItemTreeEvent::Drag(_) => {}
    };

    let close_menu = move |_: Event<MouseData>| {
        menu_open_for.set(None);
        menu_node_sig.set(None);
    };

    // Read local signals only — no session subscription here.
    let map_root = map_root_sig.read();
    // The document root's raw id — we skip action buttons for this row since
    // the top-level "+ Add section" button already covers adding H1 sections.
    let doc_root_raw_id: u64 = map_root.as_ref().map(|r| r.id).unwrap_or(u64::MAX);
    let is_empty = map_root
        .as_ref()
        .map(|r| r.children.is_empty())
        .unwrap_or(true);
    let selected_raw_id = item_tree
        .read()
        .visible_rows()
        .into_iter()
        .find(|row| row.is_selected && row.id.0 != doc_root_raw_id)
        .map(|row| row.id.0);
    let selected_node = selected_raw_id
        .and_then(|id| map_root.as_ref().and_then(|root| find_node(root, id)))
        .cloned();
    let is_raw = *is_raw_sig.read();
    // (view_mode is read only via session signal in use_effect; in_focus not needed here)

    rsx! {
        aside {
            class: "document-map-pane",
            "aria-label": t(lang, "aria.document_map"),
            onclick: close_menu,

            h2 { class: "document-map-title", {t(lang, "document_map.title")} }

            div {
                class: "document-map-create-actions",
                "aria-label": t(lang, "document_map.create_actions"),
                button {
                    class: "document-map-create-action",
                    title: t(lang, "document_map.action.add_top_level"),
                    "aria-label": t(lang, "document_map.action.add_top_level"),
                    onclick: move |ev| {
                        ev.stop_propagation();
                        if !commit_draft_if_dirty(
                            &mut session.clone(),
                            &mut draft.clone(),
                            &mut status.clone(),
                        ) {
                            return;
                        }
                        status.clone().set("struct.add_top.pending".into());
                    },
                    "+ "
                    {t(lang, "document_map.add_top_level")}
                }
                if let Some(selected) = selected_node.clone() {
                    SelectedSectionCreateButtons {
                        node: selected,
                        session,
                        locale,
                        draft,
                        status,
                    }
                }
            }

            if is_empty {
                p { class: "document-map-empty", {t(lang, "document_map.empty")} }
                p { class: "document-map-hint", {t(lang, "document_map.no_headings_hint")} }
            } else {
                // Render rows directly from item_tree so each row contains
                // both the tree content, the action trigger, and any open menu.
                // This keeps row capabilities visually attached to the row
                // that owns them.
                div {
                    class: "document-map-tree",
                    tabindex: "0",
                    onkeydown: move |evt| {
                        use dioxus_swdir_tree::TreeKey;
                        let tree_key = match evt.key() {
                            Key::ArrowUp => TreeKey::Up,
                            Key::ArrowDown => TreeKey::Down,
                            Key::ArrowLeft => TreeKey::Left,
                            Key::ArrowRight => TreeKey::Right,
                            Key::Enter => TreeKey::Enter,
                            Key::Home => TreeKey::Home,
                            Key::End => TreeKey::End,
                            Key::Escape => TreeKey::Escape,
                            Key::Character(ref ch) if ch == " " => TreeKey::Space,
                            _ => return,
                        };
                        let mods = dioxus_swdir_tree::Modifiers {
                            shift: evt.modifiers().shift(),
                            ctrl: evt.modifiers().ctrl(),
                        };
                        let event = {
                            let tree = item_tree.read();
                            tree.handle_key(tree_key, mods)
                        };
                        if let Some(ev) = event {
                            evt.prevent_default();
                            on_event(ev);
                        }
                    },
                    {item_tree.read().visible_rows().into_iter().filter_map(|row| {
                        let node_id = node_id_from_raw(row.id.0);
                        let raw_id = row.id.0;
                        // Never render the synthetic document root. It has no
                        // title and no buttons; rendering it as a blank first
                        // row shifts every real section row down one position,
                        // so clicks land on the wrong row (H1's + would target
                        // H2 and add an H3). Suppress it entirely.
                        if raw_id == doc_root_raw_id {
                            return None;
                        }
                        // Subtract the root's depth so H1 (tree depth 1) starts
                        // at indent 0, H2 at 16px, and so on.
                        let indent_px = row.depth.saturating_sub(1) * 16;
                        let caret = if row.has_children {
                            if row.is_expanded { "▾" } else { "▸" }
                        } else { " " };
                        let mut row_class = "dx-swdir-row".to_string();
                        if row.is_selected { row_class.push_str(" dx-swdir-row--selected"); }
                        Some(rsx! {
                            div {
                                key: "{raw_id}",
                                class: "{row_class}",
                                style: "padding-left: {indent_px}px;",
                                // Clicking anywhere on the row selects it.
                                onclick: move |ev| {
                                    ev.stop_propagation();
                                    menu_open_for.set(None);
                                    menu_node_sig.set(None);
                                    if !commit_draft_if_dirty(
                                        &mut session.clone(),
                                        &mut draft.clone(),
                                        &mut status.clone(),
                                    ) {
                                        return;
                                    }
                                    if session.write().focus(node_id).is_ok() {
                                        item_tree
                                            .write()
                                            .on_selected(SwNodeId(raw_id), SelectionMode::Replace);
                                        sync_draft(&session, &mut draft.clone());
                                    }
                                },
                                // Caret toggles expand/collapse
                                span {
                                    class: "dx-swdir-caret",
                                    onclick: move |ev| {
                                        ev.stop_propagation();
                                        item_tree.write().on_toggled(SwNodeId(raw_id));
                                    },
                                    "{caret}"
                                }
                                // Icon
                                span { class: "dx-swdir-icon" }
                                // Label (no separate onclick — the row div handles click)
                                span {
                                    class: "dx-swdir-label",
                                    style: "flex: 1; overflow: hidden; text-overflow: ellipsis; pointer-events: none;",
                                    "{row.label}"
                                }
                                // Row mutation/organization menu.
                                button {
                                    class: "row-menu-btn",
                                    title: t(lang, "document_map.actions"),
                                    "aria-label": t(lang, "document_map.actions"),
                                    "aria-haspopup": "menu",
                                    "aria-expanded": if *menu_open_for.read() == Some(raw_id) { "true" } else { "false" },
                                    onmousedown: move |ev| ev.prevent_default(),
                                    onclick: move |ev| {
                                        ev.stop_propagation();
                                        let cur = *menu_open_for.read();
                                        if cur == Some(raw_id) {
                                            menu_open_for.set(None);
                                            menu_node_sig.set(None);
                                            return;
                                        }
                                        menu_open_for.set(None);
                                        menu_node_sig.set(None);
                                        if !commit_draft_if_dirty(
                                            &mut session.clone(),
                                            &mut draft.clone(),
                                            &mut status.clone(),
                                        ) {
                                            return;
                                        }
                                        let _ = session.write().focus(node_id);
                                        sync_draft(&session, &mut draft.clone());
                                        item_tree
                                            .write()
                                            .on_selected(SwNodeId(raw_id), SelectionMode::Replace);
                                        let fresh_root = session.read().document_map_nodes();
                                        if let Some(menu_node) =
                                            find_node(&fresh_root, raw_id).cloned()
                                        {
                                            map_root_sig.set(Some(fresh_root));
                                            menu_node_sig.set(Some(menu_node));
                                            menu_open_for.set(Some(raw_id));
                                            focus_open_row_menu(raw_id);
                                        }
                                    },
                                    onkeydown: move |ev| {
                                        let activates_menu = matches!(ev.key(), Key::Enter)
                                            || matches!(ev.key(), Key::Character(ref ch) if ch == " ");
                                        if !activates_menu {
                                            return;
                                        }
                                        ev.stop_propagation();
                                        ev.prevent_default();
                                        let cur = *menu_open_for.read();
                                        if cur == Some(raw_id) {
                                            menu_open_for.set(None);
                                            menu_node_sig.set(None);
                                            return;
                                        }
                                        menu_open_for.set(None);
                                        menu_node_sig.set(None);
                                        if !commit_draft_if_dirty(
                                            &mut session.clone(),
                                            &mut draft.clone(),
                                            &mut status.clone(),
                                        ) {
                                            return;
                                        }
                                        let _ = session.write().focus(node_id);
                                        sync_draft(&session, &mut draft.clone());
                                        item_tree
                                            .write()
                                            .on_selected(SwNodeId(raw_id), SelectionMode::Replace);
                                        let fresh_root = session.read().document_map_nodes();
                                        if let Some(menu_node) =
                                            find_node(&fresh_root, raw_id).cloned()
                                        {
                                            map_root_sig.set(Some(fresh_root));
                                            menu_node_sig.set(Some(menu_node));
                                            menu_open_for.set(Some(raw_id));
                                            focus_open_row_menu(raw_id);
                                        }
                                    },
                                    "⋯"
                                }
                                if *menu_open_for.read() == Some(raw_id) {
                                    if let Some(menu_node) = menu_node_sig
                                        .read()
                                        .as_ref()
                                        .filter(|node| node.id == raw_id)
                                        .cloned() {
                                        NodeRowMenu {
                                            node: menu_node,
                                            session,
                                            locale,
                                            draft,
                                            status,
                                            menu_open_for,
                                        }
                                    }
                                }
                            }
                        })
                    })}
                }
            }

            // Plain-text view toggle is anchored at the bottom of the panel.
            button {
                class: "document-map-show-raw",
                onclick: move |ev| {
                    ev.stop_propagation();
                    if is_raw {
                        session.write().leave_raw();
                        return;
                    }
                    if !commit_draft_if_dirty(
                        &mut session.clone(),
                        &mut draft.clone(),
                        &mut status.clone(),
                    ) {
                        return;
                    }
                    session.write().show_raw();
                },
                if is_raw {
                    {t(lang, "raw.back")}
                } else {
                    {t(lang, "document_map.action.show_plain_text")}
                }
            }
        }
    }
}

// ── Selected-section creation buttons ────────────────────────────────────────

#[component]
fn SelectedSectionCreateButtons(
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

// ── Node row menu ─────────────────────────────────────────────────────────────

#[component]
fn NodeRowMenu(
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn disabled_title(lang: Locale, cap: &MapCapability) -> String {
    if let MapCapability::Disabled(r) = cap {
        t(lang, r.catalog_key()).to_string()
    } else {
        String::new()
    }
}

fn capability_title(lang: Locale, cap: &MapCapability, enabled_key: &'static str) -> String {
    if let MapCapability::Disabled(r) = cap {
        t(lang, r.catalog_key()).to_string()
    } else {
        t(lang, enabled_key).to_string()
    }
}

fn find_node(root: &DocumentMapNode, id: u64) -> Option<&DocumentMapNode> {
    if root.id == id {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_node(c, id))
}

/// Collect the `SwNodeId`s of all ancestors of `target` (root → parent,
/// not including `target` itself) into `out`. Returns `true` if found.
fn collect_ancestors(node: &DocumentMapNode, target: SwNodeId, out: &mut Vec<SwNodeId>) -> bool {
    if SwNodeId(node.id) == target {
        return true;
    }
    for child in &node.children {
        if collect_ancestors(child, target, out) {
            out.push(SwNodeId(node.id));
            return true;
        }
    }
    false
}
