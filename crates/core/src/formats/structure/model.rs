//! Document structure shape (RFC-053 §6).

use crate::{ByteRange, DocumentFormat, DocumentRevision, NodeId};

use super::NodeCapabilities;

/// A format adapter's derived structure for one document (RFC-053 §6).
///
/// Disposable and rebuildable from source text (RFC-053 §3.2); nothing
/// treats a `DocumentStructure` as a second source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentStructure {
    pub format: DocumentFormat,
    pub root_id: NodeId,
    pub nodes: Vec<StructureNode>,
    pub revision: DocumentRevision,
}

/// One node in a [`DocumentStructure`] (RFC-053 §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub title: String,
    pub kind: StructureNodeKind,
    pub depth: usize,
    pub source_range: Option<ByteRange>,
    pub editable_range: Option<ByteRange>,
    pub children: Vec<NodeId>,
    pub capabilities: NodeCapabilities,
}

/// What a [`StructureNode`] represents, independent of its source format
/// (RFC-053 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureNodeKind {
    /// The synthetic document root; never shown as a row.
    DocumentRoot,
    /// A Markdown heading section.
    MarkdownSection,
    /// A structured group (JSON object, TOML table, ...).
    Group,
    /// A structured list (JSON array, TOML array of tables, ...).
    List,
    /// A leaf value (JSON/TOML scalar).
    Value,
    /// Unparsed source shown as-is, e.g. a `PlainText` document.
    RawRegion,
    /// A node the active format cannot represent.
    Unsupported,
}
