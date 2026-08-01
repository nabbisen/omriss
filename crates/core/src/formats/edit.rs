//! Edit model: focused replacement and structure commands (RFC-053 §9).

use crate::{ByteRange, DocumentRevision, EditResult, NodeId};

/// A focused-content draft, validated and ready to apply (RFC-053 §9.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedEdit {
    pub node_id: NodeId,
    pub base_revision: DocumentRevision,
    pub replacement_range: ByteRange,
    pub replacement_text: String,
    pub description: EditDescription,
}

/// What kind of edit a [`ValidatedEdit`] represents.
///
/// Kept to exactly the one case that exists today: focused body/value
/// replacement. Structural edits go through [`StructureCommand`] instead,
/// which does not use this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditDescription {
    FocusedContentReplacement,
}

/// The result of applying a [`ValidatedEdit`] or a [`StructureCommand`]
/// (RFC-053 §9).
///
/// Wraps the shipped [`EditResult`] rather than duplicating its fields:
/// every adapter mutation ultimately goes through `Document`'s existing
/// replacement path (RFC-053 §7.1), which already returns this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedEdit {
    pub result: EditResult,
}

/// A structural command (RFC-053 §9.2). One vocabulary for every format;
/// the Markdown adapter maps each onto its already-shipped RFC-023/024/025
/// operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureCommand {
    AddInside {
        target: NodeId,
        spec: NewNodeSpec,
    },
    AddAfter {
        target: NodeId,
        spec: NewNodeSpec,
    },
    Rename {
        target: NodeId,
        new_name: String,
    },
    Move {
        target: NodeId,
        direction: MoveDirection,
    },
    JoinWithPrevious {
        target: NodeId,
    },
    Delete {
        target: NodeId,
    },
}

/// Where a [`StructureCommand::Move`] goes (RFC-053 §9.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
    /// Markdown: demote (RFC-023).
    InsidePrevious,
    /// Markdown: promote (RFC-023).
    OutOneLevel,
}

/// The minimal description of a new node for
/// [`StructureCommand::AddInside`] / [`StructureCommand::AddAfter`].
///
/// RFC-053 does not define this type's shape. Kept to exactly what every
/// shipped "add section" call site needs — a title — since level,
/// position, and everything else format-specific is derived by the adapter
/// itself (for Markdown: incrementing the parent's heading level for
/// `AddInside`, reusing the target's own level for `AddAfter`, mirroring
/// the shipped `omriss-ui` behavior in `session/structural.rs` and
/// `shell/actions.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewNodeSpec {
    pub title: String,
}
