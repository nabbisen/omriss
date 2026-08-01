//! `Outline` -> `DocumentStructure` projection helpers for `build_structure`.

use crate::formats::error::StructureErrorKind;
use crate::formats::structure::{
    DocumentStructure, NodeCapabilities, StructureNode, StructureNodeKind,
    markdown_node_capabilities,
};
use crate::{NodeId, Outline, SectionNode};

pub(super) fn find_node(
    structure: &DocumentStructure,
    id: NodeId,
) -> Result<&StructureNode, StructureErrorKind> {
    structure
        .nodes
        .iter()
        .find(|n| n.id == id)
        .ok_or(StructureErrorKind::UnsafeRange)
}

pub(super) fn structure_node(outline: &Outline, section: &SectionNode) -> StructureNode {
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
