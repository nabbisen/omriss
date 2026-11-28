//! What a finished save does to the document and to the status line (RFC-068 §8.2).
//!
//! One function for both ways a save can end up here (Save, and Overwrite in the
//! external-change dialog), so they cannot disagree, and so the rule can be tested
//! without a window: **a file that was written is saved.** Whether the disk also
//! confirmed it is a warning on a save, never a reason to call it a failure.

use omriss_ui::EditorSession;

use crate::file::durable_write::Durability;
use crate::file::file_dialog::SaveOutcome;
use crate::shell::failure_text::save_failure_key;

/// Applies `outcome` to the session, and returns the status key to show (`None` when
/// the user cancelled and nothing happened).
///
/// - `Saved`, confirmed: the document is marked saved and the file's stamp recorded, so
///   the next save compares with what this one wrote; "Saved".
/// - `Saved`, not confirmed: the same; the status says the file was saved but the disk
///   did not confirm it.
/// - `Failed`: nothing changes; the status says why, from the typed reason.
pub(crate) fn apply_save_outcome(
    session: &mut EditorSession,
    outcome: SaveOutcome,
) -> Option<&'static str> {
    match outcome {
        SaveOutcome::Cancelled => None,
        SaveOutcome::Failed { kind } => Some(save_failure_key(kind)),
        SaveOutcome::Saved {
            path,
            stamp,
            durability,
        } => {
            session.mark_saved(Some(path));
            session.set_disk_stamp(Some(stamp));
            Some(match durability {
                Durability::Confirmed => "status.saved",
                Durability::Unconfirmed { .. } => "status.saved_unconfirmed",
            })
        }
    }
}

#[cfg(test)]
mod tests;
