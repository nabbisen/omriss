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

mod create_buttons;
mod draft;
mod menu_focus;
mod node_tree;
mod row_menu;

use dioxus::prelude::*;
use dioxus_swdir_tree::item_tree::node::NodeId as SwNodeId;
use dioxus_swdir_tree::{ItemTree, ItemTreeEvent, SelectionMode};
use omriss_core::DocumentFormat;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{DocumentMapNode, EditorSession, ViewMode, node_id_from_raw};

use create_buttons::SelectedSectionCreateButtons;
use draft::{commit_draft_if_dirty, sync_draft};
use menu_focus::focus_open_row_menu;
use node_tree::{collect_ancestors, find_node, to_item_node};
use row_menu::NodeRowMenu;

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
    // RFC-054 J3F-IMPL-001: the top-level "+ Add section" button below is a
    // panel-level control, not a row action -- it is not driven by
    // `NodeCapabilities` at all (the root's own capabilities are always
    // `hidden()`, for every format, so there is nothing on the root node to
    // gate it with). Structural add/rename/move/delete has no committed plan
    // for any format but Markdown (RFC-054 §13 Phase 4 is out of scope for
    // JSON entirely; TOML/YAML remain unimplemented), so gate on format
    // directly rather than inventing a new capability semantic for one button.
    let mut format_sig = use_signal(|| DocumentFormat::Markdown);

    // `session` is subscribed here only. Writing local signals inside this
    // effect is safe: Dioxus 0.7 tracks reactive deps by what is *read*
    // during the effect, not what is written. `item_tree` and `map_root_sig`
    // are therefore not deps of this effect and do not re-trigger it.
    use_effect(move || {
        let root_node = session.read().document_map_nodes();
        let view = session.read().view_mode();
        let is_raw = session.read().is_raw();
        let format = session.read().format();

        item_tree.write().set_tree(to_item_node(&root_node));

        // The root itself is never rendered as a row (the caller renders
        // `root.children`), but `set_tree` starts every node collapsed
        // (`is_expanded: false`), including the root -- so its children stay
        // invisible until something expands it. For Markdown this was always
        // masked by the auto-focus-on-open below expanding the focused node's
        // ancestors (root among them); a format with nothing to focus (RFC-054
        // J3: JSON/PlainText opt out of auto-focus entirely, see
        // `crates/app/src/shell/actions.rs::handle_load`) never took that path
        // and rendered an empty Document Map despite a correctly computed,
        // non-empty structure -- found live, not by a unit test, since
        // `document_map_nodes()` itself was already correct. Expand the root
        // unconditionally so top-level rows are visible on open regardless of
        // format or focus state.
        let root_sw_id = SwNodeId(root_node.id);
        if item_tree.read().is_expanded(root_sw_id) == Some(false) {
            item_tree.write().on_toggled(root_sw_id);
        }

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
        format_sig.set(format);
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
                // RFC-054 J3F-IMPL-001: structural add has no committed plan
                // for any format but Markdown (RFC-054 §13 Phase 4 is out of
                // scope for JSON; TOML/YAML are unimplemented) -- this button
                // spliced a Markdown H1 heading into whatever text was open,
                // corrupting a JSON document's source, before this gate.
                if *format_sig.read() == DocumentFormat::Markdown {
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
