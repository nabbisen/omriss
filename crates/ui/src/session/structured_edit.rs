//! Committing an edit to the focused JSON node (RFC-054 J7b — the write
//! half of app wiring). Read-only access lives in `focus_bridge.rs`; this
//! file is everything that can change `document`, plus the two read-only
//! helpers ([`focused_structured_raw_text`][EditorSession::focused_structured_raw_text],
//! [`structured_draft_differs`][EditorSession::structured_draft_differs])
//! those write paths need but J7a's read-only surface did not.

use omriss_core::{
    DocumentFormat, DocumentFormatAdapter, DraftState, EditResult, FocusedContent, JsonAdapter,
    StructureErrorKind, StructureNodeKind,
};

impl super::EditorSession {
    /// The focused JSON container's own full raw source text (RFC-054
    /// §7.4), for pre-populating its raw-text editor — `focused_content()`
    /// (via `focused_structured_content()`) only ever returns a truncated
    /// `summary`. `None` for anything but a focused JSON `Group`/`List`.
    pub fn focused_structured_raw_text(&self) -> Option<String> {
        if self.format != DocumentFormat::Json {
            return None;
        }
        let id = self.view.focused()?;
        let structure = JsonAdapter
            .build_structure(self.document.source(), self.document.revision())
            .ok()?;
        let node = structure.nodes.iter().find(|n| n.id == id)?;
        if !matches!(
            node.kind,
            StructureNodeKind::Group | StructureNodeKind::List
        ) {
            return None;
        }
        let range = node.editable_range?;
        self.document
            .source()
            .get(range.as_range())
            .map(String::from)
    }

    /// True when `draft` differs from the focused JSON node's current
    /// committed text — RFC-053 §9.3's dirty check, generalized past
    /// Markdown's `current_snapshot().body` comparison.
    pub fn structured_draft_differs(&self, draft: &str) -> bool {
        match self.focused_structured_content() {
            Some(FocusedContent::StructuredValue { editable_text, .. }) => draft != editable_text,
            Some(FocusedContent::StructuredGroup { .. }) => {
                self.focused_structured_raw_text().as_deref() != Some(draft)
            }
            _ => false,
        }
    }

    /// Live `DraftState` for `draft` against the focused JSON node
    /// (RFC-053 §9.3), without committing. `Clean` when unchanged,
    /// `InvalidUncommitted` when changed but not valid per RFC-054 §8/§8.5
    /// — the block-on-invalid gate a caller uses to disable navigation,
    /// save, and preview while the user is mid-edit on something invalid.
    pub fn structured_draft_state(&self, draft: &str) -> DraftState {
        if !self.structured_draft_differs(draft) {
            return DraftState::Clean;
        }
        match self.validate_structured_draft(draft) {
            Ok(()) => DraftState::ValidUncommitted,
            Err(_) => DraftState::InvalidUncommitted,
        }
    }

    /// Validates `draft` against the focused JSON node without committing.
    /// Rebuilds structure fresh (RFC-054 §0.4), same as every other
    /// structured read here — never cached across calls.
    pub fn validate_structured_draft(&self, draft: &str) -> Result<(), StructureErrorKind> {
        let id = self
            .view
            .focused()
            .ok_or(StructureErrorKind::InternalInvariantFailed)?;
        let structure = JsonAdapter
            .build_structure(self.document.source(), self.document.revision())
            .map_err(|e| e.kind)?;
        JsonAdapter
            .validate_focused_edit(self.document.source(), &structure, id, draft)
            .map(|_| ())
            .map_err(|e| e.kind)
    }

    /// Commits `draft` as an edit to the focused JSON node (RFC-054 J7b),
    /// mirroring `commit_focused_body`'s shape for Markdown: validate,
    /// then apply through the same `Document::replace_range` (J1) path
    /// J5/J6 already proved byte-preserving. The focused node's own id is
    /// unaffected by a value edit (only its content changes, never its
    /// tree position — RFC-054 §13.3), so the view stays focused on the
    /// same node after a successful commit.
    pub fn commit_structured_draft(
        &mut self,
        draft: &str,
    ) -> Result<EditResult, StructureErrorKind> {
        let id = self
            .view
            .focused()
            .ok_or(StructureErrorKind::InternalInvariantFailed)?;
        let structure = JsonAdapter
            .build_structure(self.document.source(), self.document.revision())
            .map_err(|e| e.kind)?;
        let edit = JsonAdapter
            .validate_focused_edit(self.document.source(), &structure, id, draft)
            .map_err(|e| e.kind)?;
        let applied = JsonAdapter
            .apply_validated_edit(&mut self.document, edit)
            .map_err(|e| e.kind)?;
        Ok(applied.result)
    }
}
