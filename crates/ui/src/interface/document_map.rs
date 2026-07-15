//! Document Map view-model types (RFC-049, RFC-053).
//!
//! This module defines the format-neutral tree that the `omriss` app renders in
//! the left panel. It is built by `EditorSession::document_map_nodes()` and
//! contains only plain Rust — no Dioxus, no WebView dependency.
//!
//! The capability fields follow RFC-053: each action slot carries a
//! `MapCapability` rather than a bare `bool`, so the Document Map can show
//! exactly which actions are disabled and why, without the Document Map
//! needing to know Markdown internals.

use omriss_core::NodeId;

// ── Draft state ───────────────────────────────────────────────────────────────

/// Tracks the validity of an in-progress focused-content edit (RFC-053).
///
/// This is editor-local state, not a second user-visible saved-document dirty
/// state. The user-facing file status remains `Saved` / `Unsaved changes`;
/// `DraftState` drives the block-on-invalid gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DraftState {
    /// No local edit in progress; matches the committed content.
    #[default]
    Clean,
    /// The draft differs from committed content and is format-valid.
    ValidUncommitted,
    /// The draft differs from committed content and fails format validation.
    /// Navigation, save, and preview are blocked until resolved.
    InvalidUncommitted,
}

impl DraftState {
    /// Returns `true` when the draft blocks navigation/save/preview.
    pub fn blocks_navigation(self) -> bool {
        self == Self::InvalidUncommitted
    }
}

// ── Capability model ──────────────────────────────────────────────────────────

/// Why an action is disabled on a particular node.
///
/// This is a core-owned type (RFC-001): it may appear in `omriss-core` crate types
/// that are returned to `omriss-ui`. `omriss-ui` maps it to i18n catalog keys
/// at render time; it never contains UI-layer types.
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
    /// The file changed on disk; session-level overlay applied by `omriss-ui`.
    ExternalChangeConflict,
}

impl CapabilityReason {
    /// Returns the i18n catalog key for this reason.
    pub fn catalog_key(self) -> &'static str {
        match self {
            Self::RootNode => "capability.disabled.root_node",
            Self::NoSibling => "capability.disabled.no_sibling",
            Self::NoParent => "capability.disabled.no_parent",
            Self::ReadOnlyFormat => "capability.disabled.read_only_format",
            Self::ExperimentalFormat => "capability.disabled.experimental_format",
            Self::UnsafePreservation => "capability.disabled.unsafe_preservation",
            Self::UnsupportedForFormat => "capability.disabled.unsupported_for_format",
            Self::ExternalChangeConflict => "capability.disabled.external_change_conflict",
        }
    }
}

/// Whether an action is available on a Document Map node (RFC-053).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapCapability {
    /// Action is available.
    Allowed,
    /// Action is available in principle but currently blocked; shown disabled
    /// with a plain-language explanation from `CapabilityReason::catalog_key`.
    Disabled(CapabilityReason),
    /// Action is not applicable to this node/format; do not render it.
    Hidden,
}

impl MapCapability {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden)
    }
}

// ── Per-node capabilities ─────────────────────────────────────────────────────

/// Granular action availability for one Document Map node.
///
/// Populated from `SiblingInfo` and node properties; returned as part of
/// `DocumentMapNode`. The Document Map renders row menus based on these
/// capabilities and never needs to re-derive them from format internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapNodeCapabilities {
    pub can_add_inside: MapCapability,
    pub can_add_after: MapCapability,
    pub can_rename: MapCapability,
    pub can_move_up: MapCapability,
    pub can_move_down: MapCapability,
    pub can_move_inside_previous: MapCapability,
    pub can_move_out_one_level: MapCapability,
    pub can_join_with_previous: MapCapability,
    pub can_delete: MapCapability,
    pub can_show_plain_text: MapCapability,
}

impl MapNodeCapabilities {
    /// All actions hidden — used for the synthetic root node.
    pub fn hidden() -> Self {
        Self {
            can_add_inside: MapCapability::Hidden,
            can_add_after: MapCapability::Hidden,
            can_rename: MapCapability::Hidden,
            can_move_up: MapCapability::Hidden,
            can_move_down: MapCapability::Hidden,
            can_move_inside_previous: MapCapability::Hidden,
            can_move_out_one_level: MapCapability::Hidden,
            can_join_with_previous: MapCapability::Hidden,
            can_delete: MapCapability::Hidden,
            can_show_plain_text: MapCapability::Allowed,
        }
    }
}

// ── DocumentMapNode ───────────────────────────────────────────────────────────

/// One row in the Document Map, ready for the left-panel tree widget.
///
/// Format-neutral: for Markdown this represents a section; for JSON/TOML it
/// will represent an object, array, or value. Only Markdown is implemented in
/// RFC-048/049.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentMapNode {
    /// Raw `NodeId.0` value — cast to the widget's node type at the call site.
    pub id: u64,
    /// The display title (heading text for Markdown; key for JSON/TOML).
    pub title: String,
    /// Ordered child nodes.
    pub children: Vec<DocumentMapNode>,
    /// Whether this node is currently selected in the editor.
    pub is_selected: bool,
    /// Available actions for the row menu.
    pub capabilities: MapNodeCapabilities,
}

impl DocumentMapNode {
    /// Total number of descendants (not counting self).
    pub fn descendant_count(&self) -> usize {
        self.children.iter().map(|c| 1 + c.descendant_count()).sum()
    }
}

// ── Helper: Markdown node id back-cast ───────────────────────────────────────

/// Converts a raw `u64` from `DocumentMapNode::id` back to a core `NodeId`.
pub fn node_id_from_raw(raw: u64) -> NodeId {
    NodeId(raw)
}
