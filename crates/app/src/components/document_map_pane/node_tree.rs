//! Document Map node-tree helpers: `ItemNode` conversion, lookup, and
//! capability-title labels shared by the panel and its row menus.

use dioxus_swdir_tree::item_tree::node::ItemNode;
use dioxus_swdir_tree::item_tree::node::NodeId as SwNodeId;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{Capability, CapabilityReasonCatalogKey, DocumentMapNode};

pub(super) fn to_item_node(n: &DocumentMapNode) -> ItemNode<String> {
    let id = SwNodeId(n.id);
    let children: Vec<ItemNode<String>> = n.children.iter().map(to_item_node).collect();
    if children.is_empty() {
        ItemNode::leaf(id, n.title.clone())
    } else {
        ItemNode::branch(id, n.title.clone(), children)
    }
}

pub(super) fn disabled_title(lang: Locale, cap: &Capability) -> String {
    if let Capability::Disabled { reason } = cap {
        t(lang, reason.catalog_key()).to_string()
    } else {
        String::new()
    }
}

pub(super) fn capability_title(
    lang: Locale,
    cap: &Capability,
    enabled_key: &'static str,
) -> String {
    if let Capability::Disabled { reason } = cap {
        t(lang, reason.catalog_key()).to_string()
    } else {
        t(lang, enabled_key).to_string()
    }
}

pub(super) fn find_node(root: &DocumentMapNode, id: u64) -> Option<&DocumentMapNode> {
    if root.id == id {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_node(c, id))
}

/// Collect the `SwNodeId`s of all ancestors of `target` (root → parent,
/// not including `target` itself) into `out`. Returns `true` if found.
pub(super) fn collect_ancestors(
    node: &DocumentMapNode,
    target: SwNodeId,
    out: &mut Vec<SwNodeId>,
) -> bool {
    if SwNodeId(node.id) == target {
        return true;
    }
    for child in &node.children {
        if collect_ancestors(child, target, out) {
            out.push(SwNodeId(node.id));
            return true;
        }
    }
    false
}
