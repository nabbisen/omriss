//! `OutlineNode` bridge — converts a `omriss` `Outline` into a
//! plain-Rust tree suitable for handoff to a tree widget such as
//! `dioxus-swdir-tree`'s `ItemTreeView`.

impl super::EditorSession {
    /// Build an `OutlineNode` tree describing the full document outline.
    ///
    /// Each node carries the section's title (empty string for the synthetic
    /// root) and the raw `NodeId` as a `u64`, so the caller can construct
    /// `dioxus_swdir_tree_core::ItemNode` without taking a dependency on
    /// `omriss` types directly.
    pub fn outline_nodes(&self) -> OutlineNode {
        let outline = self.document.outline();
        build_outline_node(outline, outline.root_id())
    }
}

/// A single node in the document outline, ready for handoff to a tree widget.
#[derive(Debug, Clone)]
pub struct OutlineNode {
    /// Raw `NodeId` value — cast to the widget's `NodeId(u64)` at the call site.
    pub id: u64,
    /// Section heading title; empty string for the synthetic root.
    pub title: String,
    /// Ordered child nodes.
    pub children: Vec<OutlineNode>,
}

fn build_outline_node(outline: &omriss_core::Outline, id: omriss_core::NodeId) -> OutlineNode {
    let node = outline
        .node(id)
        .expect("NodeId from same outline always valid");
    let children = node
        .children
        .iter()
        .map(|&child_id| build_outline_node(outline, child_id))
        .collect();
    OutlineNode {
        id: id.0,
        title: node.title.clone(),
        children,
    }
}
