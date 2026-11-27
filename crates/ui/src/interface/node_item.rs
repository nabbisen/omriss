//! One node, as the panes list it (RFC-085): a card in the overview, a crumb
//! in the breadcrumb, a step in a search result's path.
//!
//! Format-neutral and read from the document's `DocumentStructure`, whatever
//! the format. The fields `id`, `title`, `level` and `child_count` are the ones
//! the Markdown-only `OutlineItem` had, so Markdown callers read the same names.

use omriss_core::{HeadingLevel, NodeId, StructureNodeKind, ValueKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeItem {
    pub id: NodeId,
    /// Heading text, JSON key or index; empty for the document's root.
    pub title: String,
    /// `None` for the root and for every non-Markdown node.
    pub level: Option<HeadingLevel>,
    /// Direct members: subsections for Markdown, entries for a JSON group or list.
    pub child_count: usize,
    pub kind: StructureNodeKind,
    /// The title is a list member's number, made up by omriss; say it with
    /// [`display_title`](crate::display_title), never raw. Such a title is not
    /// searchable (RFC-077 §8.1).
    pub positional: bool,
    /// The document's own root: the whole-document view, not a node under it. (A
    /// JSON document's root is its top-level value, so this is not `kind`.)
    pub is_root: bool,
    /// What a JSON value is (text, number, on/off, empty); `None` for anything else.
    pub value_kind: Option<ValueKind>,
}

impl NodeItem {
    /// The title as a person reads it, in `lang` (RFC-073 §8.6).
    pub fn display_title(&self, lang: crate::i18n::Locale) -> String {
        crate::interface::title::display_title(lang, &self.title, self.positional)
    }
}
