//! Markdown capability computation, ported from `omriss-ui`'s shipped
//! `compute_capabilities` (RFC-049) so it can be tested at the core level
//! ahead of `MarkdownAdapter` (RFC-053 S4).
//!
//! This duplicates, rather than moves,
//! `crates/ui/src/session/document_map_bridge.rs::compute_capabilities` (and
//! the sibling lookup it reads from
//! `crates/ui/src/editor/navigation.rs::sibling_info`): `omriss-ui` keeps its
//! own copy until RFC-053 S3 reconciles it onto these types. The duplication
//! is temporary and intentional (RFC-053 task-breakdown-pr-plan, S2).

use crate::index::outline::HeadingLevel;
use crate::{NodeId, Outline};

use super::{Capability, CapabilityReason, NodeCapabilities};

/// Computes per-action capabilities for a Markdown section node.
///
/// `id` must be a non-root node that exists in `outline`; the document root
/// has no per-action capabilities of its own — callers use
/// [`NodeCapabilities::hidden`] for it, exactly as `omriss-ui` does today.
pub fn markdown_node_capabilities(outline: &Outline, id: NodeId) -> NodeCapabilities {
    let node = outline
        .node(id)
        .expect("markdown_node_capabilities: id must exist in outline");
    let (prev_sibling, next_sibling) = sibling_neighbors(outline, id);

    // can_move_up: has a previous sibling.
    let can_move_up = if prev_sibling.is_some() {
        Capability::Allowed
    } else {
        Capability::Disabled {
            reason: CapabilityReason::NoSibling,
        }
    };

    // can_move_down: has a next sibling.
    let can_move_down = if next_sibling.is_some() {
        Capability::Allowed
    } else {
        Capability::Disabled {
            reason: CapabilityReason::NoSibling,
        }
    };

    // can_move_inside_previous (demote): needs a previous sibling to move
    // into, and must not already be at the maximum heading depth.
    let can_move_inside_previous = if prev_sibling.is_some() {
        let at_max_depth = node.level.map(|l| l == HeadingLevel::H6).unwrap_or(false);
        if at_max_depth {
            Capability::Disabled {
                reason: CapabilityReason::NoSibling,
            }
        } else {
            Capability::Allowed
        }
    } else {
        Capability::Disabled {
            reason: CapabilityReason::NoSibling,
        }
    };

    // can_move_out_one_level (promote): needs a parent that is not the root,
    // and must not already be at the minimum heading depth.
    let can_move_out_one_level = {
        let is_root_child = node
            .parent_id
            .map(|pid| pid == outline.root_id())
            .unwrap_or(true);
        if is_root_child {
            Capability::Disabled {
                reason: CapabilityReason::NoParent,
            }
        } else {
            let at_min_depth = node.level.map(|l| l == HeadingLevel::H1).unwrap_or(false);
            if at_min_depth {
                Capability::Disabled {
                    reason: CapabilityReason::NoParent,
                }
            } else {
                Capability::Allowed
            }
        }
    };

    // can_join_with_previous: needs a previous sibling without children.
    // Removing a heading after a sibling subtree can make the joined body
    // parse under the previous sibling's last child instead of the sibling.
    let can_join_with_previous = if let Some(prev_id) = prev_sibling {
        let prev = outline.node(prev_id).expect("sibling id is valid");
        if prev.children.is_empty() {
            Capability::Allowed
        } else {
            Capability::Disabled {
                reason: CapabilityReason::UnsafePreservation,
            }
        }
    } else {
        Capability::Disabled {
            reason: CapabilityReason::NoSibling,
        }
    };

    NodeCapabilities {
        can_select: Capability::Allowed,
        can_edit_content: Capability::Allowed,
        can_add_inside: Capability::Allowed,
        can_add_after: Capability::Allowed,
        can_rename: Capability::Allowed,
        can_move_up,
        can_move_down,
        can_move_inside_previous,
        can_move_out_one_level,
        can_join_with_previous,
        can_delete: Capability::Allowed,
        can_show_plain_text: Capability::Allowed,
    }
}

/// Returns `(previous sibling, next sibling)` of `id` within its parent's
/// children, or `(None, None)` if `id` has no entry in `outline`.
///
/// A scoped port of `omriss_ui::editor::navigation::sibling_info`: that
/// function also reports `parent` and `first_child` for zoom/back-forward
/// navigation, neither of which capability computation reads (the shipped
/// `compute_capabilities` re-derives its own parent check directly from
/// `node.parent_id`), so only the two fields capability computation actually
/// uses are ported here.
///
/// `pub(crate)` because `MarkdownAdapter::structure_command` (RFC-053 S4b)
/// also needs sibling lookup for `MoveDirection::Up`/`Down`, mirroring the
/// shipped `move_focused_up`/`move_focused_down` in
/// `omriss_ui::session::structural`.
pub(crate) fn sibling_neighbors(outline: &Outline, id: NodeId) -> (Option<NodeId>, Option<NodeId>) {
    let Some(node) = outline.node(id) else {
        return (None, None);
    };
    let siblings = node
        .parent_id
        .and_then(|pid| outline.node(pid))
        .map(|p| p.children.as_slice())
        .unwrap_or(&[]);

    let pos = siblings.iter().position(|&c| c == id);
    let prev = pos.and_then(|i| i.checked_sub(1)).map(|i| siblings[i]);
    let next = pos.and_then(|i| {
        let j = i + 1;
        if j < siblings.len() {
            Some(siblings[j])
        } else {
            None
        }
    });
    (prev, next)
}
