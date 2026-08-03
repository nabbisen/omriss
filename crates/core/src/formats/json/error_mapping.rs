//! Maps `Document::replace_range`'s shipped `EditError` onto the RFC-053
//! §11 error taxonomy. Mirrors `markdown::error_mapping::map_edit_error`
//! exactly: `replace_range` (J1) and `replace_section_body` both funnel
//! through the same `Document::apply_replacement` transactional path
//! (RFC-053 §7.1), so the same `EditError` variants apply to both, with
//! the same mapping.

use crate::EditError;
use crate::formats::error::{ApplyEditError, StructureErrorKind};

pub(super) fn map_edit_error(error: EditError) -> ApplyEditError {
    let kind = match error {
        EditError::RevisionMismatch { .. }
        | EditError::StaleNode(_)
        | EditError::InvalidRange(_) => StructureErrorKind::UnsafeRange,
        EditError::IndexAfterEdit(_) | EditError::NothingToUndo | EditError::NothingToRedo => {
            StructureErrorKind::InternalInvariantFailed
        }
    };
    ApplyEditError { kind }
}
