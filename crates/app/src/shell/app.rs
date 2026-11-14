//! Root Dioxus component — signal wiring and render tree.
//!
//! Action logic lives in [`actions`]. Keyboard/palette dispatch lives in
//! [`dispatch`]. Modal enum and signal helpers live in [`app_ctx`].

use std::time::SystemTime;

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{EditorSession, ViewMode};

use crate::file::file_dialog;
use crate::input::keyboard;
use crate::shell::actions::{
    handle_confirm_delete, handle_ext_modified_choice, handle_load, handle_new_guarded,
    handle_open_guarded, handle_save, handle_section_title_choice, handle_unsaved_choice,
};
use crate::shell::app_ctx::{AppCtx, Modal};
use crate::shell::dispatch::{dispatch_command, dispatch_palette, open_search_if_available};
use crate::storage::settings::AppSettings;
use omriss_components::draft_sync;
use omriss_components::{
    CommandPalette, ConfirmDeleteChoice, ConfirmDeleteDialog, DocumentMapPane, ErrorDialog,
    ExtModifiedChoice, ExtModifiedDialog, FocusedContentPane, OverviewPane, RawSourceView,
    SearchPanel, SectionTitleAction, SectionTitleChoice, SectionTitleDialog, StatusBar,
    StructureDialog, StructureRequest, Toolbar, UnsavedChoice, UnsavedDialog, WelcomeScreen,
    focus_active_modal, focus_active_palette, trap_modal_tab, use_structure_requests,
};

const STYLE: &str = include_str!("../../assets/style.css");

