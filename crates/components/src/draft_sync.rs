//! The single format-aware "is there a pending draft, commit it before
//! navigating, sync it after focus changes" implementation (RFC-054 J7b).
//!
//! Before this file, the same three-function shape existed independently
//! in `app_ctx.rs` (bundled `AppCtx`), `document_map_pane/draft.rs`
//! (bare signals, no `AppCtx` available there), and `toolbar.rs`'s own
//! inline `undo`/`redo`/`back`/`forward` closures — three copies of one
//! rule is one too many; this is the single implementation all three now
//! call. Markdown's half is the pre-existing logic, byte-identical;
//! non-Markdown routes to `EditorSession`'s RFC-054 J7b structured-draft
//! methods instead.

use dioxus::prelude::*;
use omriss_core::{DocumentFormat, FocusedContent};
use omriss_ui::{EditorSession, StructureErrorKindCatalogKey};

/// RFC-054 §10's context-specific message for a JSON edit failure, chosen
/// generically — without value-kind context, since every caller here is a
/// "commit whatever's pending before navigating" safety net, not the
/// editor widget itself. The more specific per-kind message (e.g. "This
/// number is not valid yet.") is shown inline by
/// `focused_content_pane/structured.rs`'s editor, live, as the user
/// types — the signal this generic fallback rarely needs to improve on.
pub fn json_edit_error_key(kind: omriss_core::StructureErrorKind) -> &'static str {
    use omriss_core::StructureErrorKind;
    match kind {
        StructureErrorKind::InvalidSyntax => "focused_content.json.error.invalid_raw_value",
        StructureErrorKind::UnsafeRange => "focused_content.json.error.unsafe_change",
        StructureErrorKind::UnsupportedFeature => "focused_content.json.error.unsupported_change",
        // RFC-054 §10 has no row for these; falling back to J4's generic
        // RFC-053 §11 table is exactly what that table's own note
        // describes as correct ("used wherever only the kind is known"),
        // not a shortcut around a §10 row that exists.
        StructureErrorKind::TooLarge | StructureErrorKind::InternalInvariantFailed => {
            kind.catalog_key()
        }
    }
}

/// True when `draft` differs from the focused node's currently committed
/// content — Markdown's section body, or a JSON node's editable/raw text.
pub fn is_pending(session: &Signal<EditorSession>, draft: &Signal<String>) -> bool {
    let s = session.read();
    // In raw edit mode the one live draft is the raw text (RFC-076 §3.3): the
    // section draft is committed before the raw view opens, so the two can
    // never both be pending.
    if s.raw_editing() {
        return s.raw_draft_differs(&draft.read());
    }
    if s.format() == DocumentFormat::Markdown {
        let d = draft.read();
        s.current_snapshot().is_some_and(|snap| *d != snap.body)
    } else {
        s.structured_draft_differs(&draft.read())
    }
}

/// Commits `draft` into the session if it differs from the committed
/// content; returns `false` (and sets `status`) if the commit fails, so
/// the caller can block whatever navigation it was about to do.
pub fn commit_or_block(
    session: &mut Signal<EditorSession>,
    draft: &mut Signal<String>,
    status: &mut Signal<String>,
) -> bool {
    if session.read().raw_editing() {
        let d = draft.read().clone();
        if !session.read().raw_draft_differs(&d) {
            return true;
        }
        let result = session.write().commit_raw_draft(&d);
        return match result {
            Ok(_) => true,
            Err(_) => {
                status.set("error.stale_edit".into());
                false
            }
        };
    }
    if session.read().format() != DocumentFormat::Markdown {
        let d = draft.read().clone();
        if !session.read().structured_draft_differs(&d) {
            return true;
        }
        let result = session.write().commit_structured_draft(&d);
        return match result {
            Ok(_) => true,
            Err(kind) => {
                status.set(json_edit_error_key(kind).into());
                false
            }
        };
    }
    let snap = session.read().current_snapshot();
    let Some(s) = snap else { return true };
    let d = draft.read().clone();
    if d != s.body && session.write().commit_focused_body(&s, d).is_err() {
        status.set("error.stale_edit".into());
        return false;
    }
    true
}

/// [`commit_or_block`], then — on success — [`sync`] the draft to what was
/// actually committed (RFC-069 §2.1).
///
/// The resync is the point. A commit stores the body the *session* decided on —
/// a non-last section's body gets a newline appended — while the draft signal
/// still holds what the user typed. Without the resync `is_pending` stays true
/// forever, so the dirty dot stays lit after a byte-correct save, and every
/// later navigation, preview toggle or search re-commits an identical body: a
/// no-op that still bumps the revision and pushes an undo record, so the next
/// Ctrl+Z appears to do nothing. Every caller that commits and then carries on
/// must therefore go through this, not through `commit_or_block` alone.
pub fn commit_and_resync(
    session: &mut Signal<EditorSession>,
    draft: &mut Signal<String>,
    status: &mut Signal<String>,
) -> bool {
    if !commit_or_block(session, draft, status) {
        return false;
    }
    sync(session, draft);
    true
}

/// Syncs `draft` to the focused node's current committed content —
/// Markdown's section body, or (RFC-054 J7b) a JSON node's editable text
/// (a value) or full raw text (a container).
pub fn sync(session: &Signal<EditorSession>, draft: &mut Signal<String>) {
    if session.read().raw_editing() {
        // Reset the raw draft to the document and record the revision it is
        // now composed against.
        let mut session = *session;
        let text = session.write().reload_raw_draft();
        draft.set(text);
        return;
    }
    let s = session.read();
    let text = if s.format() == DocumentFormat::Markdown {
        s.current_snapshot().map(|snap| snap.body)
    } else {
        match s.focused_structured_content() {
            Some(FocusedContent::StructuredValue { editable_text, .. }) => Some(editable_text),
            Some(FocusedContent::StructuredGroup { .. }) => s.focused_structured_raw_text(),
            _ => None,
        }
    };
    drop(s);
    draft.set(text.unwrap_or_default());
}

#[cfg(test)]
mod tests;
