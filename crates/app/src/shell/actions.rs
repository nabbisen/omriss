//! Action handlers — stateless free functions that act on [`AppCtx`] signals.
//!
//! Each function corresponds to one `use_callback` in [`super::app`].
//! Extracting them here keeps `app.rs` to signal wiring and the render tree.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::EditorSession;

use crate::components::{
    ConfirmDeleteChoice, ExtModifiedChoice, SectionTitleAction, SectionTitleChoice, UnsavedChoice,
};
use crate::file::file_dialog::{self, OpenOutcome, SaveOutcome};
use crate::shell::app_ctx::{AppCtx, Modal, commit_pending, has_pending_draft, sync_draft};
use crate::storage::settings::AppSettings;

// ── File operations ──────────────────────────────────────────────────────────

/// Handle the result of an open-file operation.
pub(crate) fn handle_load(outcome: OpenOutcome, mut ctx: AppCtx) {
    match outcome {
        OpenOutcome::Cancelled => {}
        OpenOutcome::Failed { cause } => {
            ctx.modal.set(Modal::OpenError { cause });
        }
        OpenOutcome::Loaded {
            text,
            name,
            profile,
            mtime,
        } => {
            // RFC-054 J3: the format governs which adapter's structure
            // document_map_nodes() projects. detect_format is extension-only
            // (RFC-052 §5.1); a real path is available here (unlike inside
            // omriss-ui, which only ever sees `file_name: Option<String>`),
            // so detection happens at this boundary and the result is
            // supplied to the session, matching how `profile` already flows
            // in "detected upstream, passed in".
            let format = omriss_core::formats::detection::detect_format(
                Some(std::path::Path::new(&name)),
                &text,
            );
            match EditorSession::open_detected(text, Some(name.clone()), profile, format) {
                Ok(opened) => {
                    ctx.session.set(opened);
                    ctx.selected_card.set(0);
                    // Auto-focus the first section so the editor is
                    // immediately ready -- Markdown only. `outline_items`/
                    // `focus` both still read the Markdown heading parse
                    // `Document::parse` builds over any text (RFC-054
                    // §0.1); for a non-Markdown format that parse is
                    // meaningless, so auto-focusing would show garbage
                    // content instead of the Document Map this slice
                    // exists to render. Right-panel content for non-Markdown
                    // formats is out of scope until later RFC-054 slices.
                    if format == DocumentFormat::Markdown {
                        let first_id = ctx
                            .session
                            .read()
                            .outline_items()
                            .into_iter()
                            .next()
                            .map(|i| i.id);
                        if let Some(id) = first_id {
                            let _ = ctx.session.write().focus(id);
                        }
                    }
                    sync_draft(ctx);
                    ctx.saved_mtime.set(mtime);
                    ctx.status.set("status.ready".into());
                    // RFC-036: persist the path to the recent-files list.
                    let mut settings = AppSettings::load();
                    settings.push_recent(&name);
                    settings.save();
                    ctx.recent_files.set(settings.valid_recent_files());
                }
                Err(_) => ctx.modal.set(Modal::OpenError {
                    cause: "Could not parse the file structure.".into(),
                }),
            }
        }
    }
}

/// Open a file, guarded by an unsaved-changes check.
pub(crate) fn handle_open_guarded(mut ctx: AppCtx) {
    if ctx.session.read().is_dirty() || has_pending_draft(ctx) {
        ctx.modal.set(Modal::UnsavedBeforeOpen);
    } else {
        handle_load(file_dialog::open_markdown(), ctx);
    }
}

/// Save the current document (`force_new_path = true` triggers Save As).
pub(crate) fn handle_save(mut ctx: AppCtx, force_new_path: bool) {
    if !commit_pending(ctx) {
        return;
    }
    let existing = if force_new_path {
        None
    } else {
        ctx.session.read().file_name().map(|s| s.to_string())
    };
    // External modification check (RFC-015).
    if let (Some(path), Some(mtime)) = (existing.as_deref(), *ctx.saved_mtime.read())
        && file_dialog::was_modified_externally(path, mtime)
    {
        ctx.modal.set(Modal::ExternalModified);
        return;
    }
    let profile = ctx.session.read().profile().clone();
    let outcome =
        file_dialog::save_markdown(existing.as_deref(), ctx.session.read().source(), &profile);
    match outcome {
        SaveOutcome::Cancelled => {}
        SaveOutcome::Failed => ctx.status.set("error.save_failed".into()),
        SaveOutcome::Saved { name, mtime } => {
            ctx.session.write().mark_saved(Some(name));
            ctx.saved_mtime.set(mtime);
            ctx.status.set("status.saved".into());
        }
    }
}

/// Create a blank document, resetting all transient state.
pub(crate) fn handle_new(mut ctx: AppCtx) {
    ctx.session.set(EditorSession::new_document());
    ctx.selected_card.set(0);
    ctx.draft.set(String::new());
    ctx.saved_mtime.set(None);
    ctx.status.set("status.ready".into());
}

/// Create a blank document, guarded by an unsaved-changes check.
pub(crate) fn handle_new_guarded(mut ctx: AppCtx) {
    if ctx.session.read().is_dirty() || has_pending_draft(ctx) {
        ctx.modal.set(Modal::UnsavedBeforeNew);
    } else {
        handle_new(ctx);
    }
}

