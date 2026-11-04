//! Committing an edit of the plain file text (RFC-076).
//!
//! The raw view edits the document's source directly, so this is the one
//! mutation that can change every heading at once. Three things hold and none
//! is negotiable: the commit is **one** transaction and one undo step (it goes
//! through `Document::replace_range`, the same primitive JSON uses, so
//! `omriss-core` does not change), the user is told when the structure changed
//! (it is bracketed by RFC-071's detector like every other source rewrite), and
//! focus returns to the outline instead of being guessed at.

use omriss_core::{ByteRange, DocumentRevision, EditError, EditResult};

use crate::editor::raw_text::{as_shown_in_a_text_area, raw_edit};
use crate::editor::structure_change::Operation;

/// What a raw-text commit did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawCommit {
    /// The draft shows exactly what the document shows. Nothing was written, no
    /// history entry was made, and no warning was raised or cleared.
    Unchanged,
    /// The document was edited, as one undo step.
    Applied(EditResult),
}

impl super::EditorSession {
    /// The source as the raw view's text area shows it: line endings as `\n`.
    /// This is the text the raw draft starts from and is compared against.
    pub fn raw_text(&self) -> String {
        as_shown_in_a_text_area(self.document.source())
    }

    /// The revision a raw edit is composed against. The commit is refused if the
    /// document has moved on since (strict equality, as for every other edit).
    pub fn revision(&self) -> DocumentRevision {
        self.document.revision()
    }

    /// Whether the raw view is currently in edit mode. False whenever the raw
    /// view is not showing, so a stale flag can never make a later visit
    /// editable.
    pub fn raw_editing(&self) -> bool {
        self.raw_editing && self.view.is_raw()
    }

    /// Makes the raw view editable and returns the text its draft starts from.
    /// Does nothing (and returns an empty string) outside the raw view.
    pub fn begin_raw_edit(&mut self) -> String {
        if !self.view.is_raw() {
            return String::new();
        }
        self.raw_editing = true;
        self.reload_raw_draft()
    }

    /// Returns the raw view to read-only. The caller commits any pending draft
    /// first; this discards nothing and writes nothing.
    pub fn end_raw_edit(&mut self) {
        self.raw_editing = false;
    }

    /// The text a raw draft should now hold, recording the revision it is
    /// composed against. Call whenever the draft is reset to the document: on
    /// entering edit mode, after a commit, after undo or redo.
    pub fn reload_raw_draft(&mut self) -> String {
        self.raw_base = Some(self.document.revision());
        self.raw_text()
    }

    /// Commits the raw draft against the revision [`reload_raw_draft`] recorded.
    ///
    /// [`reload_raw_draft`]: Self::reload_raw_draft
    pub fn commit_raw_draft(&mut self, draft: &str) -> Result<RawCommit, EditError> {
        let base = self.raw_base.unwrap_or_else(|| self.document.revision());
        self.commit_raw_text(draft, base)
    }

    /// Whether `draft` would change the document if committed.
    pub fn raw_draft_differs(&self, draft: &str) -> bool {
        raw_edit(self.document.source(), draft).is_some()
    }

    /// Commits a raw-text `draft` composed against `base_revision`.
    ///
    /// A draft is compared with the source as a text area shows it and applied
    /// as one minimal replacement of the original bytes, so line endings the
    /// user did not touch survive (`editor::raw_text` explains why this is not
    /// "replace everything with the draft"). A draft that changes nothing writes
    /// nothing. On success focus is the whole-document outline and navigation
    /// history is cleared: nothing is remapped, because a section that still
    /// resolves by id after the document was rewritten is a coincidence, not the
    /// section the user was in.
    pub fn commit_raw_text(
        &mut self,
        draft: &str,
        base_revision: DocumentRevision,
    ) -> Result<RawCommit, EditError> {
        let Some(edit) = raw_edit(self.document.source(), draft) else {
            // Still refuse a stale revision: "nothing to commit" must not hide
            // that the draft was composed against a different document.
            if base_revision != self.document.revision() {
                return Err(EditError::RevisionMismatch {
                    expected: self.document.revision(),
                    actual: base_revision,
                });
            }
            return Ok(RawCommit::Unchanged);
        };
        let range = ByteRange::new(edit.range.start, edit.range.end)?;
        let pending = self.begin_change(Operation::RawCommit);
        let result = self
            .document
            .replace_range(range, edit.replacement, base_revision)?;
        self.finish_change(pending);
        self.view.reset_to_outline();
        Ok(RawCommit::Applied(result))
    }
}
