//! Builds a `DocumentMapNode` tree from the current session outline (RFC-049).
//!
//! This bridges `omriss::Outline` (Markdown-only, RFC-006/007) to the
//! format-neutral `DocumentMapNode` used by the left-panel Document Map.
//! When JSON/TOML adapters arrive (RFC-053/054/055) they will provide their
//! own bridge implementations; the Dioxus component is unchanged.

use omriss::{NodeId, Outline};

use crate::editor::navigation::sibling_info;
use crate::interface::document_map::{
    CapabilityReason, DocumentMapNode, MapCapability, MapNodeCapabilities,
};

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
        MapNodeCapabilities::hidden()
    } else {
        compute_capabilities(outline, id)
    };

    DocumentMapNode {
        id: id.0,
        title: node.title.clone(),
        children,
        is_selected: selected == Some(id),
        capabilities,
    }
}

/// Compute per-action capabilities for a Markdown section node.
fn compute_capabilities(outline: &Outline, id: NodeId) -> MapNodeCapabilities {
    let info = sibling_info(outline, id);
    let node = outline.node(id).expect("valid id");

    // can_move_up: has a previous sibling
    let can_move_up = if info.prev_sibling.is_some() {
        MapCapability::Allowed
    } else {
        MapCapability::Disabled(CapabilityReason::NoSibling)
    };

    // can_move_down: has a next sibling
    let can_move_down = if info.next_sibling.is_some() {
        MapCapability::Allowed
    } else {
        MapCapability::Disabled(CapabilityReason::NoSibling)
    };

    // can_move_inside_previous (demote): needs a previous sibling to move into
    let can_move_inside_previous = if info.prev_sibling.is_some() {
        // Also check heading level limit (H6 cannot be demoted further)
        let at_max_depth = node
            .level
            .map(|l| l == omriss::HeadingLevel::H6)
            .unwrap_or(false);
        if at_max_depth {
            MapCapability::Disabled(CapabilityReason::NoSibling)
        } else {
            MapCapability::Allowed
        }
    } else {
        MapCapability::Disabled(CapabilityReason::NoSibling)
    };

    // can_move_out_one_level (promote): needs a parent that is not the root
    let can_move_out_one_level = {
        let is_root_child = node
            .parent_id
            .map(|pid| pid == outline.root_id())
            .unwrap_or(true);
        if is_root_child {
            MapCapability::Disabled(CapabilityReason::NoParent)
        } else {
            // Also check heading level limit (H1 cannot be promoted further)
            let at_min_depth = node
                .level
                .map(|l| l == omriss::HeadingLevel::H1)
                .unwrap_or(false);
            if at_min_depth {
                MapCapability::Disabled(CapabilityReason::NoParent)
            } else {
                MapCapability::Allowed
            }
        }
    };

    // can_join_with_previous: needs a previous sibling without children.
    // Removing a heading after a sibling subtree can make the joined body
    // parse under the previous sibling's last child instead of the sibling.
    let can_join_with_previous = if let Some(prev_id) = info.prev_sibling {
        let prev = outline.node(prev_id).expect("sibling id is valid");
        if prev.children.is_empty() {
            MapCapability::Allowed
        } else {
            MapCapability::Disabled(CapabilityReason::UnsafePreservation)
        }
    } else {
        MapCapability::Disabled(CapabilityReason::NoSibling)
    };

    MapNodeCapabilities {
        can_add_inside: MapCapability::Allowed,
        can_add_after: MapCapability::Allowed,
        can_rename: MapCapability::Allowed,
        can_move_up,
        can_move_down,
        can_move_inside_previous,
        can_move_out_one_level,
        can_join_with_previous,
        can_delete: MapCapability::Allowed,
        can_show_plain_text: MapCapability::Allowed,
    }
}
