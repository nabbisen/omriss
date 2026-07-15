//! Document statistics for the status bar (RFC-046).
//!
//! Word counting uses `str::split_whitespace` which gives correct results for
//! ASCII text and reasonable approximations for CJK-heavy documents where
//! space-based word segmentation is conventional enough for a writing aid.

use omriss_core::{Document, NodeId, Outline};

/// Lightweight statistics for the status bar display.
#[derive(Debug, Clone, Copy, Default)]
pub struct DocumentStats {
    /// Total word count across the whole canonical source.
    pub total_words: usize,
    /// Word count of the currently focused section body (0 in overview mode).
    pub focused_words: usize,
    /// Number of named sections in the outline (root excluded).
    pub section_count: usize,
}

/// Counts the whitespace-delimited words in `text`.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Computes statistics for `doc`, optionally scoped to `focus_id`.
pub fn compute_stats(doc: &Document, focus_id: Option<NodeId>) -> DocumentStats {
    let section_count = non_root_section_count(doc.outline());
    let total_words = word_count(doc.source());
    let focused_words = focus_id
        .and_then(|id| doc.outline().node(id))
        .and_then(|n| doc.source().get(n.body_range.as_range()))
        .map(word_count)
        .unwrap_or(0);

    DocumentStats {
        total_words,
        focused_words,
        section_count,
    }
}

fn non_root_section_count(outline: &Outline) -> usize {
    // `iter()` yields all nodes; skip the root (has no level).
    outline.iter().filter(|n| n.level.is_some()).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omriss_core::Document;

    fn doc(md: &str) -> Document {
        Document::parse(md.to_string()).unwrap()
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
