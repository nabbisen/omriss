//! Document search over the canonical Markdown source (RFC-021).
//!
//! Search is purely read-only: it never mutates the document. Results are
//! grouped by section node so the UI can show the containing path and the
//! user can navigate directly to the matching section.

use omriss_core::{ByteRange, Document, NodeId, OutlineItem};

/// One match of a search query inside the canonical source text.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Byte range in the full canonical source text (RFC-002 ByteRange).
    pub range: ByteRange,
    /// The section whose body range contains this match.
    pub containing_node: NodeId,
    /// Breadcrumb path from root to `containing_node`.
    pub path: Vec<OutlineItem>,
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

/// Searches the body of every section in `doc` for `query`.
/// Results are in source order. Empty query returns no results.
pub fn search_document(doc: &Document, query: &str) -> Vec<SearchMatch> {
    if query.trim().is_empty() {
        return vec![];
    }
    let source = doc.source();
    let outline = doc.outline();
    let mut results = Vec::new();

    for node in outline.iter() {
        let body = match source.get(node.body_range.as_range()) {
            Some(s) => s,
            None => continue,
        };
        for (start_in_body, end_in_body) in find_all(body, query) {
            let abs_start = node.body_range.start + start_in_body;
            let abs_end = node.body_range.start + end_in_body;
            let preview = make_preview(body, start_in_body);
            let path = outline
                .path(node.id)
                .unwrap_or_default()
                .iter()
                .map(|n| OutlineItem {
                    id: n.id,
                    title: n.title.clone(),
                    level: n.level,
                    child_count: n.children.len(),
                })
                .collect();
            results.push(SearchMatch {
                range: ByteRange::new(abs_start, abs_end).unwrap_or(ByteRange::empty_at(abs_start)),
                containing_node: node.id,
                path,
                preview,
            });
        }
    }
    results
}

/// Searches only the body of the node identified by `scope_node`.
/// Falls back to whole-document search when `scope_node` does not exist.
pub fn search_section(doc: &Document, scope_node: NodeId, query: &str) -> Vec<SearchMatch> {
    if query.trim().is_empty() {
        return vec![];
    }
    let source = doc.source();
    let outline = doc.outline();
    let Some(node) = outline.node(scope_node) else {
        return search_document(doc, query);
    };
    let body = match source.get(node.body_range.as_range()) {
        Some(s) => s,
        None => return vec![],
    };
    let path: Vec<OutlineItem> = outline
        .path(node.id)
        .unwrap_or_default()
        .iter()
        .map(|n| OutlineItem {
            id: n.id,
            title: n.title.clone(),
            level: n.level,
            child_count: n.children.len(),
        })
        .collect();

    find_all(body, query)
        .into_iter()
        .map(|(start_in_body, end_in_body)| {
            let abs_start = node.body_range.start + start_in_body;
            let abs_end = node.body_range.start + end_in_body;
            SearchMatch {
                range: ByteRange::new(abs_start, abs_end).unwrap_or(ByteRange::empty_at(abs_start)),
                containing_node: node.id,
                path: path.clone(),
                preview: make_preview(body, start_in_body),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omriss_core::Document;

    fn doc(md: &str) -> Document {
        Document::parse(md.to_string()).unwrap()
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
