//! What each node of a document says, read from its `DocumentStructure`
//! (RFC-085 §3.3 items 5–6). Search and statistics both read this, so they cannot
//! disagree about a node's text.
//!
//! A node's *own* text: a Markdown section's body, a JSON value (decoded). A JSON
//! group or list has none of its own: its members are nodes. Titles (headings,
//! keys) are not here; RFC-077 adds them as one more kind of result on the same
//! walk.

use std::collections::HashMap;

use omriss_core::{
    ByteRange, DocumentFormat, DocumentFormatAdapter, DocumentStructure, FocusedContent,
    JsonAdapter, NodeId, StructureNodeKind,
};

/// One node's own text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeText {
    pub id: NodeId,
    pub text: String,
    /// Where the text sits in the source. When `exact`, `text` is the source at
    /// this range byte for byte, so an offset in the text is an offset in the
    /// source. When not (a JSON string is shown decoded, its escapes resolved),
    /// the range is the whole value.
    pub range: ByteRange,
    pub exact: bool,
}

/// The ids under `structure`'s root in document order (parents before their
/// children, siblings in order), root included.
pub fn preorder(structure: &DocumentStructure, index: &HashMap<NodeId, usize>) -> Vec<NodeId> {
    let mut out = Vec::with_capacity(structure.nodes.len());
    let mut stack = vec![structure.root_id];
    while let Some(id) = stack.pop() {
        out.push(id);
        if let Some(&i) = index.get(&id) {
            stack.extend(structure.nodes[i].children.iter().rev().copied());
        }
    }
    out
}

/// `id` and everything under it, in document order. Empty when `id` is not in
/// the structure.
pub fn subtree(
    structure: &DocumentStructure,
    index: &HashMap<NodeId, usize>,
    id: NodeId,
) -> Vec<NodeId> {
    if !index.contains_key(&id) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut stack = vec![id];
    while let Some(id) = stack.pop() {
        out.push(id);
        if let Some(&i) = index.get(&id) {
            stack.extend(structure.nodes[i].children.iter().rev().copied());
        }
    }
    out
}

/// The own text of every node in `ids`, in that order; nodes with none are left out.
pub fn node_texts(
    source: &str,
    structure: &DocumentStructure,
    index: &HashMap<NodeId, usize>,
    ids: &[NodeId],
) -> Vec<NodeText> {
    ids.iter()
        .filter_map(|&id| {
            let node = &structure.nodes[*index.get(&id)?];
            match (structure.format, node.kind) {
                (DocumentFormat::Json, StructureNodeKind::Value) => {
                    let range = node.editable_range.or(node.source_range)?;
                    let text = match JsonAdapter.focused_content(source, structure, id).ok()? {
                        FocusedContent::StructuredValue { display_text, .. } => display_text,
                        _ => return None,
                    };
                    Some(NodeText {
                        id,
                        text,
                        range,
                        exact: false,
                    })
                }
                (DocumentFormat::Json, _) => None,
                (DocumentFormat::Markdown, _) => {
                    let range = node.editable_range?;
                    Some(NodeText {
                        id,
                        text: source.get(range.as_range())?.to_string(),
                        range,
                        exact: true,
                    })
                }
                // The plain-text fallback, and any format with no reader of its own:
                // its whole text is the one region the structure has.
                _ => {
                    let range = node.source_range?;
                    Some(NodeText {
                        id,
                        text: source.get(range.as_range())?.to_string(),
                        range,
                        exact: true,
                    })
                }
            }
        })
        .collect()
}
