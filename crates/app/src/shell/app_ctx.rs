//! Shared application context bundled as a `Copy` struct for clean
//! handoff between the `App` component and free-function action/dispatch
//! handlers in `actions.rs` and `dispatch.rs`.

use std::time::SystemTime;

use dioxus::prelude::*;
use omriss_ui::EditorSession;

use crate::components::SectionTitleAction;
use crate::shell::draft_sync;

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
//
// Thin wrappers over `draft_sync` (RFC-054 J7b), which owns the one
// format-aware implementation shared with `document_map_pane/draft.rs`
// and `toolbar.rs`.

/// Syncs the draft editor buffer to the committed content of the focused
/// node — Markdown's section body, or (RFC-054 J7b) a focused JSON node's
/// current editable/raw text.
pub(crate) fn sync_draft(mut ctx: AppCtx) {
    draft_sync::sync(&ctx.session, &mut ctx.draft);
}

/// True when the Writing Area contains text that has not yet been applied
/// to the canonical document session — Markdown's body, or (RFC-054 J7b)
/// a focused JSON node's draft.
pub(crate) fn has_pending_draft(ctx: AppCtx) -> bool {
    draft_sync::is_pending(&ctx.session, &ctx.draft)
}

/// Commits any pending draft into `omriss` before a save or navigation.
pub(crate) fn commit_pending(mut ctx: AppCtx) -> bool {
    draft_sync::commit_or_block(&mut ctx.session, &mut ctx.draft, &mut ctx.status)
}
