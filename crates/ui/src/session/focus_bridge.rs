//! Format-aware focus and JSON's read-only focused-content accessor
//! (RFC-054 J7a — the read half of app wiring; committing edits is J7b).
//!
//! `EditorSession::focus`'s Markdown path is completely unchanged: it is
//! still the literal `document.focus_snapshot(id)` call every existing
//! `crates/ui/src/tests/session_tests.rs` assertion already exercises, so
//! that protected suite needed no change here. Every other format
//! validates `id` against that format's own `DocumentStructure` — built
//! fresh per RFC-054 §0.4, never cached — instead of the Markdown outline
//! a JSON-space `NodeId` was never a member of.

use omriss_core::{
    DocumentError, DocumentFormat, DocumentFormatAdapter, FocusSnapshot, JsonAdapter, NodeId,
};

impl super::EditorSession {
    /// Focuses a node and returns its snapshot. For Markdown this is
    /// exactly the pre-RFC-054 behavior: `document.focus_snapshot(id)`
    /// against the shared heading outline. For every other format
    /// (RFC-054 J7a), `id` is validated against that format's own adapter
    /// structure instead — a JSON-space id was never a member of the
    /// Markdown outline `document.focus_snapshot` reads, which is why
    /// this used to fail safely for JSON ids
    /// (`focusing_a_json_derived_node_id_fails_safely_without_corrupting_session_state`,
    /// now revisited: it documented a real safety property this slice
    /// deliberately replaces with a different one — see
    /// `session_format_tests.rs`). The returned snapshot carries only a
    /// real `title` and `revision` for non-Markdown formats; `body`,
    /// `children`, and `path` are empty placeholders, since
    /// `focused_structured_content()` is the real read path for those
    /// formats, not this snapshot.
    pub fn focus(&mut self, id: NodeId) -> Result<FocusSnapshot, DocumentError> {
        let snapshot = match self.format {
            DocumentFormat::Markdown => self.document.focus_snapshot(id)?,
            _ => {
                let structure = super::structure_bridge::build_structure_or_fallback(
                    self.format,
                    self.document.source(),
                    self.document.revision(),
                );
                let node = structure
                    .nodes
                    .iter()
                    .find(|n| n.id == id)
                    .ok_or(DocumentError::NodeNotFound(id))?;
                FocusSnapshot {
                    node_id: id,
                    title: node.title.clone(),
                    level: None,
                    body: String::new(),
                    children: Vec::new(),
                    path: Vec::new(),
                    revision: self.document.revision(),
                }
            }
        };
        self.view.focus(id);
        Ok(snapshot)
    }

    /// Read-only right-panel content for a focused JSON node (RFC-054
    /// J7a). Markdown continues to use `current_snapshot()`/
    /// `FocusSnapshot.body`; every other format has no real adapter yet.
    /// `None` when nothing is focused, the id no longer resolves, or the
    /// file failed to parse as JSON (the Document Map falls back to
    /// `PlainTextAdapter` silently in that case — RFC-052 §5.2 — so there
    /// is nothing a user could have focused to reach here; `None` is the
    /// defensive case, not the expected one).
    pub fn focused_structured_content(&self) -> Option<omriss_core::FocusedContent> {
        let id = self.view.focused()?;
        if self.format != DocumentFormat::Json {
            return None;
        }
        // Built directly, not via `build_structure_or_fallback`: this call
        // is adapter-specific (`JsonAdapter::focused_content` on a
        // `PlainTextAdapter`-built structure would be a category error),
        // so a parse failure here must mean "nothing to show," not "fall
        // back."
        let structure = JsonAdapter
            .build_structure(self.document.source(), self.document.revision())
            .ok()?;
        JsonAdapter
            .focused_content(self.document.source(), &structure, id)
            .ok()
    }
}
