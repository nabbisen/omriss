//! Application toolbar: file actions, navigation controls, dirty marker (RFC-010 shell, RFC-014 keyboard contract).

use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::{Locale, t};

use crate::shell::draft_sync;

#[component]
pub fn Toolbar(
    session: Signal<EditorSession>,
    locale: Signal<Locale>,
    draft: Signal<String>,
    status: Signal<String>,
    on_open: EventHandler<()>,
    on_save: EventHandler<()>,
    on_save_as: EventHandler<()>,
    search_available: bool,
    on_search: EventHandler<()>,
) -> Element {
    let lang = *locale.read();
    let local_dirty = draft_sync::is_pending(&session, &draft);
    let dirty = session.read().is_dirty() || local_dirty;
    let can_undo = session.read().can_undo();
    let can_redo = session.read().can_redo();
    let can_back = session.read().can_go_back();
    let can_forward = session.read().can_go_forward();
    let mut settings_open = use_signal(|| false);
    let settings_expanded = *settings_open.read();
    let search_title = if search_available {
        t(lang, "search.title")
    } else {
        t(lang, "search.unavailable")
    };

    // Undo/Redo intentionally *block* on a pending draft rather than
    // committing it first (unlike Back/Forward/Save below) — undoing
    // while there's an uncommitted edit is ambiguous, so the user resolves
    // it first.
    let undo = move |_| {
        if draft_sync::is_pending(&session, &draft) {
            status.set("status.unsaved".into());
            return;
        }
        if session.write().undo().is_ok() {
            draft_sync::sync(&session, &mut draft);
        }
    };
    let redo = move |_| {
        if draft_sync::is_pending(&session, &draft) {
            status.set("status.unsaved".into());
            return;
        }
        if session.write().redo().is_ok() {
            draft_sync::sync(&session, &mut draft);
        }
    };
    let back = move |_| {
        if !draft_sync::commit_or_block(&mut session, &mut draft, &mut status) {
            return;
        }
        session.write().back();
        draft_sync::sync(&session, &mut draft);
    };
    let forward = move |_| {
        if !draft_sync::commit_or_block(&mut session, &mut draft, &mut status) {
            return;
        }
        session.write().forward();
        draft_sync::sync(&session, &mut draft);
    };

    rsx! {
        header { class: "toolbar", role: "toolbar", "aria-label": t(lang, "menu.file"),
            if settings_expanded {
                div {
                    class: "toolbar-menu-backdrop",
                    onclick: move |_| settings_open.set(false),
                }
            }
            button { onclick: move |_| on_open.call(()), {t(lang, "menu.file.open")} }
            button { onclick: move |_| on_save.call(()), {t(lang, "menu.file.save")} }
            button { onclick: move |_| on_save_as.call(()), {t(lang, "menu.file.save_as")} }
            div { class: "toolbar-sep" }
            button { disabled: !can_undo, onclick: undo, {t(lang, "toolbar.undo")} }
            button { disabled: !can_redo, onclick: redo, {t(lang, "toolbar.redo")} }
            div { class: "toolbar-sep" }
            button { disabled: !can_back, onclick: back, {t(lang, "nav.back")} }
            button { disabled: !can_forward, onclick: forward, {t(lang, "nav.forward")} }
            div { class: "spacer" }
            if dirty {
                span { class: "dirty-indicator", "aria-label": t(lang, "status.unsaved"), "●" }
            }
            div {
                class: "toolbar-actions",
                onclick: move |event| event.stop_propagation(),
                button {
                    class: "toolbar-icon-btn",
                    disabled: !search_available,
                    title: "{search_title}",
                    "aria-label": "{search_title}",
                    onclick: move |_| {
                        if search_available {
                            settings_open.set(false);
                            on_search.call(());
                        }
                    },
                    "\u{2315}"
                }
                div {
                    class: "toolbar-settings",
                    button {
                        class: "toolbar-icon-btn",
                        title: t(lang, "menu.settings"),
                        "aria-label": t(lang, "menu.settings"),
                        "aria-haspopup": "menu",
                        "aria-expanded": "{settings_expanded}",
                        onclick: move |_| {
                            let open = *settings_open.read();
                            settings_open.set(!open);
                        },
                        "\u{2699}"
                    }
                    if settings_expanded {
                        div {
                            class: "toolbar-settings-menu",
                            role: "menu",
                            "aria-label": t(lang, "menu.settings"),
                            tabindex: 0,
                            onclick: move |event| event.stop_propagation(),
                            onkeydown: move |event| {
                                if event.data().key() == Key::Escape {
                                    event.stop_propagation();
                                    event.prevent_default();
                                    settings_open.set(false);
                                }
                            },
                            label { class: "toolbar-settings-label", {t(lang, "menu.language")} }
                            select {
                                class: "toolbar-locale",
                                "aria-label": t(lang, "menu.language"),
                                onchange: move |event| {
                                    if let Some(picked) = Locale::from_tag(&event.value()) {
                                        locale.set(picked);
                                    }
                                },
                                for entry in Locale::ALL {
                                    option {
                                        value: entry.tag(),
                                        selected: *entry == lang,
                                        {entry.native_name()}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
