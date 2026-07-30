//! Structural edit error taxonomy (RFC-026 conflict taxonomy).

use crate::doc::revision::DocumentRevision;
use crate::error::{EditError, IndexError};
use crate::index::outline::NodeId;

/// Errors raised by structural editing operations (RFC-026 conflict taxonomy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralEditError {
    /// The edit was based on a stale revision; caller must reload (RFC-002).
    RevisionMismatch {
        expected: DocumentRevision,
        actual: DocumentRevision,
    },
    /// Source node no longer exists in the current outline.
    StaleNode(NodeId),
    /// Target node no longer exists in the current outline (move ops).
    StaleTarget(NodeId),
    /// Setext headings are not supported for promote/demote in M5.
    UnsupportedHeadingStyle,
    /// Cannot promote an H1 or demote an H6.
    InvalidLevel,
    /// Cannot move a section into one of its own descendants.
    CannotMoveIntoDescendant,
    /// Cannot move a section before/after itself.
    CannotMoveSelf,
    /// The operation requires a focused section, but no section is focused.
    NoFocusedSection,
    /// Cannot delete the synthetic root node.
    CannotDeleteRoot,
    /// No adjacent sibling of the right kind to merge with.
    NoAdjacentSibling,
    /// Operation would risk moving content under the wrong section.
    UnsafePreservation,
    /// Split offset falls outside the section's body range.
    InvalidSplitOffset,
    /// Section title is empty or cannot be represented safely.
    InvalidTitle,
    /// The underlying text replacement failed (re-index error or invalid range).
    Edit(EditError),
}

impl std::fmt::Display for StructuralEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionMismatch { expected, actual } => write!(
                f,
                "revision mismatch: document is at {expected:?}, edit based on {actual:?}"
            ),
            Self::StaleNode(id) => write!(f, "source node stale: {id:?}"),
            Self::StaleTarget(id) => write!(f, "target node stale: {id:?}"),
            Self::UnsupportedHeadingStyle => {
                write!(
                    f,
                    "promote/demote requires ATX headings (# …); convert setext first"
                )
            }
            Self::InvalidLevel => write!(
                f,
                "heading level limit reached (H1 cannot be promoted; H6 cannot be demoted)"
            ),
            Self::CannotMoveIntoDescendant => {
                write!(f, "cannot move a section into its own descendant")
            }
            Self::CannotMoveSelf => write!(f, "source and target are the same section"),
            Self::NoFocusedSection => write!(f, "no focused section"),
            Self::CannotDeleteRoot => write!(f, "cannot delete the root node"),
            Self::NoAdjacentSibling => write!(f, "no adjacent sibling to merge with"),
            Self::UnsafePreservation => write!(f, "operation could change unrelated structure"),
            Self::InvalidSplitOffset => write!(f, "split offset is outside the section body"),
            Self::InvalidTitle => write!(f, "section title is empty or invalid"),
            Self::Edit(e) => write!(f, "underlying edit error: {e}"),
        }
    }
}

impl std::error::Error for StructuralEditError {}

impl From<EditError> for StructuralEditError {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}

impl From<IndexError> for StructuralEditError {
    fn from(e: IndexError) -> Self {
        Self::Edit(EditError::IndexAfterEdit(e))
    }
}
