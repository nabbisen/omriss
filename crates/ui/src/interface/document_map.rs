//! Document Map view-model types (RFC-049, RFC-053).
//!
//! This module defines the format-neutral tree that the `omriss` app renders in
//! the left panel. It is built by `EditorSession::document_map_nodes()` and
//! contains only plain Rust — no Dioxus, no WebView dependency.
//!
//! The capability vocabulary itself — `Capability`, `NodeCapabilities`,
//! `CapabilityReason` — and its production for Markdown now live in
//! `omriss-core` (RFC-053 §6/§14, reconciled here in S3). This module keeps
//! only what must not move: the `DocumentMapNode` view projection (it nests
//! children by value and carries `is_selected`, which is session state, not
//! document structure) and the catalog-key mapping, since `omriss-core` must
//! never depend on `omriss-ui` (RFC-001) or name a catalog key.

use omriss_core::{CapabilityReason, NodeCapabilities, NodeId};

// ── Capability catalog-key mapping ────────────────────────────────────────────

/// Maps a core-owned [`CapabilityReason`] to its i18n catalog key (RFC-043).
///
/// This mapping must live in `omriss-ui`: `omriss-core` never depends on
/// `omriss-ui` (RFC-001) and must never name a catalog key (RFC-053 §6).
pub trait CapabilityReasonCatalogKey {
    /// Returns the i18n catalog key for this reason.
    fn catalog_key(self) -> &'static str;
}

impl CapabilityReasonCatalogKey for CapabilityReason {
    fn catalog_key(self) -> &'static str {
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
    pub capabilities: NodeCapabilities,
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
