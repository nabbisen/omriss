//! The canonical document model (RFC-002) and its public API (RFC-005).
//!
//! The raw Markdown source text is the only canonical representation. The
//! outline is a derived index rebuilt after every committed mutation, and the
//! saved file is always the canonical text written back verbatim — never an
//! AST serialization.

use crate::doc::edit::{EditResult, ReplaceSectionBody};
use crate::doc::history::{EditHistory, EditRecord};
use crate::doc::revision::DocumentRevision;
use crate::error::{DocumentError, EditError};
use crate::index::builder::build_outline;
use crate::index::outline::{HeadingLevel, NodeId, Outline};
use crate::range::{ByteRange, RangeError};

/// Backing storage for the canonical text (RFC-002).
///
/// M0/M1 use a `String`; the enum keeps room for a rope-backed variant once
/// measurements justify it, without changing the public `Document` API.
#[derive(Debug, Clone)]
enum TextBuffer {
    String(String),
}

impl TextBuffer {
    fn from_string(text: String) -> Self {
        TextBuffer::String(text)
    }

    fn as_str(&self) -> &str {
        match self {
            TextBuffer::String(s) => s,
        }
    }

    fn slice(&self, range: ByteRange) -> Result<&str, RangeError> {
        let source = self.as_str();
        range.validate_in(source)?;
        Ok(&source[range.as_range()])
    }

    fn replace(&mut self, range: ByteRange, replacement: &str) -> Result<(), RangeError> {
        range.validate_in(self.as_str())?;
        match self {
            TextBuffer::String(s) => s.replace_range(range.as_range(), replacement),
        }
        Ok(())
    }
}

/// A lightweight projection of one outline node for UI lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineItem {
    pub id: NodeId,
    pub title: String,
    /// `None` for the synthetic root.
    pub level: Option<HeadingLevel>,
    pub child_count: usize,
}

/// A stable projection of one focused section (RFC-005): everything the UI
/// needs for breadcrumbs, the focus editor, and direct child cards, without
/// traversing core internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    pub node_id: NodeId,
    pub title: String,
    /// `None` when the root (pre-heading content) is focused.
    pub level: Option<HeadingLevel>,
    /// Exact body text of the focused section.
    pub body: String,
    /// Immediate children in source order.
    pub children: Vec<OutlineItem>,
    /// Breadcrumb path from the root to this node, inclusive.
    pub path: Vec<OutlineItem>,
    /// Revision this snapshot was taken from; commits carry it back.
    pub revision: DocumentRevision,
}

/// The canonical Markdown document and its derived state.
#[derive(Debug, Clone)]
pub struct Document {
    text: TextBuffer,
    outline: Outline,
    revision: DocumentRevision,
    history: EditHistory,
}

impl Document {
    /// Parses Markdown text into a document. The text is stored verbatim as
    /// the canonical source; only the outline is derived from it.
    pub fn parse(markdown: String) -> Result<Self, DocumentError> {
        let outline = build_outline(&markdown)?;
        Ok(Self {
            text: TextBuffer::from_string(markdown),
            outline,
            revision: DocumentRevision::INITIAL,
            history: EditHistory::default(),
        })
    }

