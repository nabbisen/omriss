//! The Document Map's tree (RFC-049), projected from the session's
//! `DocumentStructure` (RFC-085 §3.1) by `super::structure_bridge`. Selection is
//! session state rather than document structure (RFC-053 §14), so it is added
//! here, from the focused node.

use crate::interface::document_map::DocumentMapNode;

impl super::EditorSession {
    /// Build a `DocumentMapNode` tree for the Document Map left panel.
    ///
    /// Returns the root node. The root itself is not shown as a row; the
    /// caller renders `root.children`. `selected_id` is the currently focused
    /// `NodeId`, if any.
    ///
    /// Every format is projected from its `DocumentStructure` (RFC-085 §3.1):
    /// for Markdown, `MarkdownAdapter`'s, whose capabilities are
    /// `markdown_node_capabilities` (RFC-053 S2).
    pub fn document_map_nodes(&self) -> DocumentMapNode {
        let selected = self.view.focused();
        let (structure, _) = self.pane_view();
        super::structure_bridge::document_map_node(&structure, selected)
    }
}
