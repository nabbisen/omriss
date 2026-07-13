//! Structural-editing methods on [`super::EditorSession`] (RFC-023..026).

use omriss::EditResult;

impl super::EditorSession {
    // ── Structural editing façade (RFC-023..026) ─────────────────────────────

    fn focused_section_id(&self) -> Result<omriss::NodeId, omriss::StructuralEditError> {
        self.view
            .focused()
            .ok_or(omriss::StructuralEditError::NoFocusedSection)
    }

    /// Whether the focused section can be promoted (not H1, not setext).
    pub fn can_promote(&self) -> bool {
        self.view
            .focused()
            .and_then(|id| self.document.outline().node(id))
            .and_then(|n| n.level)
            .map(|l| l != omriss::HeadingLevel::H1)
            .unwrap_or(false)
    }

    /// Whether the focused section can be demoted (not H6, not setext).
    pub fn can_demote(&self) -> bool {
        self.view
            .focused()
            .and_then(|id| self.document.outline().node(id))
            .and_then(|n| n.level)
            .map(|l| l != omriss::HeadingLevel::H6)
            .unwrap_or(false)
    }

    /// Whether the focused section can be deleted (not root).
    pub fn can_delete(&self) -> bool {
        self.view
            .focused()
            .and_then(|id| self.document.outline().node(id))
            .map(|n| !n.is_root())
            .unwrap_or(false)
    }

    /// Whether the focused section can be merged with the previous sibling.
    pub fn can_merge_up(&self) -> bool {
        let Some(id) = self.view.focused() else {
            return false;
        };
        let outline = self.document.outline();
        let info = crate::editor::navigation::sibling_info(outline, id);
        info.prev_sibling
            .and_then(|prev_id| outline.node(prev_id))
            .map(|prev| prev.children.is_empty())
            .unwrap_or(false)
    }

    /// Promotes the focused section heading one level (RFC-023).
    pub fn promote_focused(&mut self) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        let result = self.document.promote_section(id, rev)?;
        let stale = self.prune_dead_history();
        if stale {
            self.view.show_outline();
        }
        Ok(result)
    }

    /// Demotes the focused section heading one level (RFC-023).
    pub fn demote_focused(&mut self) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        let result = self.document.demote_section(id, rev)?;
        self.prune_dead_history();
        Ok(result)
    }

    /// Moves the focused section to the given target (RFC-024).
    pub fn move_focused(
        &mut self,
        target: omriss::MoveTarget,
    ) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        let result = self.document.move_section(id, target, rev)?;
        self.prune_dead_history();
        Ok(result)
    }

    /// Splits the focused section body at `offset_in_body` bytes (RFC-025).
    pub fn split_focused(
        &mut self,
        offset_in_body: usize,
        new_title: &str,
        new_level: omriss::HeadingLevel,
    ) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        self.document
            .split_section(id, offset_in_body, new_title, new_level, rev)
    }

    /// Appends a new child section at the very end of the focused section —
    /// after all existing children — by splitting at `full_range.end`.
    /// This is the correct offset for "Add section inside" so that repeated
    /// additions produce children in the order they were added.
    pub fn append_child_to_focused(
        &mut self,
        new_title: &str,
        new_level: omriss::HeadingLevel,
    ) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let node = self
            .document
            .outline()
            .node(id)
            .ok_or(omriss::StructuralEditError::StaleNode(id))?;
        // offset_in_body must be relative to body_range.start.
        // full_range.end is past all children; body_range.start is after the
        // heading line. Their difference places the new heading at the bottom.
        let offset = node.full_range.end - node.body_range.start;
        let rev = self.document.revision();
        self.document
            .split_section(id, offset, new_title, new_level, rev)
    }

    /// Appends a new sibling section immediately after the focused section and
    /// its subtree. The new section uses the focused section's heading level.
    pub fn add_after_focused(
        &mut self,
        new_title: &str,
    ) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let node = self
            .document
            .outline()
            .node(id)
            .ok_or(omriss::StructuralEditError::StaleNode(id))?;
        let level = node
            .level
            .ok_or(omriss::StructuralEditError::CannotDeleteRoot)?;
        let offset = node.full_range.end - node.body_range.start;
        let rev = self.document.revision();
        self.document
            .split_section(id, offset, new_title, level, rev)
    }

    /// Renames the focused section heading without changing body or children.
    pub fn rename_focused(
        &mut self,
        new_title: &str,
    ) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        self.document.rename_section(id, new_title, rev)
    }

    /// Appends a new top-level H1 section at the end of the document —
    /// after all existing top-level sections. Uses `full_range.end` so
    /// repeated additions produce sections in the order they were added
    /// (same logic as `append_child_to_focused`).
    pub fn add_top_level_section(
        &mut self,
        title: &str,
    ) -> Result<EditResult, omriss::StructuralEditError> {
        let root_id = self.document.outline().root_id();
        let root = self
            .document
            .outline()
            .node(root_id)
            .ok_or(omriss::StructuralEditError::CannotDeleteRoot)?;
        // full_range.end covers the entire document; body_range.start is 0
        // for the synthetic root. Their difference places the new H1 after
        // every existing top-level section and its subtree.
        let offset = root.full_range.end - root.body_range.start;
        let rev = self.document.revision();
        self.document
            .split_section(root_id, offset, title, omriss::HeadingLevel::H1, rev)
    }

    /// Deletes the focused section and its subtree (RFC-025).
    /// The UI must confirm with the user before calling.
    pub fn delete_focused(&mut self) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        let result = self.document.delete_section(id, rev)?;
        self.view.show_outline();
        self.prune_dead_history();
        Ok(result)
    }

    /// Merges the focused section with its previous sibling (RFC-025).
    pub fn merge_focused_up(&mut self) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let rev = self.document.revision();
        let result = self.document.merge_with_prev_sibling(id, rev)?;
        self.prune_dead_history();
        Ok(result)
    }

    /// Move the focused section up one position among its siblings (RFC-024).
    pub fn move_focused_up(&mut self) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let info = crate::editor::navigation::sibling_info(self.document.outline(), id);
        let prev = info
            .prev_sibling
            .ok_or(omriss::StructuralEditError::NoAdjacentSibling)?;
        self.move_focused(omriss::MoveTarget::Before(prev))
    }

    /// Move the focused section down one position among its siblings (RFC-024).
    pub fn move_focused_down(&mut self) -> Result<EditResult, omriss::StructuralEditError> {
        let id = self.focused_section_id()?;
        let info = crate::editor::navigation::sibling_info(self.document.outline(), id);
        let next = info
            .next_sibling
            .ok_or(omriss::StructuralEditError::NoAdjacentSibling)?;
        self.move_focused(omriss::MoveTarget::After(next))
    }

    /// Whether the focused section can be moved up (has a previous sibling).
    pub fn can_move_up(&self) -> bool {
        self.view
            .focused()
            .map(|id| {
                crate::editor::navigation::sibling_info(self.document.outline(), id)
                    .prev_sibling
                    .is_some()
            })
            .unwrap_or(false)
    }

    /// Whether the focused section can be moved down (has a next sibling).
    pub fn can_move_down(&self) -> bool {
        self.view
            .focused()
            .map(|id| {
                crate::editor::navigation::sibling_info(self.document.outline(), id)
                    .next_sibling
                    .is_some()
            })
            .unwrap_or(false)
    }
}
