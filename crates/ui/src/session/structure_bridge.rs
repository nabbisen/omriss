//! The session's `DocumentStructure`, for every format (RFC-085 §3.1): the cache
//! every pane reads, and the `DocumentMapNode` tree built from it (RFC-054 J3).
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
//!
//! `EditorSession::structure_result`/`structure_or_fallback` (RFC-067 §3.1)
//! are the single place a non-Markdown structure is built: one cache entry,
//! keyed on `DocumentRevision`, shared by every reader — the Document Map,
//! `focus`, `prune_dead_history`, and JSON's structured-editing read/write
//! path in `focus_bridge.rs`/`structured_edit.rs`. RFC-054 §0.4's rule is
//! unweakened by this: a revision match is a *proof* the source hasn't
//! changed since the cached structure was built, not a guess, and a draft
//! (held in a UI signal, never passed to `Document`) never bumps the
//! revision — so this is exactly the layer RFC-054 §0.4 always needed
//! beneath it, not an exception to it.

use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use omriss_core::{
    DocumentFormat, DocumentFormatAdapter, DocumentRevision, DocumentStructure, JsonAdapter,
    MarkdownAdapter, NodeId, PlainTextAdapter, StructureError, TomlAdapter,
    YamlExperimentalAdapter,
};

use crate::editor::node_text::{NodeText, TitleKind, node_texts, preorder, searchable_title};
use crate::interface::document_map::DocumentMapNode;

/// One cache slot: the active format adapter's own `build_structure`
/// attempt (not merged with the `PlainTextAdapter` fallback — see
/// `EditorSession::structure_or_fallback`), plus the revision it was built
/// from. `Markdown`/`PlainText`/`Unsupported` never populate this; they
/// have no adapter attempt to cache (`document_map_nodes` doesn't call
/// through here for Markdown at all, and the other two have nothing but
/// the fallback).
#[derive(Debug, Clone)]
pub(super) struct StructureCache {
    revision: DocumentRevision,
    result: Result<Rc<DocumentStructure>, StructureError>,
    /// Node id -> position in `DocumentStructure.nodes`, built with the entry, so a
    /// pane looking up many nodes (cards, crumbs, search paths) does not scan.
    index: Rc<HashMap<NodeId, usize>>,
    /// Every node's own text, in document order, built the first time something
    /// reads it and kept as long as the revision stands (RFC-085 §8.3): search per
    /// keystroke, statistics and a card's kind label decode nothing twice.
    texts: OnceCell<Rc<PaneTexts>>,
}

/// Every node's own text, with a position index by node id, and what search walks:
/// one entry per node that has a searchable title or a text, in document order.
#[derive(Debug)]
pub(crate) struct PaneTexts {
    pub(crate) all: Vec<NodeText>,
    pub(crate) by_id: HashMap<NodeId, usize>,
    pub(crate) entries: Vec<PaneEntry>,
}

/// One node as search sees it: its searchable title (RFC-077) and, by position in
/// [`PaneTexts::all`], its own text.
#[derive(Debug)]
pub(crate) struct PaneEntry {
    pub(crate) id: NodeId,
    pub(crate) title: Option<(String, TitleKind)>,
    pub(crate) text: Option<usize>,
    /// Where the node starts in the source.
    pub(crate) start: usize,
}

impl super::EditorSession {
    /// The active format adapter's own structure for the current document —
    /// from cache if the revision matches, freshly built and cached
    /// otherwise (RFC-067 §3.1). `Err` means the adapter couldn't parse the
    /// source as its format (today, only `JsonAdapter` is real); callers
    /// needing "real structure, or nothing" (JSON's structured-editing
    /// reads/writes, in `focus_bridge.rs`/`structured_edit.rs`) use this
    /// directly. Callers needing "always something to show" (the Document
    /// Map) use `structure_or_fallback` instead. `pub(super)`: shared by
    /// the sibling `focus_bridge`/`structured_edit` modules.
    pub(super) fn structure_result(&self) -> Result<Rc<DocumentStructure>, StructureError> {
        let revision = self.document.revision();
        if let Some(cached) = self.structure_cache.borrow().as_ref()
            && cached.revision == revision
        {
            return cached.result.clone();
        }
        let source = self.document.source();
        let result = match self.format {
            DocumentFormat::Markdown => MarkdownAdapter.build_structure(source, revision),
            DocumentFormat::Json => JsonAdapter.build_structure(source, revision),
            DocumentFormat::Toml => TomlAdapter.build_structure(source, revision),
            DocumentFormat::YamlExperimental => {
                YamlExperimentalAdapter.build_structure(source, revision)
            }
            DocumentFormat::PlainText | DocumentFormat::Unsupported => Err(fallback_only()),
        }
        .map(Rc::new);
        let index = Rc::new(match &result {
            Ok(structure) => index_of(structure),
            Err(_) => HashMap::new(),
        });
        *self.structure_cache.borrow_mut() = Some(StructureCache {
            revision,
            result: result.clone(),
            index,
            texts: OnceCell::new(),
        });
        result
    }

