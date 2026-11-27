//! Document statistics for the status bar (RFC-046).
//!
//! Word counting uses `str::split_whitespace` which gives correct results for
//! ASCII text and reasonable approximations for CJK-heavy documents where
//! space-based word segmentation is conventional enough for a writing aid.

use std::collections::HashSet;

use omriss_core::{DocumentFormat, DocumentStructure, NodeId, StructureNodeKind};

use crate::editor::node_text::NodeText;

/// What `section_count` counts: the format's own noun (RFC-085 §3.3 item 6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StatsUnit {
    /// A Markdown document's sections.
    #[default]
    Sections,
    /// A JSON document's values (its leaves; groups and lists are not counted).
    Values,
}

/// Lightweight statistics for the status bar display.
#[derive(Debug, Clone, Copy, Default)]
pub struct DocumentStats {
    /// Total word count across the document's text.
    pub total_words: usize,
    /// Word count of the focused node's text (0 in overview mode).
    pub focused_words: usize,
    /// Number of `unit`s in the document (the root excluded).
    pub section_count: usize,
    pub unit: StatsUnit,
}

/// Counts the whitespace-delimited words in `text`.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Statistics over a document's `structure`.
///
/// `texts` is every node's own text (`node_text::node_texts` over the whole
/// document); `focus_ids` is the focused node and what is under it. A Markdown
/// document counts the words of its whole source and, for the focused section,
/// of its own body — as it always has. Any other format counts the words of the
/// node texts, since its source is mostly syntax.
pub fn compute_stats(
    source: &str,
    structure: &DocumentStructure,
    texts: &[NodeText],
    focus: Option<(NodeId, &[NodeId])>,
) -> DocumentStats {
    let markdown = structure.format == DocumentFormat::Markdown;
    let unit = if markdown {
        StatsUnit::Sections
    } else {
        StatsUnit::Values
    };
    let section_count = structure
        .nodes
        .iter()
        .filter(|n| match unit {
            StatsUnit::Sections => n.kind == StructureNodeKind::MarkdownSection,
            StatsUnit::Values => n.kind == StructureNodeKind::Value,
        })
        .count();
    let (total_words, focused_words) = if markdown {
        let own = focus.and_then(|(id, _)| texts.iter().find(|t| t.id == id));
        (
            word_count(source),
            own.map(|t| word_count(&t.text)).unwrap_or(0),
        )
    } else {
        let total = texts.iter().map(|t| word_count(&t.text)).sum();
        let focused = focus
            .map(|(_, ids)| {
                let ids: HashSet<NodeId> = ids.iter().copied().collect();
                texts
                    .iter()
                    .filter(|t| ids.contains(&t.id))
                    .map(|t| word_count(&t.text))
                    .sum()
            })
            .unwrap_or(0);
        (total, focused)
    };
    DocumentStats {
        total_words,
        focused_words,
        section_count,
        unit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::node_text::{node_texts, preorder, subtree};
    use omriss_core::{Document, DocumentFormatAdapter, MarkdownAdapter};

    fn doc(md: &str) -> Document {
        Document::parse(md.to_string()).unwrap()
    }

    /// What a Markdown session computes.
    fn compute_stats(d: &Document, focus: Option<NodeId>) -> DocumentStats {
        let structure = MarkdownAdapter
            .build_structure(d.source(), d.revision())
            .unwrap();
        let index = structure
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id, i))
            .collect();
        let texts = node_texts(
            d.source(),
            &structure,
            &index,
            &preorder(&structure, &index),
        );
        let ids = focus.map(|id| subtree(&structure, &index, id));
        super::compute_stats(d.source(), &structure, &texts, focus.zip(ids.as_deref()))
    }

    #[test]
    fn empty_text_count_zero() {
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn counts_words_correctly() {
        assert_eq!(word_count("hello world foo"), 3);
    }

    #[test]
    fn leading_trailing_whitespace_ignored() {
        assert_eq!(word_count("  hello   world  "), 2);
    }

    #[test]
    fn section_count_excludes_root() {
        let d = doc("# A\n\n## A.1\n\n# B\n");
        let stats = compute_stats(&d, None);
        assert_eq!(stats.section_count, 3);
    }

    #[test]
    fn focused_words_scoped_to_body() {
        let d = doc("# A\none two three\n\n# B\nfive words here two more\n");
        let b_id = *d.outline().root().children.last().unwrap();
        let stats = compute_stats(&d, Some(b_id));
        assert_eq!(stats.focused_words, 5);
    }

    #[test]
    fn stats_update_reflects_committed_edit() {
        let mut d = doc("# A\none two\n");
        let a_id = d.outline().root().children[0];
        let before = compute_stats(&d, Some(a_id));
        d.replace_section_body(omriss_core::ReplaceSectionBody {
            node_id: a_id,
            base_revision: d.revision(),
            new_body: "one two three four\n".into(),
        })
        .unwrap();
        let after = compute_stats(&d, Some(a_id));
        assert!(after.focused_words > before.focused_words);
    }
}
