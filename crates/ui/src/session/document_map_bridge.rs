//! Builds a `DocumentMapNode` tree from the current session outline (RFC-049).
//!
//! This bridges `omriss_core::Outline` (Markdown-only, RFC-006/007) to the
//! format-neutral `DocumentMapNode` used by the left-panel Document Map.
//! Capability production is `omriss_core::formats::structure::markdown_node_capabilities`
//! (RFC-053 S2); this module only projects the outline into `DocumentMapNode`s
//! and tracks selection, which is session state rather than document
//! structure (RFC-053 §14). When JSON/TOML adapters arrive (RFC-053/054/055)
//! they will provide their own bridge implementations; the Dioxus component
//! is unchanged.

use omriss_core::formats::structure::markdown_node_capabilities;
use omriss_core::{NodeCapabilities, NodeId, Outline};

use crate::interface::document_map::DocumentMapNode;

impl super::EditorSession {
    /// Build a `DocumentMapNode` tree for the Document Map left panel.
    ///
    /// Returns the root node. The root itself is not shown as a row; the
    /// caller renders `root.children`. `selected_id` is the currently focused
    /// `NodeId`, if any.
    pub fn document_map_nodes(&self) -> DocumentMapNode {
        let outline = self.document.outline();
        let selected = self.view.focused();
        build_map_node(outline, outline.root_id(), selected)
    }
}

fn build_map_node(outline: &Outline, id: NodeId, selected: Option<NodeId>) -> DocumentMapNode {
    let node = outline
        .node(id)
        .expect("NodeId from same outline is always valid");

    let children = node
        .children
        .iter()
        .map(|&cid| build_map_node(outline, cid, selected))
        .collect();

    let capabilities = if node.is_root() {
        NodeCapabilities::hidden()
    } else {
        markdown_node_capabilities(outline, id)
    };

    DocumentMapNode {
        id: id.0,
        title: node.title.clone(),
        children,
        is_selected: selected == Some(id),
        capabilities,
    }
}
