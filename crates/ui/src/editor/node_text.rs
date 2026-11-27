//! What each node of a document says, read from its `DocumentStructure`
//! (RFC-085 §3.3 items 5–6). Search and statistics both read this, so they cannot
//! disagree about a node's text.
//!
//! A node's *own* text, as its format's adapter says it (RFC-085 §8.2): a Markdown
//! section's body, a JSON value (decoded). A JSON group or list has none of its
//! own: its members are nodes. Titles (headings,
//! keys) are not here; RFC-077 adds them as one more kind of result on the same
//! walk.

use std::collections::HashMap;

use omriss_core::{
    ByteRange, DocumentFormat, DocumentFormatAdapter, DocumentStructure, JsonAdapter,
    MarkdownAdapter, NodeId, PlainTextAdapter, TomlAdapter, ValueKind, YamlExperimentalAdapter,
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
    /// What a JSON value is; `None` for any other node.
    pub value_kind: Option<ValueKind>,
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

/// The own text of every node in `ids`, in that order; nodes with none are left
/// out. What a node's own text *is* is its format's to say
/// (`DocumentFormatAdapter::own_text`, RFC-085 §8.2); this only asks, node by
/// node, so reading every text is linear in the document.
pub fn node_texts(
    source: &str,
    structure: &DocumentStructure,
    index: &HashMap<NodeId, usize>,
    ids: &[NodeId],
) -> Vec<NodeText> {
    ids.iter()
        .filter_map(|&id| {
            let node = &structure.nodes[*index.get(&id)?];
            let own = match structure.format {
                DocumentFormat::Markdown => MarkdownAdapter.own_text(source, node),
                DocumentFormat::Json => JsonAdapter.own_text(source, node),
                DocumentFormat::PlainText | DocumentFormat::Unsupported => {
                    PlainTextAdapter.own_text(source, node)
                }
                DocumentFormat::Toml => TomlAdapter.own_text(source, node),
                DocumentFormat::YamlExperimental => YamlExperimentalAdapter.own_text(source, node),
            }?;
            Some(NodeText {
                id,
                text: own.text,
                range: own.range,
                exact: own.exact,
                value_kind: own.value_kind,
            })
        })
        .collect()
}
