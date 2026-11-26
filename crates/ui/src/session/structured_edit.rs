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

use crate::editor::raw_text::{apply_text_area_draft, newline_of, text_area_edit};

impl super::EditorSession {
    /// The text a structured `draft` means for the focused JSON node, as source:
    /// for a container's raw-text editor (a text area, which only ever delivers
    /// `\n`) the original text with just the user's change applied and inserted
    /// newlines in the document's style (RFC-084); for a scalar's one-line input
    /// the draft itself. `None` when a container's draft changes nothing.
    fn structured_draft_source(&self, draft: &str) -> Option<String> {
        match self.focused_structured_raw_text() {
            Some(original) => {
                apply_text_area_draft(&original, draft, newline_of(self.document.source()))
            }
            None => Some(draft.to_string()),
        }
    }

    /// The focused JSON container's own full raw source text (RFC-054
    /// §7.4), for pre-populating its raw-text editor — `focused_content()`
    /// (via `focused_structured_content()`) only ever returns a truncated
    /// `summary`. `None` for anything but a focused JSON `Group`/`List`.
    pub fn focused_structured_raw_text(&self) -> Option<String> {
        if self.format != DocumentFormat::Json {
            return None;
        }
        let id = self.view.focused()?;
        let structure = self.structure_result().ok()?;
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
                // In the text-area view, so a CRLF container the user has not
                // changed is not "pending" (RFC-084 §4.2).
                match self.focused_structured_raw_text() {
                    Some(original) => {
                        text_area_edit(&original, draft, newline_of(self.document.source()))
                            .is_some()
                    }
                    None => true,
                }
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
    /// Structure comes from the revision-keyed cache (RFC-067 §3.1) — sound
    /// per RFC-054 §0.4 because a draft never bumps the revision, so this
    /// and every other read below it in one render/keystroke share a single
    /// build rather than one each.
    pub fn validate_structured_draft(&self, draft: &str) -> Result<(), StructureErrorKind> {
        let id = self
            .view
            .focused()
            .ok_or(StructureErrorKind::InternalInvariantFailed)?;
        let structure = self.structure_result().map_err(|e| e.kind)?;
        let source_text = self
            .structured_draft_source(draft)
            .unwrap_or_else(|| draft.to_string());
        JsonAdapter
            .validate_focused_edit(self.document.source(), &structure, id, &source_text)
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
        let structure = self.structure_result().map_err(|e| e.kind)?;
        let source_text = self
            .structured_draft_source(draft)
            .unwrap_or_else(|| draft.to_string());
        let edit = JsonAdapter
            .validate_focused_edit(self.document.source(), &structure, id, &source_text)
            .map_err(|e| e.kind)?;
        let applied = JsonAdapter
            .apply_validated_edit(&mut self.document, edit)
            .map_err(|e| e.kind)?;
        // A non-Markdown document has no heading outline to protect (RFC-071).
        self.structure_change = None;
        Ok(applied.result)
    }
}
