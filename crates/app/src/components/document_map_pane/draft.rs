//! Shared draft-commit helpers for the Document Map panel and its row menus.

use dioxus::prelude::*;
use omriss_ui::EditorSession;

pub(super) fn commit_draft_if_dirty(
    session: &mut Signal<EditorSession>,
    draft: &mut Signal<String>,
    status: &mut Signal<String>,
) -> bool {
    let snap = session.read().current_snapshot();
    let Some(s) = snap else { return true };
    let d = draft.read().clone();
    if d != s.body && session.write().commit_focused_body(&s, d).is_err() {
        status.set("error.stale_edit".into());
        return false;
    }
    true
}

pub(super) fn sync_draft(session: &Signal<EditorSession>, draft: &mut Signal<String>) {
    let body = session
        .read()
        .current_snapshot()
        .map(|s| s.body)
        .unwrap_or_default();
    draft.set(body);
}
