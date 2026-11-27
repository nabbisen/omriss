//! Document search over each node's own text (RFC-021, RFC-085 §3.3 item 5).
//!
//! Search is purely read-only: it never mutates the document. It reads
//! [`NodeText`]s, which come from the document's `DocumentStructure`, so it
//! works the same for every format: a Markdown section's body, a JSON value.
//! Each result names the node it was found in, so the UI can show the node's
//! path and open it. Titles are RFC-077's: one more kind of result, found on the
//! same walk.

use omriss_core::{ByteRange, NodeId};

use crate::editor::node_text::{NodeText, TitleKind};
use crate::interface::node_item::NodeItem;

/// What a search result is a match of (RFC-077 §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMatchKind {
    /// Text inside a node: a Markdown section's body, a JSON value.
    Text,
    /// A Markdown heading's title.
    Heading,
    /// An object member's key.
    Key,
}

/// One match of a search query inside the canonical source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    /// What matched: the node's text, or its title (RFC-077 §4.2). A title match
    /// has no offset inside any text: it opens its node, and `range` is only
    /// where the node starts.
    pub kind: SearchMatchKind,
    /// Byte range in the full canonical source text (RFC-002 ByteRange): the
    /// match itself in a Markdown body; the whole value for a JSON value, whose
    /// text is shown decoded and so has no byte offsets of its own.
    pub range: ByteRange,
    /// The node whose own text contains this match.
    pub containing_node: NodeId,
    /// Breadcrumb path from root to `containing_node`.
    pub path: Vec<NodeItem>,
    /// Short text snippet around the match for display.
    pub preview: String,
}

// ── UTF-8–safe case-insensitive substring search ─────────────────────────────

/// Returns byte offsets of all case-insensitive occurrences of `needle` in
/// `haystack`.  The offsets are valid UTF-8 character boundaries in `haystack`.
fn find_all(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return vec![];
    }
    let needle_lower: Vec<char> = needle.chars().flat_map(|c| c.to_lowercase()).collect();
    let hay_chars: Vec<(usize, char)> = haystack.char_indices().collect();
    let n = needle_lower.len();
    let mut results = Vec::new();

    'outer: for i in 0..hay_chars.len().saturating_sub(n.saturating_sub(1)) {
        let mut ni = 0;
        let mut hi = i;
        loop {
            if ni == n {
                // Full match: record byte start/end.
                let start = hay_chars[i].0;
                let end = if i + n < hay_chars.len() {
                    hay_chars[i + n].0
                } else {
                    haystack.len()
                };
                results.push((start, end));
                continue 'outer;
            }
            if hi >= hay_chars.len() {
                continue 'outer;
            }
            let hay_lower: Vec<char> = hay_chars[hi].1.to_lowercase().collect();
            if hay_lower.is_empty() || hay_lower[0] != needle_lower[ni] {
                continue 'outer;
            }
            hi += 1;
            ni += 1;
        }
    }
    results
}

/// Extracts a short preview snippet (≤80 chars) around a match.
fn make_preview(body: &str, match_start: usize) -> String {
    const CONTEXT: usize = 40;
    let char_indices: Vec<(usize, char)> = body.char_indices().collect();
    // Find the char index of match_start.
    let ci = char_indices.partition_point(|&(b, _)| b < match_start);
    let start_ci = ci.saturating_sub(CONTEXT);
    let end_ci = (ci + CONTEXT).min(char_indices.len());
    let snippet: String = char_indices[start_ci..end_ci]
        .iter()
        .map(|&(_, c)| c)
        .collect();
    let prefix = if start_ci > 0 { "…" } else { "" };
    let suffix = if end_ci < char_indices.len() {
        "…"
    } else {
        ""
    };
    format!("{prefix}{}{suffix}", snippet.replace('\n', " "))
}

// ── public API ────────────────────────────────────────────────────────────────

/// One node as the search walk sees it (RFC-077 §8): its searchable title, if
/// any, and its own text, if any. `where` the node starts is what a title match
/// reports, since a title is not at an offset inside a text.
pub struct SearchEntry<'a> {
    pub id: NodeId,
    pub title: Option<(&'a str, TitleKind)>,
    pub text: Option<&'a NodeText>,
    /// Where the node starts in the source.
    pub start: usize,
}

