//! Builds a `DocumentMapNode` tree from a `DocumentFormatAdapter`'s
//! `DocumentStructure`, for every format except Markdown (RFC-054 J3;
//! `document_map_bridge` handles Markdown, unchanged).
//!
//! When the active format's own adapter fails to build structure — today,
//! only JSON has a real one; TOML/YAML are still stubs that always fail —
//! this falls back to `PlainTextAdapter`, which never fails and produces no
//! synthetic structure (RFC-052 §5.2). That fallback is silent by design:
//! the file still opens, `EditorSession::show_raw()` (already wired,
//! format-agnostic) remains available regardless of what the Document Map
//! shows, and RFC-052 §5.2 forbids inventing a hierarchy for a format this
//! slice cannot parse — an empty Document Map is the honest result, not a
//! bug.

use omriss_core::{
    DocumentFormat, DocumentFormatAdapter, DocumentRevision, DocumentStructure, JsonAdapter,
    NodeId, PlainTextAdapter, TomlAdapter, YamlExperimentalAdapter,
};

use crate::interface::document_map::DocumentMapNode;

/// Builds the `DocumentMapNode` tree for `format` from `source`/`revision`
/// (read live, per call, never cached — RFC-053 §7.0/§0.4: the session
/// must pass `document.revision()` at the moment of building).
pub(super) fn document_map_node(
    format: DocumentFormat,
    source: &str,
    revision: DocumentRevision,
    selected: Option<NodeId>,
) -> DocumentMapNode {
    let structure = build_structure_or_fallback(format, source, revision);
    let index = Index::build(&structure);
    // The root id always resolves: it came from the same `structure` the
    // index was just built over.
    index
        .node(structure.root_id, selected)
        .expect("root_id from the same DocumentStructure is always present")
}

/// Tries `format`'s own adapter; falls back to `PlainTextAdapter` on any
/// failure. `PlainTextAdapter::build_structure` never fails, so this always
/// returns a usable structure.
///
/// `pub(super)`: RFC-054 J7 reuses this from the sibling `focus_bridge`
/// module, which needs the identical structure the Document Map is built
/// from -- so any `NodeId` a user could have clicked resolves the same way
/// in both places.
pub(super) fn build_structure_or_fallback(
    format: DocumentFormat,
    source: &str,
    revision: DocumentRevision,
) -> DocumentStructure {
    let primary = match format {
        DocumentFormat::Json => JsonAdapter.build_structure(source, revision),
        DocumentFormat::Toml => TomlAdapter.build_structure(source, revision),
        DocumentFormat::YamlExperimental => {
            YamlExperimentalAdapter.build_structure(source, revision)
        }
        DocumentFormat::PlainText | DocumentFormat::Unsupported | DocumentFormat::Markdown => {
            Err(fallback_only())
        }
    };
    primary.unwrap_or_else(|_| {
        PlainTextAdapter
            .build_structure(source, revision)
            .expect("PlainTextAdapter::build_structure never fails")
    })
}

fn fallback_only() -> omriss_core::StructureError {
    omriss_core::StructureErrorKind::UnsupportedFeature.into()
}

/// A `NodeId` -> `StructureNode` index over one `DocumentStructure`, built
/// once per call so recursive lookup is O(1) instead of a linear scan per
/// node (`DocumentStructure.nodes` is a flat, unordered `Vec`).
struct Index<'a> {
    by_id: std::collections::HashMap<NodeId, &'a omriss_core::StructureNode>,
}

impl<'a> Index<'a> {
    fn build(structure: &'a DocumentStructure) -> Self {
        Self {
            by_id: structure.nodes.iter().map(|n| (n.id, n)).collect(),
        }
    }

    /// Returns `None` if `id` is absent from the index — a malformed
    /// `StructureNode.children` entry naming an id not present in
    /// `DocumentStructure.nodes` (RFC-054 J3-IMPL-001: unreachable for
    /// `JsonAdapter`/`PlainTextAdapter` today, both well-tested, but the
    /// single consumption point every future adapter's output flows
    /// through). A missing child is skipped rather than panicking, so a
    /// bug in a future adapter degrades the Document Map to a partial
    /// tree instead of crashing the app.
    fn node(&self, id: NodeId, selected: Option<NodeId>) -> Option<DocumentMapNode> {
        let node = self.by_id.get(&id)?;
        let children = node
            .children
            .iter()
            .filter_map(|&cid| self.node(cid, selected))
            .collect();

        Some(DocumentMapNode {
            id: id.0,
            title: node.title.clone(),
            kind: node.kind,
            children,
            is_selected: selected == Some(id),
            capabilities: node.capabilities.clone(),
        })
    }
}
