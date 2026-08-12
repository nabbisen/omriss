//! Shared draft-commit helpers for the Document Map panel and its row
//! menus — thin wrappers over `shell::draft_sync` (RFC-054 J7b), the one
//! format-aware implementation, since this module only has bare signal
//! props, not a full `AppCtx`.

use dioxus::prelude::*;
use omriss_ui::EditorSession;

use crate::shell::draft_sync;

pub(super) fn commit_draft_if_dirty(
    session: &mut Signal<EditorSession>,
    draft: &mut Signal<String>,
    status: &mut Signal<String>,
) -> bool {
    draft_sync::commit_or_block(session, draft, status)
}

pub(super) fn sync_draft(session: &Signal<EditorSession>, draft: &mut Signal<String>) {
    draft_sync::sync(session, draft);
}
