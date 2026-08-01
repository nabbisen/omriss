//! Markdown format adapter (RFC-053 S4a).
//!
//! `MarkdownAdapter::build_structure` projects the shipped `Outline`
//! (RFC-006/007) into a [`DocumentStructure`] (RFC-053 §6), reusing every
//! `NodeId` the outline already assigned — never re-deriving, hashing, or
//! "improving" them (RFC-053 §13.2). RFC-023/024/025 focus-restoration
//! behavior depends on the current scheme.
//!
//! This is S4a of the RFC-053 S4 split: it mutates nothing. Structural
//! mutation (`StructureCommand` wrapping the shipped RFC-023/024/025
//! operations, and the full `DocumentFormatAdapter` trait implementation) is
//! S4b.

use crate::formats::error::{StructureError, StructureErrorKind};
use crate::formats::structure::{
    DocumentStructure, NodeCapabilities, StructureNode, StructureNodeKind,
    markdown_node_capabilities,
};
use crate::{Document, DocumentFormat, NodeId, Outline, SectionNode};

/// The Markdown format adapter (RFC-053 §17.1).
///
/// Owns no state — every method is a pure function of its arguments, per
/// RFC-053 §3.1.
#[derive(Debug, Default, Clone, Copy)]
pub struct MarkdownAdapter;

impl MarkdownAdapter {
    /// Projects `source` into a [`DocumentStructure`], preserving every
    /// `NodeId` the shipped outline builder assigns (RFC-053 §13.2).
    ///
    /// Markdown's own parser does not reject malformed input — headings are
    /// detected permissively by `pulldown-cmark` — so the only realistic
    /// failure here is an internal outline invariant violation, never
    /// "invalid" user Markdown.
    pub fn build_structure(&self, source: &str) -> Result<DocumentStructure, StructureError> {
        let document = Document::parse(source.to_string()).map_err(|_| StructureError {
            kind: StructureErrorKind::InternalInvariantFailed,
        })?;
        let outline = document.outline();

        let nodes = outline
            .iter()
            .map(|section| structure_node(outline, section))
            .collect();

        Ok(DocumentStructure {
            format: DocumentFormat::Markdown,
            root_id: outline.root_id(),
            nodes,
            revision: document.revision(),
        })
    }
}

fn structure_node(outline: &Outline, section: &SectionNode) -> StructureNode {
    let capabilities = if section.is_root() {
        NodeCapabilities::hidden()
    } else {
        markdown_node_capabilities(outline, section.id)
    };

    StructureNode {
        id: section.id,
        parent_id: section.parent_id,
        title: section.title.clone(),
        kind: if section.is_root() {
            StructureNodeKind::DocumentRoot
        } else {
            StructureNodeKind::MarkdownSection
        },
        depth: depth_of(outline, section.id),
        source_range: Some(section.full_range),
        editable_range: Some(section.body_range),
        children: section.children.clone(),
        capabilities,
    }
}

/// Ancestor count from the root (root itself is depth 0). Distinct from
/// heading level: skipped heading levels (e.g. an H1 followed directly by an
/// H3 child, per RFC-007) make tree depth and heading level diverge.
fn depth_of(outline: &Outline, id: NodeId) -> usize {
    let mut depth = 0;
    let mut current = id;
    while let Some(node) = outline.node(current) {
        let Some(parent_id) = node.parent_id else {
            break;
        };
        depth += 1;
        current = parent_id;
    }
    depth
}
