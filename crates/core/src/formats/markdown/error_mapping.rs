//! Maps shipped edit/structural errors onto the RFC-053 §11 error taxonomy.

use crate::formats::error::{ApplyEditError, StructureCommandError, StructureErrorKind};
use crate::{EditError, StructuralEditError};

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

fn map_structural_error(error: StructuralEditError) -> StructureErrorKind {
    match error {
        StructuralEditError::RevisionMismatch { .. }
        | StructuralEditError::StaleNode(_)
        | StructuralEditError::StaleTarget(_)
        | StructuralEditError::CannotMoveIntoDescendant
        | StructuralEditError::CannotMoveSelf
        | StructuralEditError::NoFocusedSection
        | StructuralEditError::CannotDeleteRoot
        | StructuralEditError::NoAdjacentSibling
        | StructuralEditError::UnsafePreservation
        | StructuralEditError::InvalidSplitOffset
        | StructuralEditError::InvalidTitle => StructureErrorKind::UnsafeRange,
        StructuralEditError::UnsupportedHeadingStyle | StructuralEditError::InvalidLevel => {
            StructureErrorKind::UnsupportedFeature
        }
        StructuralEditError::Edit(edit_error) => map_edit_error(edit_error).kind,
    }
}

impl From<StructuralEditError> for StructureCommandError {
    fn from(error: StructuralEditError) -> Self {
        Self {
            kind: map_structural_error(error),
        }
    }
}
