//! Shared application context bundled as a `Copy` struct for clean
//! handoff between the `App` component and free-function action/dispatch
//! handlers in `actions.rs` and `dispatch.rs`.

use std::time::SystemTime;

use dioxus::prelude::*;
use omriss_ui::EditorSession;

use crate::components::SectionTitleAction;

// ── Modal state ──────────────────────────────────────────────────────────────

/// Which blocking dialog (if any) is currently visible.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Modal {
    #[default]
    None,
    /// Unsaved-changes guard — pending action runs after the user decides.
    UnsavedBeforeOpen,
    UnsavedBeforeNew,
    /// External file modification detected before overwriting the disk file.
    ExternalModified,
    /// Confirm deletion of a section and its subtree (RFC-025).
    ConfirmDelete {
        title: String,
        child_count: usize,
    },
    /// Collect a title for a Document Map structure action.
    SectionTitle {
        action: SectionTitleAction,
        initial_title: String,
    },
    /// File open failed — surface cause and recovery options (RFC-039).
    OpenError {
        cause: String,
    },
}

// ── AppCtx ───────────────────────────────────────────────────────────────────

/// All signals that action handlers and the keyboard dispatcher need.
/// `Signal<T>: Copy`, so this struct is `Copy` and threads cleanly into
/// free functions without reference lifetimes.
#[derive(Clone, Copy)]
pub(crate) struct AppCtx {
    pub session: Signal<EditorSession>,
    pub draft: Signal<String>,
    pub status: Signal<String>,
    pub selected_card: Signal<usize>,
    pub modal: Signal<Modal>,
    pub saved_mtime: Signal<Option<SystemTime>>,
    pub recent_files: Signal<Vec<String>>,
}

// ── Shared helpers ───────────────────────────────────────────────────────────

/// Syncs the draft editor buffer to the committed body of the focused section.
pub(crate) fn sync_draft(ctx: AppCtx) {
    let mut draft = ctx.draft;
    let body = ctx
        .session
        .read()
        .current_snapshot()
        .map(|s| s.body)
        .unwrap_or_default();
    draft.set(body);
}

/// True when the Writing Area contains text that has not yet been applied to
/// the canonical document session.
pub(crate) fn has_pending_draft(ctx: AppCtx) -> bool {
    let draft = ctx.draft.read().clone();
    ctx.session
        .read()
        .current_snapshot()
        .is_some_and(|snapshot| draft != snapshot.body)
}

/// Commits any pending draft into `omriss` before a save or navigation.
pub(crate) fn commit_pending(mut ctx: AppCtx) -> bool {
    let snap = ctx.session.read().current_snapshot();
    if let Some(snapshot) = snap {
        let d = ctx.draft.read().clone();
        if d != snapshot.body {
            return match ctx.session.write().commit_focused_body(&snapshot, d) {
                Ok(_) => true,
                Err(_) => {
                    ctx.status.set("error.stale_edit".into());
                    false
                }
            };
        }
    }
    true
}