#[component]
pub fn App() -> Element {
    let initial_locale = try_consume_context::<Locale>().unwrap_or_default();
    let initial_settings = try_consume_context::<AppSettings>().unwrap_or_default();
    let startup_arg =
        try_consume_context::<crate::cli::StartupArg>().unwrap_or(crate::cli::StartupArg::None);

    // ── signals ───────────────────────────────────────────────────────────────

    let session: Signal<EditorSession> = use_signal(EditorSession::new_empty);
    let locale = use_signal(move || initial_locale);
    let draft = use_signal(String::new);
    let status = use_signal(|| "status.ready".to_string());
    let selected_card = use_signal(|| 0usize);
    let modal: Signal<Modal> = use_signal(Modal::default);
    let saved_mtime: Signal<Option<SystemTime>> = use_signal(|| None);
    let recent_files = use_signal(move || initial_settings.valid_recent_files());
    let search_open = use_signal(|| false);
    let palette_open = use_signal(|| false);
    let preview_open = use_signal(|| false);
    let structure_request: Signal<Option<StructureRequest>> = use_signal(|| None);

    let ctx = AppCtx {
        session,
        draft,
        status,
        selected_card,
        modal,
        saved_mtime,
        recent_files,
    };

    // ── callbacks — thin wrappers around free functions ───────────────────────

    let do_load = use_callback(move |outcome| handle_load(outcome, ctx));
    let do_open_guarded = use_callback(move |()| handle_open_guarded(ctx));
    let do_save = use_callback(move |()| handle_save(ctx, false));
    let do_save_as = use_callback(move |()| handle_save(ctx, true));
    let do_new_guarded = use_callback(move |()| handle_new_guarded(ctx));

    // RFC-063: open the file named on the command line, if any, through the
    // identical `open_markdown_path` -> `handle_load` route the Recent
    // Files entry already takes.
    //
    // RFC063-IMPL-001 (Minor, `.git-exclude/reviewed/009-rfc-063-command-line-file-argument-implementation-review.md`):
    // "runs exactly once" used to be emergent from the closure reading no
    // signal, not structural -- a future edit adding any signal read,
    // directly or through a helper, would have silently turned a one-time
    // startup load into a repeated one, discarding unsaved edits on
    // regression. `startup_load_done` makes the once-ness structural
    // instead: the guard is checked *before* the match, so no matter what
    // the match arms come to read later, they still execute at most once.
    // (This self-triggers one extra, harmless re-run -- the `set(true)`
    // is itself a read `use_effect` subscribes to -- which the guard exits
    // from immediately.)
    let mut startup_load_done = use_signal(|| false);
    use_effect(move || {
        if *startup_load_done.read() {
            return;
        }
        startup_load_done.set(true);
        match &startup_arg {
            crate::cli::StartupArg::None => {}
            crate::cli::StartupArg::Path(path) => {
                do_load.call(file_dialog::open_markdown_path(path));
            }
            crate::cli::StartupArg::RejectedOption(_) => {
                let mut modal = modal;
                modal.set(Modal::OpenError {
                    cause: "omriss takes a file path and accepts no options.".into(),
                });
            }
        }
    });

    let on_unsaved_choice =
        use_callback(move |choice: UnsavedChoice| handle_unsaved_choice(choice, ctx));
    let on_ext_modified_choice =
        use_callback(move |choice: ExtModifiedChoice| handle_ext_modified_choice(choice, ctx));
    let on_confirm_delete_choice =
        use_callback(move |choice: ConfirmDeleteChoice| handle_confirm_delete(choice, ctx));
    let on_section_title_choice = use_callback(
        move |(action, choice): (SectionTitleAction, SectionTitleChoice)| {
            handle_section_title_choice(action, choice, ctx)
        },
    );

    let on_keydown = use_callback(move |event: Event<KeyboardData>| {
        let active_modal = modal.read().clone();
        if !matches!(active_modal, Modal::None) {
            if trap_modal_tab(&event) {
                return;
            }
            if matches!(event.key(), Key::Escape) {
                event.stop_propagation();
                event.prevent_default();
                match active_modal {
                    Modal::UnsavedBeforeOpen | Modal::UnsavedBeforeNew => {
                        on_unsaved_choice.call(UnsavedChoice::Cancel);
                    }
                    Modal::ExternalModified => {
                        on_ext_modified_choice.call(ExtModifiedChoice::Cancel);
                    }
                    Modal::ConfirmDelete { .. } => {
                        on_confirm_delete_choice.call(ConfirmDeleteChoice::Cancel);
                    }
                    Modal::SectionTitle { action, .. } => {
                        on_section_title_choice.call((action, SectionTitleChoice::Cancel));
                    }
                    Modal::OpenError { .. } => {
                        let mut modal = modal;
                        modal.set(Modal::None);
                    }
                    Modal::None => {}
                }
            }
            return;
        }

        if *palette_open.read() {
            if matches!(
                keyboard::interpret(&event.data()),
                Some(keyboard::AppCommand::OpenPalette)
            ) {
                event.stop_propagation();
                event.prevent_default();
                let mut po = palette_open;
                po.set(false);
                return;
            }
            if matches!(event.key(), Key::Escape) {
                event.stop_propagation();
                event.prevent_default();
                let mut po = palette_open;
                po.set(false);
                return;
            }
            if !matches!(
                event.key(),
                Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End | Key::Enter | Key::Tab
            ) {
                focus_active_palette();
            }
            return;
        }

        let Some(cmd) = keyboard::interpret(&event.data()) else {
            return;
        };
        let mode = session.read().view_mode();
        dispatch_command(cmd, mode, ctx, search_open, palette_open, preview_open);
    });

    let on_palette_execute =
        use_callback(move |id| dispatch_palette(id, ctx, search_open, preview_open));

    use_effect(move || {
        if !matches!(*modal.read(), Modal::None) {
            focus_active_modal();
        }
    });

    // ── Document Map structure requests (RFC-069 §2.5) ────────────────────────
    // The Document Map posts a typed `StructureRequest`; it is resolved to a
    // dialog in an effect, never during render, and `status` stays display-only.
    let on_structure_dialog = use_callback(move |dialog: StructureDialog| {
        let mut modal = modal;
        match dialog {
            StructureDialog::ConfirmDelete { title, child_count } => {
                modal.set(Modal::ConfirmDelete { title, child_count });
            }
            StructureDialog::SectionTitle {
                action,
                initial_title,
            } => {
                modal.set(Modal::SectionTitle {
                    action,
                    initial_title,
                });
                // Focus the dialog input after the DOM fully settles. The
                // button's `session.write().focus()` triggers DocumentMapPane's
                // use_effect -> item_tree write -> another render+patch inside
                // the first rAF; a second rAF fires after that patch.
                spawn(async move {
                    let _ = document::eval(
                        "requestAnimationFrame(() => requestAnimationFrame(() => document.querySelector('.split-dialog-input')?.focus()))",
                    );
                });
            }
        }
    });
    use_structure_requests(session, structure_request, status, on_structure_dialog);

    let is_welcome = !session.read().document_open();
    let is_raw = session.read().is_raw();
    let mode = session.read().view_mode();

    // ── render tree ───────────────────────────────────────────────────────────

    rsx! {
        style { {STYLE} }
        div {
            class: "app",
            tabindex: 0,
            onkeydown: move |event| on_keydown.call(event),

            Toolbar {
                session, locale, draft, status,
                on_open: move |()| do_open_guarded.call(()),
                on_save: move |()| do_save.call(()),
                on_save_as: move |()| do_save_as.call(()),
                search_available: !is_welcome,
                on_search: move |()| {
                    open_search_if_available(ctx, search_open);
                },
            }

            if is_welcome {
                WelcomeScreen {
                    locale,
                    recent_files,
                    on_open: move |()| do_open_guarded.call(()),
                    on_new: move |()| do_new_guarded.call(()),
                    on_open_recent: move |path: String| {
                        do_load.call(file_dialog::open_markdown_path(&path));
                    },
                }
            } else {
                div { class: "body",
                    // RFC-049: Document Map is the single structure-organization surface.
                    DocumentMapPane { session, locale, draft, status, request: structure_request }
                    if is_raw {
                        RawSourceView {
                            session,
                            locale,
                            draft,
                            status,
                        }
                    } else {
                        match mode {
                            ViewMode::Outline | ViewMode::RawSource => rsx! {
                                OverviewPane { session, locale, draft, selected_card }
                            },
                            // RFC-050: FocusedContentPane has no structure controls.
                            ViewMode::Focus(_) => rsx! {
                                FocusedContentPane { session, locale, draft, status, preview_open }
                            },
                        }
                    }
                }
            }

            StatusBar {
                session, locale, status,
                on_save_as: move |()| do_save_as.call(()),
            }

            if *search_open.read() {
                SearchPanel {
                    session, locale,
                    on_close: move |()| { let mut so = search_open; so.set(false); },
                    on_navigate: move |id| {
                        let mut session = session;
                        let mut draft = draft;
                        let mut status = status;
                        if !draft_sync::commit_or_block(&mut session, &mut draft, &mut status) {
                            return;
                        }
                        let _ = session.write().focus(id);
                        draft_sync::sync(&session, &mut draft);
                    },
                }
            }

            if *palette_open.read() {
                CommandPalette {
                    locale,
                    on_close: move |()| { let mut po = palette_open; po.set(false); },
                    on_execute: move |id| on_palette_execute.call(id),
                }
            }

            match *modal.read() {
                Modal::None => rsx! {},
                Modal::UnsavedBeforeOpen | Modal::UnsavedBeforeNew => rsx! {
                    UnsavedDialog {
                        locale,
                        on_choice: move |choice| on_unsaved_choice.call(choice),
                    }
                },
                Modal::ExternalModified => rsx! {
                    ExtModifiedDialog {
                        locale,
                        on_choice: move |choice| on_ext_modified_choice.call(choice),
                    }
                },
                Modal::ConfirmDelete { ref title, child_count } => rsx! {
                    ConfirmDeleteDialog {
                        locale,
                        section_title: title.clone(),
                        child_count,
                        on_choice: move |c| on_confirm_delete_choice.call(c),
                    }
                },
                Modal::SectionTitle { action, ref initial_title } => rsx! {
                    SectionTitleDialog {
                        locale,
                        action,
                        initial_title: initial_title.clone(),
                        on_choice: move |c| on_section_title_choice.call((action, c)),
                    }
                },
                Modal::OpenError { ref cause } => rsx! {
                    ErrorDialog {
                        locale,
                        title: "error.open_failed".to_string(),
                        cause: cause.clone(),
                        dismiss_label: t(*locale.read(), "dialog.discard.cancel").to_string(),
                        secondary_label: None,
                        on_dismiss: move |()| { let mut m = modal; m.set(Modal::None); },
                        on_secondary: None,
                    }
                },
            }
        }
    }
}
