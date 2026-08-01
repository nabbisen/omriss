//! `JsonValue` -> `DocumentStructure` projection (RFC-054 §5, §6).

use crate::formats::structure::{
    Capability, CapabilityReason, DocumentStructure, NodeCapabilities, StructureNode,
    StructureNodeKind,
};
use crate::{DocumentFormat, DocumentRevision, NodeId};

use super::scanner::JsonValue;

/// Projects a parsed `JsonValue` tree into a flat `DocumentStructure`.
/// `root_id` points directly at the top-level value's own node — there is
/// no synthetic wrapper distinct from it, unlike Markdown's root (which
/// represents real, addressable preface content before any heading). JSON
/// has no such preface; the top-level value already *is* the whole
/// document, exactly as `PlainTextAdapter`'s single node is (RFC-053
/// S5-IMPL-002: `root_id` is the authoritative pointer to the root, `kind`
/// describes content and is never used to find it).
pub(super) fn build(root: &JsonValue, revision: DocumentRevision) -> DocumentStructure {
    let mut nodes = Vec::new();
    let root_id = push_node(&mut nodes, root, None, String::new(), 0, &[]);

    DocumentStructure {
        format: DocumentFormat::Json,
        root_id,
        nodes,
        revision,
    }
}

/// Recursively projects `value` and its descendants into `nodes`, returning
/// the `NodeId` assigned to `value` itself.
///
/// `path` is `value`'s ordinal path from the root (RFC-054 §6): the
/// sequence of zero-based child indices, object-member and array-item
/// positions alike. Reusing `NodeId::from_ordinal_path` — the same
/// derivation the Markdown outline builder and `PlainTextAdapter` use — is
/// what gives duplicate object keys distinct identity for free (RFC-054
/// §15.2): two members named `"a"` at positions 0 and 2 get paths `[0]`
/// and `[2]`, and therefore different ids, without the key text ever
/// entering the derivation.
fn push_node(
    nodes: &mut Vec<StructureNode>,
    value: &JsonValue,
    parent_id: Option<NodeId>,
    title: String,
    depth: usize,
    path: &[usize],
) -> NodeId {
    let id = NodeId::from_ordinal_path(path);
    let range = value.range();

    let (kind, children) = match value {
        JsonValue::Object { members, .. } => {
            let children = members
                .iter()
                .enumerate()
                .map(|(ordinal, member)| {
                    let child_path = extend(path, ordinal);
                    push_node(
                        nodes,
                        &member.value,
                        Some(id),
                        member.key.clone(),
                        depth + 1,
                        &child_path,
                    )
                })
                .collect();
            (StructureNodeKind::Group, children)
        }
        JsonValue::Array { items, .. } => {
            let children = items
                .iter()
                .enumerate()
                .map(|(ordinal, item)| {
                    let child_path = extend(path, ordinal);
                    push_node(
                        nodes,
                        item,
                        Some(id),
                        // RFC-054 §5: "array item scalar -> row named 'Item
                        // 1', 'Item 2', etc." -- 1-based in the label,
                        // 0-based in the identity path.
                        format!("Item {}", ordinal + 1),
                        depth + 1,
                        &child_path,
                    )
                })
                .collect();
            (StructureNodeKind::List, children)
        }
        JsonValue::String { .. }
        | JsonValue::Number { .. }
        | JsonValue::Bool { .. }
        | JsonValue::Null { .. } => (StructureNodeKind::Value, Vec::new()),
    };

    // The root has no per-action capabilities of its own, exactly as
    // Markdown's synthetic root and PlainText's single node do not --
    // callers use `NodeCapabilities::hidden()` rather than a computed set.
    let capabilities = if parent_id.is_none() {
        NodeCapabilities::hidden()
    } else {
        json_node_capabilities(kind)
    };

    nodes.push(StructureNode {
        id,
        parent_id,
        title,
        kind,
        depth,
        source_range: Some(range),
        editable_range: Some(range),
        children,
        capabilities,
    });

    id
}

fn extend(path: &[usize], ordinal: usize) -> Vec<usize> {
    let mut extended = Vec::with_capacity(path.len() + 1);
    extended.extend_from_slice(path);
    extended.push(ordinal);
    extended
}

/// Every action but selection and (for containers) plain-text viewing is
/// `Disabled { ReadOnlyFormat }` or `Hidden` in this slice — J2 is
/// read-only, so nothing here can yet be `Allowed`:
///
/// - `can_select` is `Allowed`: browsing the tree is exactly what this
///   slice delivers.
/// - `can_edit_content` is `Disabled { ReadOnlyFormat }` on every real
///   node: scalar editing (J5) and container raw editing (J6) both land as
///   later slices, not a permanent refusal.
/// - `can_show_plain_text` is `Disabled { ReadOnlyFormat }` on `Group`/`List`
///   nodes only, matching RFC-054 §4.3/§7.4's "Show this part as text" —
///   a container-only affordance (J6). `Value` nodes have no raw-text
///   concept of their own in the RFC's mockups (§4.4-§4.7 show typed
///   editors, never a raw-text option), so it is `Hidden` there, not
///   `Disabled`.
/// - Every add/rename/move/delete/join capability is `Hidden`, not
///   `Disabled`: those are RFC-054 §13 Phase 4 operations, explicitly out
///   of scope for this entire handoff (not merely deferred to a later
///   J-slice within it), so the UI should not render them as blocked
///   affordances that will later switch on -- there is no committed plan
///   for that switch inside RFC-054.
fn json_node_capabilities(kind: StructureNodeKind) -> NodeCapabilities {
    let can_show_plain_text = match kind {
        StructureNodeKind::Group | StructureNodeKind::List => Capability::Disabled {
            reason: CapabilityReason::ReadOnlyFormat,
        },
        _ => Capability::Hidden,
    };

    NodeCapabilities {
        can_select: Capability::Allowed,
        can_edit_content: Capability::Disabled {
            reason: CapabilityReason::ReadOnlyFormat,
        },
        can_add_inside: Capability::Hidden,
        can_add_after: Capability::Hidden,
        can_rename: Capability::Hidden,
        can_move_up: Capability::Hidden,
        can_move_down: Capability::Hidden,
        can_move_inside_previous: Capability::Hidden,
        can_move_out_one_level: Capability::Hidden,
        can_join_with_previous: Capability::Hidden,
        can_delete: Capability::Hidden,
        can_show_plain_text,
    }
}