    /// `structure_result`, falling back to `PlainTextAdapter` (which never
    /// fails) when the active adapter can't parse the source. Used by every
    /// reader that needs a structure to display rather than to edit
    /// against: the Document Map, `focus`, `prune_dead_history`.
    pub(super) fn structure_or_fallback(&self) -> Rc<DocumentStructure> {
        self.structure_result().unwrap_or_else(|_| {
            Rc::new(
                PlainTextAdapter
                    .build_structure(self.document.source(), self.document.revision())
                    .expect("PlainTextAdapter::build_structure never fails"),
            )
        })
    }
}

pub(super) fn index_of(structure: &DocumentStructure) -> HashMap<NodeId, usize> {
    structure
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id, i))
        .collect()
}

impl super::EditorSession {
    /// The structure every pane reads (RFC-085 §3.1) with its node index: the
    /// active format's own structure, cached by revision, or — when that adapter
    /// cannot read the source — the plain-text fallback, a lone root.
    pub(super) fn pane_view(&self) -> (Rc<DocumentStructure>, Rc<HashMap<NodeId, usize>>) {
        if self.structure_result().is_ok()
            && let Some(cache) = self.structure_cache.borrow().as_ref()
            && let Ok(structure) = &cache.result
        {
            return (structure.clone(), cache.index.clone());
        }
        let structure = self.structure_or_fallback();
        let index = Rc::new(index_of(&structure));
        (structure, index)
    }

    /// Every node's own text for the current revision: cached with the structure
    /// (a draft never bumps the revision, so typing rebuilds nothing; a commit,
    /// undo or redo does), or, when the adapter cannot read the source, built
    /// from the plain-text fallback each time (one node).
    pub(crate) fn pane_texts(&self) -> Rc<PaneTexts> {
        let build = |structure: &DocumentStructure, index: &HashMap<NodeId, usize>| {
            let ids = preorder(structure, index);
            let all = node_texts(self.document.source(), structure, index, &ids);
            let by_id: HashMap<NodeId, usize> =
                all.iter().enumerate().map(|(i, t)| (t.id, i)).collect();
            let entries = ids
                .iter()
                .filter_map(|&id| {
                    let node = &structure.nodes[*index.get(&id)?];
                    let title = searchable_title(structure, index, node)
                        .map(|(title, kind)| (title.to_string(), kind));
                    let text = by_id.get(&id).copied();
                    let start = node.source_range.map_or(0, |r| r.start);
                    (title.is_some() || text.is_some()).then_some(PaneEntry {
                        id,
                        title,
                        text,
                        start,
                    })
                })
                .collect();
            Rc::new(PaneTexts {
                all,
                by_id,
                entries,
            })
        };
        if self.structure_result().is_ok()
            && let Some(cache) = self.structure_cache.borrow().as_ref()
            && let Ok(structure) = &cache.result
        {
            return cache
                .texts
                .get_or_init(|| build(structure, &cache.index))
                .clone();
        }
        let (structure, index) = self.pane_view();
        build(&structure, &index)
    }

    /// Whether the active format's adapter could read the source. `false` is an
    /// unreadable `.json`, say: panes then have only the plain-text fallback.
    pub fn structure_readable(&self) -> bool {
        self.structure_result().is_ok()
    }
}

fn fallback_only() -> StructureError {
    omriss_core::StructureErrorKind::UnsupportedFeature.into()
}

/// Builds the `DocumentMapNode` tree from an already-built `structure`.
pub(super) fn document_map_node(
    structure: &DocumentStructure,
    selected: Option<NodeId>,
) -> DocumentMapNode {
    let index = Index::build(structure);
    // The root id always resolves: it came from the same `structure` the
    // index was just built over.
    index
        .node(structure.root_id, selected)
        .expect("root_id from the same DocumentStructure is always present")
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
