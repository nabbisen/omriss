//! RFC-069 §2.5: the Document Map asks its host to open a dialog with a typed
//! request, not with a string written into the status line.
//!
//! Before this, the Document Map wrote five sentinel strings (one per action,
//! all in a `struct.*` namespace) into the **user-visible status signal**, and
//! the root matched them while rendering and wrote signals from the render
//! body. Status was simultaneously a message and a command bus, and a typo in a sentinel was a
//! silent no-op. A [`StructureRequest`] is the contract between the two sides;
//! `status` is display-only again.
//!
//! The request is consumed in a `use_effect` ([`use_structure_requests`]),
//! never in render, and is set back to `None` once handled.

use dioxus::prelude::*;
use omriss_ui::EditorSession;

use crate::split_dialog::SectionTitleAction;

/// A structure action the Document Map has prepared (the draft is committed and
/// the target section focused) and wants the host to ask the user about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureRequest {
    AddTopLevel,
    AddInside,
    AddAfter,
    Rename,
    Delete,
}

/// The dialog a request opens, described without the host's modal type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureDialog {
    /// Confirm deleting the focused section and its subtree (RFC-025).
    ConfirmDelete { title: String, child_count: usize },
    /// Collect a title for an add or rename.
    SectionTitle {
        action: SectionTitleAction,
        initial_title: String,
    },
}

/// The dialog `request` opens against `session`, or `None` when there is
/// nothing to ask about (a delete or rename with no focused section).
pub fn dialog_for(request: StructureRequest, session: &EditorSession) -> Option<StructureDialog> {
    let add = |action| StructureDialog::SectionTitle {
        action,
        initial_title: String::new(),
    };
    match request {
        StructureRequest::AddTopLevel => Some(add(SectionTitleAction::AddTopLevel)),
        StructureRequest::AddInside => Some(add(SectionTitleAction::AddInside)),
        StructureRequest::AddAfter => Some(add(SectionTitleAction::AddAfter)),
        StructureRequest::Rename => Some(StructureDialog::SectionTitle {
            action: SectionTitleAction::Rename,
            initial_title: session
                .current_snapshot()
                .map(|s| s.title)
                .unwrap_or_default(),
        }),
        StructureRequest::Delete => {
            session
                .current_snapshot()
                .map(|s| StructureDialog::ConfirmDelete {
                    title: s.title,
                    child_count: s.children.len(),
                })
        }
    }
}

/// Consumes `request` in an effect: resolves it to a dialog, hands the dialog
/// to `on_dialog`, resets `status` to ready (opening a dialog supersedes
/// whatever the status line said, as the sentinel handling always did) and sets
/// the request back to `None`.
///
/// The session is only peeked, so the effect depends on `request` alone.
pub fn use_structure_requests(
    session: Signal<EditorSession>,
    mut request: Signal<Option<StructureRequest>>,
    mut status: Signal<String>,
    on_dialog: Callback<StructureDialog>,
) {
    use_effect(move || {
        let Some(pending) = *request.read() else {
            return;
        };
        request.set(None);
        let dialog = dialog_for(pending, &session.peek());
        status.set("status.ready".into());
        if let Some(dialog) = dialog {
            on_dialog.call(dialog);
        }
    });
}

#[cfg(test)]
mod tests;
