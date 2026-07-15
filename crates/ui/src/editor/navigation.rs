//! Sibling and depth navigation helpers (RFC-020).
//!
//! These are pure functions over the `Outline`; they carry no state and are
//! called from `EditorSession` convenience wrappers.

use omriss_core::{NodeId, Outline};

/// Availability of the four spatial navigation actions from a given node.
#[derive(Debug, Clone, Default)]
pub struct SiblingInfo {
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
}

/// Computes navigation targets from `id` in `outline`.
pub fn sibling_info(outline: &Outline, id: NodeId) -> SiblingInfo {
    let Some(node) = outline.node(id) else {
        return SiblingInfo::default();
    };

    // Parent: the node's parent_id, but only if it's not the synthetic root
    // (zooming to the root goes to overview mode, not a section).
    let parent =
        node.parent_id
            .filter(|&pid| pid != outline.root_id())
            .or(if node.parent_id.is_some() {
                // parent_id == root → "parent" action goes to overview, signal that
                Some(outline.root_id())
            } else {
                None
            });

    // First child.
    let first_child = node.children.first().copied();

    // Siblings: find `id` in parent's children list.
    let siblings = node
        .parent_id
        .and_then(|pid| outline.node(pid))
        .map(|p| p.children.as_slice())
        .unwrap_or(&[]);

    let pos = siblings.iter().position(|&c| c == id);
    let prev_sibling = pos.and_then(|i| i.checked_sub(1)).map(|i| siblings[i]);
    let next_sibling = pos.and_then(|i| {
        let j = i + 1;
        if j < siblings.len() {
            Some(siblings[j])
        } else {
            None
        }
    });

    SiblingInfo {
        parent,
        first_child,
        prev_sibling,
        next_sibling,
    }
}
