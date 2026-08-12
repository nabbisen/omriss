//! Editor session: one open document plus everything the GUI needs around it.
//!
//! `EditorSession` glues `omriss_core::Document` to the view state: it owns
//! dirty tracking (content-based, so undoing back to the saved bytes clears
//! the flag), routes focused-body edits through the core command API, and
//! prunes navigation history when sections disappear after structural edits.
//! It also stores the `FileTextProfile` detected when the file was opened
//! (line endings, BOM, trailing newline — RFC-018).

use std::hash::{DefaultHasher, Hash, Hasher};

use omriss_core::{
    Document, DocumentError, DocumentFormat, DocumentRevision, EditError, EditResult,
    FocusSnapshot, OutlineItem, ReplaceSectionBody,
};

use crate::editor::view_state::{ViewMode, ViewState};
use crate::file::file_profile::FileTextProfile;

/// Length + hash of the text at the last save. Undo and redo mint fresh
/// revisions (RFC-044), so "back to the saved bytes" must be detected by
/// content, not by revision number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SavedFingerprint {
    revision: DocumentRevision,
    len: usize,
    hash: u64,
}

impl SavedFingerprint {
    fn of(text: &str, revision: DocumentRevision) -> Self {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        Self {
            revision,
            len: text.len(),
            hash: hasher.finish(),
        }
    }

    fn matches(&self, text: &str, revision: DocumentRevision) -> bool {
        if revision == self.revision {
            return true; // fast path: nothing committed since the save
        }
        Self::of(text, revision).hash == self.hash && text.len() == self.len
    }
}

/// One open document and its surrounding editor state.
#[derive(Debug, Clone)]
pub struct EditorSession {
    document: Document,
    view: ViewState,
    saved: SavedFingerprint,
    /// Display name or path of the backing file, if any.
    file_name: Option<String>,
    /// File-level text characteristics detected at open time (RFC-018).
    profile: FileTextProfile,
    /// True once the user has explicitly opened or created a document.
    /// False only for the initial startup state — used to decide whether
    /// to show the welcome screen.
    document_open: bool,
    /// Which format's structure this session should project (RFC-054 J3).
    /// `document` itself is always a parsed Markdown `Document` regardless
    /// of `format` (RFC-054 §0.1: a non-Markdown document "rides on
    /// `Document`" and simply ignores the heading outline it builds) — this
    /// field is what tells `document_map_nodes()` which adapter's structure
    /// to project instead of that outline.
    format: DocumentFormat,
}

mod document_map_bridge;
mod focus_bridge;
mod outline_bridge;
mod structural;
mod structure_bridge;
mod structured_edit;
pub use outline_bridge::OutlineNode;

impl EditorSession {
    /// Opens a session over Markdown text (e.g. file contents just read).
    pub fn open(markdown: String, file_name: Option<String>) -> Result<Self, DocumentError> {
        let profile = FileTextProfile::detect(&markdown, false);
        Self::open_detected(markdown, file_name, profile, DocumentFormat::Markdown)
    }

    /// Opens a session with a pre-computed file text profile (RFC-018). Use
    /// this when the desktop crate has already stripped a BOM or detected
    /// line endings before passing the text here.
    pub fn open_with_profile(
        markdown: String,
        file_name: Option<String>,
        profile: FileTextProfile,
    ) -> Result<Self, DocumentError> {
        Self::open_detected(markdown, file_name, profile, DocumentFormat::Markdown)
    }

    /// Opens a session for a file whose format has already been classified
    /// (RFC-054 J3; `omriss_core::formats::detection::detect_format`). The
    /// desktop crate is where a real file path is available, so it detects
    /// the format and passes it in here — the same "detected upstream,
    /// supplied here" shape `profile` already uses.
    ///
    /// `Document::parse` is called regardless of `format`: for Markdown it
    /// is the real parse; for every other format it still succeeds (RFC-002's
    /// heading parser accepts arbitrary text) but its outline is simply
    /// never read again once `format` is not `Markdown` (RFC-054 §0.1).
    pub fn open_detected(
        text: String,
        file_name: Option<String>,
        profile: FileTextProfile,
        format: DocumentFormat,
    ) -> Result<Self, DocumentError> {
        let document = Document::parse(text)?;
        let saved = SavedFingerprint::of(document.source(), document.revision());
        Ok(Self {
            document,
            view: ViewState::new(),
            saved,
            file_name,
            profile,
            document_open: true,
            format,
        })
    }

    /// The format this session's structure is projected as (RFC-054 J3).
    pub fn format(&self) -> DocumentFormat {
        self.format
    }

    /// Starts an empty, unsaved document.
    ///
    /// `document_open` is `false` — this is the initial startup placeholder.
    /// Use [`new_document`][Self::new_document] when the user explicitly
    /// chooses *New* so the welcome screen is dismissed.
    pub fn new_empty() -> Self {
        let mut s = Self::open(String::new(), None).expect("empty text always parses");
        s.document_open = false;
        s
    }

