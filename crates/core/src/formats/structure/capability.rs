//! Per-action capability vocabulary (RFC-053 §6).

/// Twelve per-action capability slots for one [`super::StructureNode`].
///
/// Granular per-action capability lets the Document Map disable exactly one
/// menu item (e.g. allow `can_move_down` but not `can_move_up`) instead of
/// gating all movement together. Capabilities are populated by an adapter's
/// `build_structure` (RFC-053 §7), which has full source and tree context,
/// so there is no separate stateless lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCapabilities {
    pub can_select: Capability,
    pub can_edit_content: Capability,
    pub can_add_inside: Capability,
    pub can_add_after: Capability,
    pub can_rename: Capability,
    pub can_move_up: Capability,
    pub can_move_down: Capability,
    pub can_move_inside_previous: Capability,
    pub can_move_out_one_level: Capability,
    pub can_join_with_previous: Capability,
    pub can_delete: Capability,
    pub can_show_plain_text: Capability,
}

impl NodeCapabilities {
    /// Every action hidden except `can_show_plain_text`. Used for nodes that
    /// are never directly acted on, such as the synthetic document root.
    pub fn hidden() -> Self {
        Self {
            can_select: Capability::Hidden,
            can_edit_content: Capability::Hidden,
            can_add_inside: Capability::Hidden,
            can_add_after: Capability::Hidden,
            can_rename: Capability::Hidden,
            can_move_up: Capability::Hidden,
            can_move_down: Capability::Hidden,
            can_move_inside_previous: Capability::Hidden,
            can_move_out_one_level: Capability::Hidden,
            can_join_with_previous: Capability::Hidden,
            can_delete: Capability::Hidden,
            can_show_plain_text: Capability::Allowed,
        }
    }
}

/// Whether an action is available on a [`super::StructureNode`] (RFC-053 §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    /// Action is available.
    Allowed,
    /// Action is available in principle but currently blocked; the UI shows
    /// it disabled with a plain-language explanation derived from `reason`.
    Disabled { reason: CapabilityReason },
    /// Action is not applicable to this node/format; the UI does not render it.
    Hidden,
}

impl Capability {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden)
    }
}

/// Why an action is disabled on a particular node.
///
/// Typed and core-owned: `omriss-core` never depends on `omriss-ui`
/// (RFC-001), so this type must not name an i18n catalog key. `omriss-ui`
/// maps a `CapabilityReason` to localized text at render time (RFC-043).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityReason {
    /// This is the root / top-level node.
    RootNode,
    /// No sibling in the required direction exists.
    NoSibling,
    /// Already at the outermost level; cannot move out.
    NoParent,
    /// Format is read-only in this release.
    ReadOnlyFormat,
    /// Format is still experimental (spike / feasibility).
    ExperimentalFormat,
    /// Operation cannot be performed without risking lossy preservation.
    UnsafePreservation,
    /// This action is not defined for the active format.
    UnsupportedForFormat,
    /// The file changed on disk. A session-level overlay applied after
    /// adapter capabilities are built; never produced by a format adapter
    /// itself (RFC-053 §6).
    ExternalChangeConflict,
}
