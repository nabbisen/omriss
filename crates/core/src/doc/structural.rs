//! Structural editing: promote/demote, move, split, delete, merge (RFC-023..026).
//!
//! All operations go through `apply_replacement`, so they:
//! - roll back automatically if re-indexing fails (RFC-008 transaction shape);
//! - record an `EditRecord` for byte-exact undo/redo (RFC-044);
//! - never partially mutate the document.
//!
//! Operations on Setext headings (level detected from underline style) are
//! rejected for promote/demote in M5; convert them to ATX first via raw view.

use crate::doc::edit::EditResult;
use crate::doc::history::EditRecord;
use crate::doc::revision::DocumentRevision;
use crate::error::{EditError, IndexError};
use crate::index::outline::{HeadingLevel, NodeId};
use crate::range::ByteRange;
use crate::{Document, Outline};

// ── Error ─────────────────────────────────────────────────────────────────────

/// Errors raised by structural editing operations (RFC-026 conflict taxonomy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralEditError {
    /// The edit was based on a stale revision; caller must reload (RFC-002).
    RevisionMismatch {
        expected: DocumentRevision,
        actual: DocumentRevision,
    },
    /// Source node no longer exists in the current outline.
    StaleNode(NodeId),
    /// Target node no longer exists in the current outline (move ops).
    StaleTarget(NodeId),
    /// Setext headings are not supported for promote/demote in M5.
    UnsupportedHeadingStyle,
    /// Cannot promote an H1 or demote an H6.
    InvalidLevel,
    /// Cannot move a section into one of its own descendants.
    CannotMoveIntoDescendant,
    /// Cannot move a section before/after itself.
    CannotMoveSelf,
    /// The operation requires a focused section, but no section is focused.
    NoFocusedSection,
    /// Cannot delete the synthetic root node.
    CannotDeleteRoot,
    /// No adjacent sibling of the right kind to merge with.
    NoAdjacentSibling,
    /// Operation would risk moving content under the wrong section.
    UnsafePreservation,
    /// Split offset falls outside the section's body range.
    InvalidSplitOffset,
    /// Section title is empty or cannot be represented safely.
    InvalidTitle,
    /// The underlying text replacement failed (re-index error or invalid range).
    Edit(EditError),
}

impl std::fmt::Display for StructuralEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionMismatch { expected, actual } => write!(
                f,
                "revision mismatch: document is at {expected:?}, edit based on {actual:?}"
            ),
            Self::StaleNode(id) => write!(f, "source node stale: {id:?}"),
            Self::StaleTarget(id) => write!(f, "target node stale: {id:?}"),
            Self::UnsupportedHeadingStyle => {
                write!(
                    f,
                    "promote/demote requires ATX headings (# …); convert setext first"
                )
            }
            Self::InvalidLevel => write!(
                f,
                "heading level limit reached (H1 cannot be promoted; H6 cannot be demoted)"
            ),
            Self::CannotMoveIntoDescendant => {
                write!(f, "cannot move a section into its own descendant")
            }
            Self::CannotMoveSelf => write!(f, "source and target are the same section"),
            Self::NoFocusedSection => write!(f, "no focused section"),
            Self::CannotDeleteRoot => write!(f, "cannot delete the root node"),
            Self::NoAdjacentSibling => write!(f, "no adjacent sibling to merge with"),
            Self::UnsafePreservation => write!(f, "operation could change unrelated structure"),
            Self::InvalidSplitOffset => write!(f, "split offset is outside the section body"),
            Self::InvalidTitle => write!(f, "section title is empty or invalid"),
            Self::Edit(e) => write!(f, "underlying edit error: {e}"),
        }
    }
}

impl std::error::Error for StructuralEditError {}

impl From<EditError> for StructuralEditError {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}

impl From<IndexError> for StructuralEditError {
    fn from(e: IndexError) -> Self {
        Self::Edit(EditError::IndexAfterEdit(e))
    }
}

// ── Move target ───────────────────────────────────────────────────────────────

/// Where to place a moved section relative to another node (RFC-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    /// Place the moved section immediately before `node` (same parent level).
    Before(NodeId),
    /// Place the moved section immediately after `node` (same parent level).
    After(NodeId),
    /// Place as the first child of `node`.
    AsFirstChildOf(NodeId),
    /// Place as the last child of `node`.
    AsLastChildOf(NodeId),
}

// ── Preflight helpers ─────────────────────────────────────────────────────────

