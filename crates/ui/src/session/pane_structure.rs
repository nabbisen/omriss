//! What the panes read: the document's `DocumentStructure`, for every format
//! (RFC-085 §3.1).
//!
//! The overview's cards and keys, the breadcrumb, Esc and Up, search and
//! statistics all go through here, so there is one tree and no second one to
//! disagree with it. The structure is the active format's own, cached by
//! revision (RFC-067 C1); when that adapter cannot read the source it is the
//! plain-text fallback, a lone root.
//!
//! The Markdown outline is read in exactly one place in this file: a section's
//! heading level, for the card label "H2" — a Markdown-only label (RFC-085 §3.1).

use omriss_core::{
    DocumentFormat, DocumentFormatAdapter, DocumentStructure, FocusedContent, JsonAdapter, NodeId,
    StructureNode, StructureNodeKind,
};

use crate::editor::node_text::{node_texts, preorder, subtree};
use crate::editor::search::{SearchMatch, search_texts};
use crate::editor::stats::{DocumentStats, compute_stats};
use crate::interface::node_item::NodeItem;

impl super::EditorSession {
    /// `node` as a pane lists it.
    fn node_item(&self, structure: &DocumentStructure, node: &StructureNode) -> NodeItem {
        let level = if self.format == DocumentFormat::Markdown {
            // Markdown-only label: a heading's level is not in the structure
            // (it is not the tree depth when a document skips a level).
            self.document.outline().node(node.id).and_then(|n| n.level)
        } else {
            None
        };
        let value_kind =
            if self.format == DocumentFormat::Json && node.kind == StructureNodeKind::Value {
                match JsonAdapter.focused_content(self.document.source(), structure, node.id) {
                    Ok(FocusedContent::StructuredValue { value_kind, .. }) => Some(value_kind),
                    _ => None,
                }
            } else {
                None
            };
        NodeItem {
            id: node.id,
            title: node.title.clone(),
            level,
            child_count: node.children.len(),
            kind: node.kind,
            is_root: node.id == structure.root_id,
            value_kind,
        }
    }

    /// The items for `ids`, in that order.
    fn items_of(&self, ids: &[NodeId]) -> Vec<NodeItem> {
        let (structure, index) = self.pane_view();
        ids.iter()
            .filter_map(|id| {
                let node = &structure.nodes[*index.get(id)?];
                Some(self.node_item(&structure, node))
            })
            .collect()
    }

    /// Top-level items: the root's direct children, or just the root for a
    /// document with nothing under it (the whole document is then one layer).
    pub fn outline_items(&self) -> Vec<NodeItem> {
        let (structure, index) = self.pane_view();
        let root = &structure.nodes[index[&structure.root_id]];
        if root.children.is_empty() {
            vec![self.node_item(&structure, root)]
        } else {
            self.items_of(&root.children)
        }
    }

    /// Items relevant to the current view:
    /// - In overview mode: the top-level items (same as [`outline_items`]).
    /// - In focus mode: the immediate children of the focused node.
    ///
    /// The cards of the overview and the targets of its keys (RFC-011, RFC-014).
    pub fn current_children(&self) -> Vec<NodeItem> {
        match self.view.focused() {
            Some(id) => {
                let (structure, index) = self.pane_view();
                match index.get(&id) {
                    Some(&i) => self.items_of(&structure.nodes[i].children),
                    None => Vec::new(),
                }
            }
            None => self.outline_items(),
        }
    }

    /// The path from the document's root to the focused node, both included;
    /// empty when nothing is focused or the node no longer resolves. The
    /// breadcrumb.
    pub fn focused_path(&self) -> Vec<NodeItem> {
        let Some(id) = self.view.focused() else {
            return Vec::new();
        };
        self.path_to(id)
    }

    /// The path from the root to `id`, both included; empty if `id` is unknown.
    pub(super) fn path_to(&self, id: NodeId) -> Vec<NodeItem> {
        let (structure, index) = self.pane_view();
        let mut ids = Vec::new();
        let mut current = Some(id);
        while let Some(id) = current {
            let Some(&i) = index.get(&id) else {
                return Vec::new();
            };
            ids.push(id);
            current = structure.nodes[i].parent_id;
        }
        ids.reverse();
        self.items_of(&ids)
    }

    /// Zooms out one level: moves focus to the parent, or returns to the
    /// whole-document view if the current focus is a direct child of the root.
    ///
    /// This is the structural "Esc" action (RFC-014). It pushes the current
    /// node onto the back history (via `ViewState::focus`/`show_outline`) so
    /// the user can return forward with Alt+Right.
    pub fn zoom_out(&mut self) {
        let Some(id) = self.view.focused() else {
            return;
        };
        let (structure, index) = self.pane_view();
        let parent_id = index.get(&id).and_then(|&i| structure.nodes[i].parent_id);
        match parent_id {
            Some(pid) if pid != structure.root_id => {
                self.view.focus(pid);
            }
            _ => {
                self.view.show_outline();
            }
        }
        self.prune_dead_history();
    }

    /// Searches the whole document for `query` (RFC-021).
    pub fn search_document(&self, query: &str) -> Vec<SearchMatch> {
        self.search_in(None, query)
    }

    /// Searches the focused node and everything under it for `query`; the whole
    /// document when nothing is focused.
    pub fn search_section(&self, query: &str) -> Vec<SearchMatch> {
        self.search_in(self.view.focused(), query)
    }

    fn search_in(&self, scope: Option<NodeId>, query: &str) -> Vec<SearchMatch> {
        let (structure, index) = self.pane_view();
        let ids = match scope {
            Some(id) if index.contains_key(&id) => subtree(&structure, &index, id),
            _ => preorder(&structure, &index),
        };
        let texts = node_texts(self.document.source(), &structure, &index, &ids);
        search_texts(&texts, query, |id| self.path_to(id))
    }

    /// Returns document statistics (RFC-046, RFC-085 §3.3 item 6): word counts
    /// and the count of the format's own unit.
    pub fn stats(&self) -> DocumentStats {
        let (structure, index) = self.pane_view();
        let all = preorder(&structure, &index);
        let texts = node_texts(self.document.source(), &structure, &index, &all);
        let focus_ids = self
            .view
            .focused()
            .map(|id| (id, subtree(&structure, &index, id)));
        compute_stats(
            self.document.source(),
            &structure,
            &texts,
            focus_ids.as_ref().map(|(id, ids)| (*id, ids.as_slice())),
        )
    }
}
