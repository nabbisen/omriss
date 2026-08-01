//! Focused content model (RFC-053 §8).
//!
//! What the right-hand editor pane renders for one selected node. Adapters
//! translate their own value/section vocabulary into this format-neutral
//! shape; `omriss-ui` translates it further into plain labels (RFC-052
//! §8.4).

use crate::formats::error::StructureErrorKind;

/// The right-hand editor's view of one selected node (RFC-053 §8).
#[derive(Debug, Clone, PartialEq)]
pub enum FocusedContent {
    /// A Markdown section: the writer edits `body` directly.
    MarkdownSection {
        title: String,
        body: String,
        preview_available: bool,
    },
    /// A single structured value (JSON/TOML scalar).
    StructuredValue {
        title: String,
        value_kind: ValueKind,
        display_text: String,
        editable_text: String,
    },
    /// A group/list too large or too structural to edit as raw text.
    StructuredGroup {
        title: String,
        child_count: usize,
        summary: String,
        raw_text_available: bool,
    },
    /// A node this format's adapter cannot safely represent.
    ///
    /// `reason` reuses [`StructureErrorKind`] rather than a dedicated
    /// "friendly reason" type RFC-053 §8 does not define — the two ask the
    /// same question ("why can't this be shown/edited") at the same level
    /// of granularity as the §11 error model.
    Unsupported {
        title: String,
        reason: StructureErrorKind,
        raw_text_available: bool,
    },
}

/// The kind of a [`FocusedContent::StructuredValue`] (RFC-053 §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Text,
    Number,
    OnOff,
    /// JSON `null` / an absent value — user-facing "No value", distinct
    /// from an empty string.
    NoValue,
    RawText,
}