// ── Modal handlers ────────────────────────────────────────────────────────────

/// Handle the user's response to the "unsaved changes" dialog.
pub(crate) fn handle_unsaved_choice(choice: UnsavedChoice, mut ctx: AppCtx) {
    let pending = ctx.modal.read().clone();
    match choice {
        UnsavedChoice::Save => {
            handle_save(ctx, false);
            if !ctx.session.read().is_dirty() {
                ctx.modal.set(Modal::None);
                match pending {
                    Modal::UnsavedBeforeOpen => {
                        handle_load(file_dialog::open_markdown(), ctx);
                    }
                    Modal::UnsavedBeforeNew => handle_new(ctx),
                    _ => {}
                }
            }
        }
        UnsavedChoice::Discard => {
            ctx.modal.set(Modal::None);
            match pending {
                Modal::UnsavedBeforeOpen => handle_load(file_dialog::open_markdown(), ctx),
                Modal::UnsavedBeforeNew => handle_new(ctx),
                _ => {}
            }
        }
        UnsavedChoice::Cancel => ctx.modal.set(Modal::None),
    }
}

/// Handle the user's response to the "external modification" dialog.
pub(crate) fn handle_ext_modified_choice(choice: ExtModifiedChoice, mut ctx: AppCtx) {
    ctx.modal.set(Modal::None);
    match choice {
        ExtModifiedChoice::Overwrite => {
            if !commit_pending(ctx) {
                return;
            }
            let existing = ctx.session.read().file_name().map(|s| s.to_string());
            let profile = ctx.session.read().profile().clone();
            let outcome = file_dialog::save_markdown(
                existing.as_deref(),
                ctx.session.read().source(),
                &profile,
            );
            match outcome {
                SaveOutcome::Saved { name, mtime } => {
                    ctx.session.write().mark_saved(Some(name));
                    ctx.saved_mtime.set(mtime);
                    ctx.status.set("status.saved".into());
                }
                SaveOutcome::Failed => ctx.status.set("error.save_failed".into()),
                SaveOutcome::Cancelled => {}
            }
        }
        ExtModifiedChoice::SaveAs => handle_save(ctx, true),
        ExtModifiedChoice::Cancel => {}
    }
}

/// Handle the user's response to the section-delete confirmation.
pub(crate) fn handle_confirm_delete(choice: ConfirmDeleteChoice, mut ctx: AppCtx) {
    ctx.modal.set(Modal::None);
    if choice == ConfirmDeleteChoice::Delete {
        let del_result = ctx.session.write().delete_focused();
        match del_result {
            Ok(_) => {
                sync_draft(ctx);
                ctx.status.set("status.unsaved".into());
            }
            Err(_) => ctx.status.set("error.struct.stale_node".into()),
        }
    }
}

/// Handle the user's response to a Document Map title dialog.
pub(crate) fn handle_section_title_choice(
    action: SectionTitleAction,
    choice: SectionTitleChoice,
    mut ctx: AppCtx,
) {
    ctx.modal.set(Modal::None);
    if let SectionTitleChoice::Confirm(title) = choice {
        let has_focus = ctx.session.read().current_snapshot().is_some();

        let result = match action {
            SectionTitleAction::AddTopLevel => ctx.session.write().add_top_level_section(&title),
            SectionTitleAction::AddInside => {
                // Compute the child heading level from the focused snapshot.
                let level = ctx
                    .session
                    .read()
                    .current_snapshot()
                    .and_then(|s| s.level)
                    .map(|l| {
                        use omriss_core::HeadingLevel::*;
                        match l {
                            H1 => H2,
                            H2 => H3,
                            H3 => H4,
                            H4 => H5,
                            _ => H6,
                        }
                    })
                    .unwrap_or(omriss_core::HeadingLevel::H2);
                // append_child_to_focused inserts at full_range.end so the
                // new section always appears after all existing children.
                ctx.session.write().append_child_to_focused(&title, level)
            }
            SectionTitleAction::AddAfter => ctx.session.write().add_after_focused(&title),
            SectionTitleAction::Rename => ctx.session.write().rename_focused(&title),
        };

        match result {
            Ok(_) => {
                let new_focus = match action {
                    SectionTitleAction::AddTopLevel => ctx
                        .session
                        .read()
                        .outline_items()
                        .last()
                        .map(|item| item.id),
                    SectionTitleAction::AddInside if has_focus => ctx
                        .session
                        .read()
                        .current_snapshot()
                        .and_then(|s| s.children.last().map(|c| c.id)),
                    SectionTitleAction::AddInside => None,
                    SectionTitleAction::AddAfter if has_focus => {
                        ctx.session.read().sibling_info().next_sibling
                    }
                    SectionTitleAction::AddAfter => None,
                    SectionTitleAction::Rename => None,
                };
                if let Some(id) = new_focus {
                    let _ = ctx.session.write().focus(id);
                }
                // Sync draft to the new section's body (empty for a freshly
                // created section) so the textarea reflects reality.
                sync_draft(ctx);
                ctx.status.set("status.unsaved".into());
            }
            Err(_) => ctx.status.set("error.struct.stale_node".into()),
        }
    }
}
