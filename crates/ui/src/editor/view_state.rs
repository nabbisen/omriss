//! Renderer-independent view state for the layer-by-layer workflow.
//!
//! The GUI shows either the whole-document outline or a single focused
//! section ("one layer at a time"). Navigation keeps browser-style
//! back/forward history of focused nodes. A raw-source overlay (RFC-017) can
//! be activated from any mode; it preserves the underlying mode so the user
//! returns to where they were. None of this touches the document text.

use omriss_core::NodeId;

/// What the main pane is currently showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// The whole-document outline (the "map").
    #[default]
    Outline,
    /// One section focused as the current working layer.
    Focus(NodeId),
    /// Full canonical Markdown source, read-only in M3 (RFC-017).
    RawSource,
}

/// Focus location plus browser-style navigation history.
///
/// Raw-source mode is an orthogonal overlay: it does not push a history
/// entry; leaving it returns to the mode that was active on entry.
#[derive(Debug, Clone, Default)]
pub struct ViewState {
    mode: ViewMode,
    back: Vec<ViewMode>,
    forward: Vec<ViewMode>,
    /// Saved mode to restore when leaving raw-source view.
    pre_raw: Option<ViewMode>,
}

impl ViewState {
    /// Starts at the whole-document outline.
    pub fn new() -> Self {
        Self::default()
    }

    /// The current view.
    pub fn mode(&self) -> ViewMode {
        self.mode
    }

    /// The currently focused node, if any.
    pub fn focused(&self) -> Option<NodeId> {
        let effective = self.pre_raw.unwrap_or(self.mode);
        match effective {
            ViewMode::Focus(id) => Some(id),
            ViewMode::Outline | ViewMode::RawSource => None,
        }
    }

    /// Whether `back()` would change the view.
    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether `forward()` would change the view.
    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Focuses `id`, pushing the previous view onto the back history and
    /// clearing the forward history (a new branch, like a browser).
    /// Re-focusing the current node is a no-op.
    pub fn focus(&mut self, id: NodeId) {
        self.leave_raw();
        self.navigate(ViewMode::Focus(id));
    }

    /// Returns to the whole-document outline through the same history rules.
    pub fn show_outline(&mut self) {
        self.leave_raw();
        self.navigate(ViewMode::Outline);
    }

    /// Enters the raw-source overlay (RFC-017). Does not push a history entry;
    /// `leave_raw()` restores exactly this mode.
    pub fn show_raw(&mut self) {
        if self.mode != ViewMode::RawSource {
            self.pre_raw = Some(self.mode);
            self.mode = ViewMode::RawSource;
        }
    }

    /// Leaves raw-source view, returning to the mode that was active on entry.
    pub fn leave_raw(&mut self) {
        if self.mode == ViewMode::RawSource {
            self.mode = self.pre_raw.take().unwrap_or(ViewMode::Outline);
        }
    }

    /// Whether the raw-source overlay is currently active.
    pub fn is_raw(&self) -> bool {
        self.mode == ViewMode::RawSource
    }

    /// Steps back in history; returns the new mode, or `None` at the start.
    pub fn back(&mut self) -> Option<ViewMode> {
        self.leave_raw();
        let previous = self.back.pop()?;
        self.forward.push(self.mode);
        self.mode = previous;
        Some(self.mode)
    }

    /// Steps forward in history; returns the new mode, or `None` at the end.
    pub fn forward(&mut self) -> Option<ViewMode> {
        self.leave_raw();
        let next = self.forward.pop()?;
        self.back.push(self.mode);
        self.mode = next;
        Some(self.mode)
    }

    /// Drops history entries (and the focus itself) that reference nodes no
    /// longer present, e.g. after a structural edit removed a section.
    /// `is_alive` reports whether a node still exists in the outline.
    pub fn retain_alive(&mut self, mut is_alive: impl FnMut(NodeId) -> bool) {
        let alive = |mode: &ViewMode, is_alive: &mut dyn FnMut(NodeId) -> bool| match mode {
            ViewMode::Outline | ViewMode::RawSource => true,
            ViewMode::Focus(id) => is_alive(*id),
        };
        self.back.retain(|mode| alive(mode, &mut is_alive));
        self.forward.retain(|mode| alive(mode, &mut is_alive));
        if let Some(pre) = &self.pre_raw
            && !alive(pre, &mut is_alive)
        {
            self.pre_raw = Some(ViewMode::Outline);
        }
        if !alive(&self.mode, &mut is_alive) {
            self.mode = ViewMode::Outline;
        }
    }

    fn navigate(&mut self, target: ViewMode) {
        if target == self.mode {
            return;
        }
        self.back.push(self.mode);
        self.forward.clear();
        self.mode = target;
    }
}