    /// Creates a blank document as if the user chose *New*.
    ///
    /// Identical to `new_empty` except `document_open` is `true`, which
    /// dismisses the welcome screen and shows the editor.
    pub fn new_document() -> Self {
        Self::open(String::new(), None).expect("empty text always parses")
    }

    /// True once the user has explicitly opened or created a document.
    /// False only at startup (initial empty state).
    pub fn document_open(&self) -> bool {
        self.document_open
    }

    /// The canonical source text — exactly what `save` must write.
    pub fn source(&self) -> &str {
        self.document.source()
    }

    /// Display name of the backing file, if any.
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// File text profile detected at open time (line endings, BOM — RFC-018).
    pub fn profile(&self) -> &FileTextProfile {
        &self.profile
    }

    /// True when the text differs byte-wise from the last saved text.
    pub fn is_dirty(&self) -> bool {
        !self
            .saved
            .matches(self.document.source(), self.document.revision())
    }

    /// Records that the current text was written out successfully.
    pub fn mark_saved(&mut self, file_name: Option<String>) {
        self.saved = SavedFingerprint::of(self.document.source(), self.document.revision());
        if file_name.is_some() {
            self.file_name = file_name;
        }
    }

    /// Current view (outline, focused section, or raw-source overlay).
    pub fn view_mode(&self) -> ViewMode {
        self.view.mode()
    }

    /// Whether the raw-source overlay is currently active (RFC-017).
    pub fn is_raw(&self) -> bool {
        self.view.is_raw()
    }

    /// Enters the read-only raw-source overlay; returns to structured view via
    /// `leave_raw()`. Does not push a navigation history entry.
    pub fn show_raw(&mut self) {
        self.view.show_raw();
    }

    /// Leaves the raw-source overlay, returning to the mode that was active.
    pub fn leave_raw(&mut self) {
        self.view.leave_raw();
    }

    /// Read-only navigation state for toolbar enablement.
    pub fn can_go_back(&self) -> bool {
        self.view.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        self.view.can_go_forward()
    }

    pub fn can_undo(&self) -> bool {
        self.document.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.document.can_redo()
    }

    /// Top-level outline rows: the root's direct children, or just the root
    /// for a document without headings.
    pub fn outline_items(&self) -> Vec<OutlineItem> {
        let root = self
            .document
            .focus_snapshot(self.document.outline().root_id())
            .expect("root always exists");
        if root.children.is_empty() {
            vec![OutlineItem {
                id: root.node_id,
                title: root.title,
                level: root.level,
                child_count: 0,
            }]
        } else {
            root.children
        }
    }

    /// Outline items relevant to the current view:
    /// - In overview mode: the top-level items (same as [`outline_items`]).
    /// - In focus mode: the immediate children of the focused section.
    ///
    /// Used to populate the Document Map and the card list for keyboard
    /// selection (RFC-011, RFC-014).
    pub fn current_children(&self) -> Vec<OutlineItem> {
        match self.view.focused() {
            Some(_) => self
                .current_snapshot()
                .map(|s| s.children)
                .unwrap_or_default(),
            None => self.outline_items(),
        }
    }

    /// Zooms out one level: moves focus to the parent section, or returns to
    /// the outline overview if the current focus is a direct child of the root.
    ///
    /// This is the structural "Esc" action (RFC-014). It pushes the current
    /// node onto the back history (via `ViewState::focus`/`show_outline`) so
    /// the user can return forward with Alt+Right.
    pub fn zoom_out(&mut self) {
        let Some(id) = self.view.focused() else {
            return;
        };
        let parent_id = self.document.outline().node(id).and_then(|n| n.parent_id);
        match parent_id {
            Some(pid) if pid != self.document.outline().root_id() => {
                self.view.focus(pid);
            }
            _ => {
                self.view.show_outline();
            }
        }
        self.prune_dead_history();
    }

    /// Returns to the whole-document outline view.
    pub fn show_outline(&mut self) {
        self.view.show_outline();
    }

    /// Browser-style back; returns the snapshot if a section is now focused.
    pub fn back(&mut self) -> Option<FocusSnapshot> {
        self.view.back()?;
        self.current_snapshot()
    }

    /// Browser-style forward; returns the snapshot if a section is now focused.
    pub fn forward(&mut self) -> Option<FocusSnapshot> {
        self.view.forward()?;
        self.current_snapshot()
    }

    /// Snapshot of the focused section, if the view is in focus mode.
    pub fn current_snapshot(&self) -> Option<FocusSnapshot> {
        let id = self.view.focused()?;
        self.document.focus_snapshot(id).ok()
    }