/// Searches `entries` (in document order) for `query`. Results are in that order,
/// and within a node its title result comes before its text results (RFC-077
/// §4.3: two results, not a merged one). One matching rule for both (§8.5): the
/// same [`find_all`]. Empty query returns no results. `path_of` gives a node's
/// path from the root.
pub fn search_entries<'a>(
    entries: impl IntoIterator<Item = SearchEntry<'a>>,
    query: &str,
    path_of: impl Fn(NodeId) -> Vec<NodeItem>,
) -> Vec<SearchMatch> {
    if query.trim().is_empty() {
        return vec![];
    }
    let mut results = Vec::new();
    for entry in entries {
        let title_hits = entry
            .title
            .map(|(title, kind)| (title, kind, find_all(title, query)))
            .filter(|(_, _, hits)| !hits.is_empty());
        let text_hits = entry
            .text
            .map(|node| (node, find_all(&node.text, query)))
            .filter(|(_, hits)| !hits.is_empty());
        if title_hits.is_none() && text_hits.is_none() {
            continue;
        }
        let path = path_of(entry.id);
        if let Some((title, kind, hits)) = title_hits {
            // One result per matching title, however many times the query occurs in it:
            // a title is opened, not sought into.
            let first = hits[0].0;
            results.push(SearchMatch {
                kind: match kind {
                    TitleKind::Heading => SearchMatchKind::Heading,
                    TitleKind::Key => SearchMatchKind::Key,
                },
                range: ByteRange::empty_at(entry.start),
                containing_node: entry.id,
                path: path.clone(),
                preview: make_preview(title, first),
            });
        }
        if let Some((node, hits)) = text_hits {
            for (start, end) in hits {
                let range = if node.exact {
                    ByteRange::new(node.range.start + start, node.range.start + end)
                        .unwrap_or(ByteRange::empty_at(node.range.start + start))
                } else {
                    node.range
                };
                results.push(SearchMatch {
                    kind: SearchMatchKind::Text,
                    range,
                    containing_node: node.id,
                    path: path.clone(),
                    preview: make_preview(&node.text, start),
                });
            }
        }
    }
    results
}

/// Searches `texts` (in document order) for `query`, bodies only: the text search
/// of RFC-021, unchanged. A title is searched through [`search_entries`].
pub fn search_texts<'a>(
    texts: impl IntoIterator<Item = &'a NodeText>,
    query: &str,
    path_of: impl Fn(NodeId) -> Vec<NodeItem>,
) -> Vec<SearchMatch> {
    search_entries(
        texts.into_iter().map(|text| SearchEntry {
            id: text.id,
            title: None,
            text: Some(text),
            start: text.range.start,
        }),
        query,
        path_of,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::node_text::{node_texts, preorder, subtree};
    use omriss_core::{Document, DocumentFormatAdapter, MarkdownAdapter};

    fn doc(md: &str) -> Document {
        Document::parse(md.to_string()).unwrap()
    }

    /// The search a Markdown session runs, over `scope`'s subtree (or all of it).
    fn run(d: &Document, scope: Option<NodeId>, query: &str) -> Vec<SearchMatch> {
        let structure = MarkdownAdapter
            .build_structure(d.source(), d.revision())
            .unwrap();
        let index = structure
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id, i))
            .collect();
        let ids = match scope {
            Some(id) => subtree(&structure, &index, id),
            None => preorder(&structure, &index),
        };
        let texts = node_texts(d.source(), &structure, &index, &ids);
        search_texts(&texts, query, |_| Vec::new())
    }

    fn search_document(d: &Document, query: &str) -> Vec<SearchMatch> {
        run(d, None, query)
    }

    fn search_section(d: &Document, scope: NodeId, query: &str) -> Vec<SearchMatch> {
        run(d, Some(scope), query)
    }

    #[test]
    fn finds_whole_document_match() {
        let d = doc("# A\nthe quick brown fox\n\n# B\nfox jumps\n");
        let results = search_document(&d, "fox");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn case_insensitive() {
        let d = doc("# A\nFOXTROT\n");
        assert!(!search_document(&d, "fox").is_empty());
    }

    #[test]
    fn section_scope_is_contained() {
        let d = doc("# A\nfoo\n\n# B\nbar foo\n");
        let a_id = d.outline().root().children[0];
        let results = search_section(&d, a_id, "foo");
        assert!(results.iter().all(|r| r.containing_node == a_id));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn empty_query_returns_nothing() {
        let d = doc("# A\nbody\n");
        assert!(search_document(&d, "").is_empty());
        assert!(search_document(&d, "  ").is_empty());
    }

    #[test]
    fn utf8_match_ranges_valid() {
        let d = doc("# 日本語\n東京は日本の首都\n");
        let results = search_document(&d, "日本");
        for r in &results {
            let source = d.source();
            assert!(source.is_char_boundary(r.range.start));
            assert!(source.is_char_boundary(r.range.end));
        }
    }
}
