//! Shared preflight checks for structural editing operations.

use crate::doc::revision::DocumentRevision;
use crate::index::outline::NodeId;
use crate::{Document, Outline};

use super::error::StructuralEditError;

pub(super) fn check_revision(
    doc: &Document,
    base: DocumentRevision,
) -> Result<(), StructuralEditError> {
    if doc.revision() != base {
        return Err(StructuralEditError::RevisionMismatch {
            expected: doc.revision(),
            actual: base,
        });
    }
    Ok(())
}

pub(super) fn is_descendant(outline: &Outline, ancestor: NodeId, candidate: NodeId) -> bool {
    outline
        .path(candidate)
        .map(|path| path.iter().any(|n| n.id == ancestor))
        .unwrap_or(false)
}