    /// Replaces the body of the currently focused section. The snapshot the
    /// edit was composed against supplies the revision check (RFC-008).
    pub fn commit_focused_body(
        &mut self,
        base: &FocusSnapshot,
        new_body: String,
    ) -> Result<EditResult, EditError> {
        let new_body = self.body_for_focused_editor_commit(base, new_body)?;
        let result = self.document.replace_section_body(ReplaceSectionBody {
            node_id: base.node_id,
            base_revision: base.revision,
            new_body,
        })?;
        self.prune_dead_history();
        Ok(result)
    }

    fn body_for_focused_editor_commit(
        &self,
        base: &FocusSnapshot,
        mut new_body: String,
    ) -> Result<String, EditError> {
        let node = self
            .document
            .outline()
            .node(base.node_id)
            .ok_or(EditError::StaleNode(base.node_id))?;
        if !new_body.is_empty()
            && !new_body.ends_with('\n')
            && node.body_range.end < self.document.source().len()
        {
            new_body.push('\n');
        }
        Ok(new_body)
    }

    /// Undoes the most recent edit (RFC-044) and prunes dead focus targets.
    pub fn undo(&mut self) -> Result<EditResult, EditError> {
        let result = self.document.undo()?;
        self.prune_dead_history();
        Ok(result)
    }

    /// Re-applies the most recently undone edit.
    pub fn redo(&mut self) -> Result<EditResult, EditError> {
        let result = self.document.redo()?;
        self.prune_dead_history();
        Ok(result)
    }

    /// Prunes dead history and returns `true` if a stale focus target was
    /// encountered (the UI can then show a non-blocking status message per
    /// RFC-019).
    pub fn prune_and_report(&mut self) -> bool {
        self.prune_dead_history()
    }

    /// Navigation availability from the currently focused node (RFC-020).
    pub fn sibling_info(&self) -> crate::editor::navigation::SiblingInfo {
        match self.view.focused() {
            Some(id) => crate::editor::navigation::sibling_info(self.document.outline(), id),
            None => crate::editor::navigation::SiblingInfo::default(),
        }
    }

    /// Navigates to the parent section (or overview if parent is root).
    /// Returns `true` if navigation happened.
    pub fn navigate_parent(&mut self) -> bool {
        let Some(id) = self.view.focused() else {
            return false;
        };
        let outline = self.document.outline();
        let Some(node) = outline.node(id) else {
            return false;
        };
        match node.parent_id {
            Some(pid) if pid != outline.root_id() => {
                self.view.focus(pid);
                true
            }
            Some(_) => {
                self.view.show_outline();
                true
            }
            None => false,
        }
    }

    /// Navigates to the first child of the focused section.
    pub fn navigate_first_child(&mut self) -> bool {
        let Some(id) = self.view.focused() else {
            return false;
        };
        let outline = self.document.outline();
        let Some(node) = outline.node(id) else {
            return false;
        };
        let Some(&child_id) = node.children.first() else {
            return false;
        };
        self.view.focus(child_id);
        true
    }

    /// Navigates to the previous sibling in source order.
    pub fn navigate_prev_sibling(&mut self) -> bool {
        let Some(id) = self.view.focused() else {
            return false;
        };
        let info = crate::editor::navigation::sibling_info(self.document.outline(), id);
        if let Some(prev) = info.prev_sibling {
            self.view.focus(prev);
            true
        } else {
            false
        }
    }

    /// Navigates to the next sibling in source order.
    pub fn navigate_next_sibling(&mut self) -> bool {
        let Some(id) = self.view.focused() else {
            return false;
        };
        let info = crate::editor::navigation::sibling_info(self.document.outline(), id);
        if let Some(next) = info.next_sibling {
            self.view.focus(next);
            true
        } else {
            false
        }
    }

    /// Searches the whole document for `query` (RFC-021).
    pub fn search_document(&self, query: &str) -> Vec<crate::editor::search::SearchMatch> {
        crate::editor::search::search_document(&self.document, query)
    }

    /// Searches only the focused section's body for `query`.
    pub fn search_section(&self, query: &str) -> Vec<crate::editor::search::SearchMatch> {
        match self.view.focused() {
            Some(id) => crate::editor::search::search_section(&self.document, id, query),
            None => crate::editor::search::search_document(&self.document, query),
        }
    }

    /// Returns document statistics (RFC-046): word counts and section count.
    pub fn stats(&self) -> crate::editor::stats::DocumentStats {
        crate::editor::stats::compute_stats(&self.document, self.view.focused())
    }

    /// Renders the focused section body as HTML for the preview pane (RFC-045).
    /// Returns an empty string when no section is focused or body is empty.
    pub fn preview_html(&self) -> String {
        self.view
            .focused()
            .and_then(|id| omriss_core::section_html(&self.document, id))
            .unwrap_or_default()
    }
}

impl Default for EditorSession {
    fn default() -> Self {
        Self::new_empty()
    }
}