fn check_revision(doc: &Document, base: DocumentRevision) -> Result<(), StructuralEditError> {
    if doc.revision() != base {
        return Err(StructuralEditError::RevisionMismatch {
            expected: doc.revision(),
            actual: base,
        });
    }
    Ok(())
}

fn is_descendant(outline: &Outline, ancestor: NodeId, candidate: NodeId) -> bool {
    outline
        .path(candidate)
        .map(|path| path.iter().any(|n| n.id == ancestor))
        .unwrap_or(false)
}

// ── Promote / Demote (RFC-023) ────────────────────────────────────────────────

/// Builds a new ATX marker string for `current_level ± delta`.
/// Returns `None` if the result would be outside H1..H6.
fn adjusted_marker(current_level: HeadingLevel, delta: i8) -> Option<String> {
    let new_depth = current_level.as_u8() as i8 + delta;
    if !(1..=6).contains(&new_depth) {
        return None;
    }
    Some("#".repeat(new_depth as usize))
}

/// Core of promote/demote: shifts the selected section subtree one heading
/// level while preserving its internal parent/child relationships.
fn change_heading_level(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
    delta: i8, // -1 = promote, +1 = demote
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    let level = node.level.ok_or(StructuralEditError::StaleNode(id))?;
    adjusted_marker(level, delta).ok_or(StructuralEditError::InvalidLevel)?;

    let source = doc.source();
    let outline = doc.outline();
    let subtree_ids: Vec<NodeId> = outline
        .iter()
        .filter(|candidate| is_descendant(outline, id, candidate.id))
        .map(|candidate| candidate.id)
        .collect();
    let mut marker_edits = Vec::with_capacity(subtree_ids.len());
    for node_id in subtree_ids {
        let n = outline
            .node(node_id)
            .ok_or(StructuralEditError::StaleNode(node_id))?;
        let level = n.level.ok_or(StructuralEditError::StaleNode(node_id))?;
        let heading_start = n.heading_range.start;
        if source.as_bytes().get(heading_start) != Some(&b'#') {
            return Err(StructuralEditError::UnsupportedHeadingStyle);
        }
        let new_marker = adjusted_marker(level, delta).ok_or(StructuralEditError::InvalidLevel)?;
        let old_marker_len = level.as_u8() as usize;
        marker_edits.push((heading_start, old_marker_len, new_marker));
    }

    let mut shifted_subtree = source[node.full_range.as_range()].to_string();
    for (heading_start, old_marker_len, new_marker) in marker_edits.iter().rev() {
        let local_start = heading_start - node.full_range.start;
        shifted_subtree.replace_range(local_start..local_start + old_marker_len, new_marker);
    }

    let new_source = if delta < 0 && has_following_sibling(outline, id)? {
        let parent_id = node.parent_id.ok_or(StructuralEditError::InvalidLevel)?;
        let parent = outline
            .node(parent_id)
            .ok_or(StructuralEditError::StaleNode(parent_id))?;
        let range = node.full_range;
        let mut s = String::with_capacity(source.len() - range.len() + shifted_subtree.len());
        s.push_str(&source[..range.start]);
        s.push_str(&source[range.end..parent.full_range.end]);
        s.push_str(&shifted_subtree);
        s.push_str(&source[parent.full_range.end..]);
        s
    } else {
        let range = node.full_range;
        let mut s = String::with_capacity(source.len() - range.len() + shifted_subtree.len());
        s.push_str(&source[..range.start]);
        s.push_str(&shifted_subtree);
        s.push_str(&source[range.end..]);
        s
    };

    let full_range = ByteRange {
        start: 0,
        end: source.len(),
    };
    let old_source = source.to_string();
    let result = doc.apply_replacement(full_range, &new_source)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text: old_source,
        new_range: result.new_range,
        new_text: new_source,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

fn has_following_sibling(outline: &Outline, id: NodeId) -> Result<bool, StructuralEditError> {
    let node = outline.node(id).ok_or(StructuralEditError::StaleNode(id))?;
    let Some(parent_id) = node.parent_id else {
        return Ok(false);
    };
    let parent = outline
        .node(parent_id)
        .ok_or(StructuralEditError::StaleNode(parent_id))?;
    let Some(position) = parent.children.iter().position(|child| *child == id) else {
        return Err(StructuralEditError::StaleNode(id));
    };
    Ok(position + 1 < parent.children.len())
}

pub(crate) fn promote_section(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    change_heading_level(doc, id, base_rev, -1)
}

pub(crate) fn demote_section(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    change_heading_level(doc, id, base_rev, 1)
}

// ── Rename section (RFC-049) ─────────────────────────────────────────────────

pub(crate) fn rename_section(
    doc: &mut Document,
    id: NodeId,
    new_title: &str,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let new_title = new_title.trim();
    if new_title.is_empty() || new_title.contains('\n') || new_title.contains('\r') {
        return Err(StructuralEditError::InvalidTitle);
    }

    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }

    let source = doc.source();
    let heading = &source[node.heading_range.as_range()];
    let line_len = heading.find('\n').unwrap_or(heading.len());
    let line = &heading[..line_len];
    let title_range = if line.starts_with('#') {
        atx_title_range(node.heading_range.start, line).ok_or(StructuralEditError::InvalidTitle)?
    } else {
        ByteRange::new(
            node.heading_range.start,
            node.heading_range.start + line_len,
        )
        .map_err(|_| StructuralEditError::InvalidTitle)?
    };

    let old_text = source[title_range.as_range()].to_string();
    let result = doc.apply_replacement(title_range, new_title)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text,
        new_range: result.new_range,
        new_text: new_title.to_string(),
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

fn atx_title_range(line_start: usize, line: &str) -> Option<ByteRange> {
    let bytes = line.as_bytes();
    let marker_len = bytes.iter().take_while(|&&b| b == b'#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }

    let mut title_start = marker_len;
    while matches!(bytes.get(title_start), Some(b' ' | b'\t')) {
        title_start += 1;
    }

    let mut title_end = line.len();
    while title_end > title_start && matches!(bytes[title_end - 1], b' ' | b'\t') {
        title_end -= 1;
    }

    let mut hash_start = title_end;
    while hash_start > title_start && bytes[hash_start - 1] == b'#' {
        hash_start -= 1;
    }
    if hash_start < title_end && hash_start > title_start {
        let mut before_hash = hash_start;
        while before_hash > title_start && matches!(bytes[before_hash - 1], b' ' | b'\t') {
            before_hash -= 1;
        }
        if before_hash < hash_start {
            title_end = before_hash;
        }
    }

    ByteRange::new(line_start + title_start, line_start + title_end).ok()
}

// ── Delete section (RFC-025) ──────────────────────────────────────────────────

pub(crate) fn delete_section(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }
    let range = node.full_range;
    let old_text = doc.source()[range.as_range()].to_string();
    let result = doc.apply_replacement(range, "")?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text,
        new_range: result.new_range,
        new_text: String::new(),
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

// ── Split section (RFC-025) ───────────────────────────────────────────────────

/// Inserts a new heading of `new_level` with `new_title` at `offset_in_body`
/// bytes into the focused section's body. The text before the offset stays as
/// the current section's body; the text after becomes the new section's body.
pub(crate) fn split_section(
    doc: &mut Document,
    id: NodeId,
    offset_in_body: usize,
    new_title: &str,
    new_level: HeadingLevel,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    let body = node.body_range;
    let full = node.full_range;

    // offset_in_body is relative to body_range.start.
    // Valid range: 0..=body.len() places the heading within the section body.
    // full.end - body.start is also valid: it inserts the heading after all
    // existing children (at full_range.end), which is the correct position
    // for appending a new child at the bottom.
    let max_offset = full.end - body.start;
    if offset_in_body > max_offset {
        return Err(StructuralEditError::InvalidSplitOffset);
    }
    let insert_pos = body.start + offset_in_body;
    if !doc.source().is_char_boundary(insert_pos) {
        return Err(StructuralEditError::InvalidSplitOffset);
    }

    let marker = "#".repeat(new_level.as_u8() as usize);
    let heading_text = format!("\n{marker} {new_title}\n\n");
    let insertion = ByteRange::empty_at(insert_pos);
    let result = doc.apply_replacement(insertion, &heading_text)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text: String::new(),
        new_range: result.new_range,
        new_text: heading_text,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

// ── Merge with previous sibling (RFC-025) ─────────────────────────────────────

/// Removes the heading marker of `id`, preserving its title as plain leading
/// text and merging its body into the preceding sibling's body. Both sections
/// must share the same parent.
pub(crate) fn merge_with_prev_sibling(
    doc: &mut Document,
    id: NodeId,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;
    let node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if node.is_root() {
        return Err(StructuralEditError::NoAdjacentSibling);
    }
    // Find the previous sibling.
    let parent = doc
        .outline()
        .node(node.parent_id.unwrap())
        .ok_or(StructuralEditError::StaleNode(id))?;
    let siblings = &parent.children;
    let pos = siblings
        .iter()
        .position(|&c| c == id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if pos == 0 {
        return Err(StructuralEditError::NoAdjacentSibling);
    }
    let prev = doc
        .outline()
        .node(siblings[pos - 1])
        .ok_or(StructuralEditError::StaleNode(id))?;
    if !prev.children.is_empty() {
        return Err(StructuralEditError::UnsafePreservation);
    }
    // Replace the heading element with its parsed plain title so merging does
    // not silently drop user-authored heading text.
    let heading_range = node.heading_range;
    let replacement =
        plain_heading_replacement(&doc.source()[heading_range.as_range()], &node.title);
    let old_text = doc.source()[heading_range.as_range()].to_string();
    let result = doc.apply_replacement(heading_range, &replacement)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text,
        new_range: result.new_range,
        new_text: replacement,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}

fn plain_heading_replacement(heading_source: &str, title: &str) -> String {
    if title.is_empty() {
        return String::new();
    }
    let newline = if heading_source.ends_with("\r\n") {
        "\r\n"
    } else if heading_source.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    format!("{title}{newline}")
}

// ── Move section (RFC-024) ────────────────────────────────────────────────────

/// Moves the full section subtree of `id` to `target`.
/// The exact source bytes are preserved; no blank-line normalization occurs.
pub(crate) fn move_section(
    doc: &mut Document,
    id: NodeId,
    target: MoveTarget,
    base_rev: DocumentRevision,
) -> Result<EditResult, StructuralEditError> {
    check_revision(doc, base_rev)?;

    let src_node = doc
        .outline()
        .node(id)
        .ok_or(StructuralEditError::StaleNode(id))?;
    if src_node.is_root() {
        return Err(StructuralEditError::CannotDeleteRoot);
    }
    let src_range = src_node.full_range;

    // Resolve insertion point and validate.
    let target_id = match target {
        MoveTarget::Before(t)
        | MoveTarget::After(t)
        | MoveTarget::AsFirstChildOf(t)
        | MoveTarget::AsLastChildOf(t) => t,
    };
    if target_id == id {
        return Err(StructuralEditError::CannotMoveSelf);
    }
    let target_node = doc
        .outline()
        .node(target_id)
        .ok_or(StructuralEditError::StaleTarget(target_id))?;

    // Prevent moving into a descendant.
    if is_descendant(doc.outline(), id, target_id) {
        return Err(StructuralEditError::CannotMoveIntoDescendant);
    }

    let insert_pos = match target {
        MoveTarget::Before(_) => target_node.heading_range.start,
        MoveTarget::After(_) => target_node.full_range.end,
        MoveTarget::AsFirstChildOf(_) => target_node.heading_range.end,
        MoveTarget::AsLastChildOf(_) => target_node.full_range.end,
    };

    // Build the new source string without any offset arithmetic.
    let source = doc.source();
    let moved = source[src_range.as_range()].to_string();
    let new_source = if insert_pos <= src_range.start {
        // Inserting before the source range.
        let mut s = String::with_capacity(source.len());
        s.push_str(&source[..insert_pos]);
        s.push_str(&moved);
        s.push_str(&source[insert_pos..src_range.start]);
        s.push_str(&source[src_range.end..]);
        s
    } else if insert_pos >= src_range.end {
        // Inserting after the source range.
        let mut s = String::with_capacity(source.len());
        s.push_str(&source[..src_range.start]);
        s.push_str(&source[src_range.end..insert_pos]);
        s.push_str(&moved);
        s.push_str(&source[insert_pos..]);
        s
    } else {
        // Target insertion point is inside the source range — shouldn't reach
        // here after descendant check, but guard it.
        return Err(StructuralEditError::CannotMoveIntoDescendant);
    };

    // Apply as a full-source replacement so the history captures the exact
    // before/after and undo is byte-exact (RFC-044).
    let full_range = ByteRange {
        start: 0,
        end: source.len(),
    };
    let old_source = source.to_string();
    let result = doc.apply_replacement(full_range, &new_source)?;
    doc.record_history(EditRecord {
        replaced_range: result.replaced_range,
        old_text: old_source,
        new_range: result.new_range,
        new_text: new_source,
        revision_before: result.old_revision,
        revision_after: result.new_revision,
    });
    Ok(result)
}