    /// Decodes bytes as UTF-8 and parses them. Non-UTF-8 input is rejected
    /// without modification (load boundary used by the desktop crate).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, DocumentError> {
        let text = String::from_utf8(bytes).map_err(|_| DocumentError::InvalidUtf8)?;
        Self::parse(text)
    }

    /// The full canonical source text. Saving writes exactly this.
    pub fn source(&self) -> &str {
        self.text.as_str()
    }

    /// Current document revision.
    pub fn revision(&self) -> DocumentRevision {
        self.revision
    }

    /// The derived outline index.
    pub fn outline(&self) -> &Outline {
        &self.outline
    }

    /// Exact body text of one section.
    pub fn section_body(&self, id: NodeId) -> Result<&str, DocumentError> {
        let node = self
            .outline
            .node(id)
            .ok_or(DocumentError::NodeNotFound(id))?;
        Ok(self.text.slice(node.body_range)?)
    }

    /// Builds the focus projection for one node (RFC-005).
    pub fn focus_snapshot(&self, id: NodeId) -> Result<FocusSnapshot, DocumentError> {
        let node = self
            .outline
            .node(id)
            .ok_or(DocumentError::NodeNotFound(id))?;
        let item = |id: NodeId| -> Option<OutlineItem> {
            self.outline.node(id).map(|n| OutlineItem {
                id: n.id,
                title: n.title.clone(),
                level: n.level,
                child_count: n.children.len(),
            })
        };
        let children = node.children.iter().filter_map(|c| item(*c)).collect();
        let path = self
            .outline
            .path(id)
            .unwrap_or_default()
            .iter()
            .filter_map(|n| item(n.id))
            .collect();
        Ok(FocusSnapshot {
            node_id: node.id,
            title: node.title.clone(),
            level: node.level,
            body: self.text.slice(node.body_range)?.to_string(),
            children,
            path,
            revision: self.revision,
        })
    }

    /// Replaces one section body (RFC-004 semantics): the heading line, child
    /// sections, siblings, ancestors, and every unrelated byte are preserved.
    /// The replacement is stored verbatim — no whitespace normalization.
    pub fn replace_section_body(
        &mut self,
        cmd: ReplaceSectionBody,
    ) -> Result<EditResult, EditError> {
        if cmd.base_revision != self.revision {
            return Err(EditError::RevisionMismatch {
                expected: self.revision,
                actual: cmd.base_revision,
            });
        }
        let node = self
            .outline
            .node(cmd.node_id)
            .ok_or(EditError::StaleNode(cmd.node_id))?;
        let range = node.body_range;
        let old_text = self.text.slice(range)?.to_string();
        let result = self.apply_replacement(range, &cmd.new_body)?;
        self.history.record(EditRecord {
            replaced_range: result.replaced_range,
            old_text,
            new_range: result.new_range,
            new_text: cmd.new_body,
            revision_before: result.old_revision,
            revision_after: result.new_revision,
        });
        Ok(result)
    }

    /// Reverses the most recent committed edit (RFC-044). Byte-exact: the
    /// pre-edit source is restored exactly. Produces a fresh revision.
    pub fn undo(&mut self) -> Result<EditResult, EditError> {
        let record = self.history.pop_undo().ok_or(EditError::NothingToUndo)?;
        match self.apply_replacement(record.new_range, &record.old_text) {
            Ok(result) => {
                self.history.push_redo(record);
                Ok(result)
            }
            Err(error) => {
                self.history.push_undo(record);
                Err(error)
            }
        }
    }

    /// Re-applies the most recently undone edit (RFC-044).
    pub fn redo(&mut self) -> Result<EditResult, EditError> {
        let record = self.history.pop_redo().ok_or(EditError::NothingToRedo)?;
        match self.apply_replacement(record.replaced_range, &record.new_text) {
            Ok(result) => {
                self.history.push_undo(record);
                Ok(result)
            }
            Err(error) => {
                self.history.push_redo(record);
                Err(error)
            }
        }
    }

    /// Whether an undo is available.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether a redo is available.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// The RFC-008 transactional mutation path shared by edits, undo, and
    /// redo: validate → replace → re-index → increment revision; roll the
    /// text back unchanged if re-indexing fails.
    pub(crate) fn apply_replacement(
        &mut self,
        range: ByteRange,
        replacement: &str,
    ) -> Result<EditResult, EditError> {
        range.validate_in(self.text.as_str())?;
        let backup = self.text.as_str().to_string();
        self.text.replace(range, replacement)?;
        match build_outline(self.text.as_str()) {
            Ok(outline) => {
                let old_revision = self.revision;
                self.outline = outline;
                self.revision = self.revision.next();
                Ok(EditResult {
                    old_revision,
                    new_revision: self.revision,
                    replaced_range: range,
                    new_range: ByteRange {
                        start: range.start,
                        end: range.start + replacement.len(),
                    },
                    reindexed: true,
                })
            }
            Err(error) => {
                self.text = TextBuffer::from_string(backup);
                Err(EditError::IndexAfterEdit(error))
            }
        }
    }

    /// Records a structural edit in the history for undo/redo support.
    /// Called by `structural.rs` after `apply_replacement` succeeds.
    pub(crate) fn record_history(&mut self, record: crate::doc::history::EditRecord) {
        self.history.record(record);
    }

    // ── Structural editing (RFC-023..026) ──────────────────────────────────────

    /// Promotes the section heading one level (e.g. H3→H2). Only ATX headings
    /// are supported; H1 cannot be promoted. Affects the heading line only.
    pub fn promote_section(
        &mut self,
        id: NodeId,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::promote_section(self, id, base_revision)
    }

    /// Demotes the section heading one level (e.g. H2→H3). Only ATX headings
    /// are supported; H6 cannot be demoted.
    pub fn demote_section(
        &mut self,
        id: NodeId,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::demote_section(self, id, base_revision)
    }

    /// Renames the section heading while preserving the body, children,
    /// siblings, and unrelated source bytes.
    pub fn rename_section(
        &mut self,
        id: NodeId,
        new_title: &str,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::rename_section(self, id, new_title, base_revision)
    }

    /// Moves the entire section subtree (heading + body + descendants) to
    /// the given target position. Preserves every byte in the moved range.
    pub fn move_section(
        &mut self,
        id: NodeId,
        target: crate::doc::structural::MoveTarget,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::move_section(self, id, target, base_revision)
    }

    /// Inserts a new heading at `offset_in_body` bytes into the section body,
    /// splitting the body at that point. Empty offset appends at the top.
    pub fn split_section(
        &mut self,
        id: NodeId,
        offset_in_body: usize,
        new_title: &str,
        new_level: HeadingLevel,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::split_section(
            self,
            id,
            offset_in_body,
            new_title,
            new_level,
            base_revision,
        )
    }

    /// Removes `id`'s full range from the source. Requires explicit
    /// confirmation in the UI before calling (the subtree is permanently gone
    /// until undo).
    pub fn delete_section(
        &mut self,
        id: NodeId,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::delete_section(self, id, base_revision)
    }

    /// Merges `id` into its previous sibling by removing `id`'s heading line,
    /// making its body a continuation of the previous sibling's body.
    pub fn merge_with_prev_sibling(
        &mut self,
        id: NodeId,
        base_revision: DocumentRevision,
    ) -> Result<EditResult, crate::doc::structural::StructuralEditError> {
        crate::doc::structural::merge_with_prev_sibling(self, id, base_revision)
    }
}
