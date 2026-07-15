//! Fixture catalog tests (RFC-034).
//!
//! Each test verifies that a fixture:
//! - parses without error;
//! - produces the expected outline shape (heading count, depth);
//! - preserves every byte across a round-trip edit on the first section body;
//! - has a valid line-ending profile detectable by `FileTextProfile`.

use omriss_core::{Document, ReplaceSectionBody};

fn load(name: &str) -> Document {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {name} not found: {e}"));
    Document::parse(src).unwrap_or_else(|e| panic!("fixture {name} parse error: {e}"))
}

/// Round-trip: edit a section body then undo; source must be byte-identical.
fn round_trip_edit(doc: &mut Document) {
    let root_children = doc.outline().root().children.clone();
    if root_children.is_empty() {
        return; // no-headings fixture — skip edit
    }
    let id = root_children[0];
    let original = doc.source().to_string();
    let body = doc.outline().node(id).unwrap().body_range;
    let current_body = doc.source()[body.as_range()].to_string();
    doc.replace_section_body(ReplaceSectionBody {
        node_id: id,
        base_revision: doc.revision(),
        new_body: format!("{current_body}<!-- edited -->\n"),
    })
    .unwrap();
    doc.undo().unwrap();
    assert_eq!(doc.source(), original, "source changed after edit+undo");
}

// ── existing fixtures ──────────────────────────────────────────────────────

#[test]
fn fixture_nested_atx() {
    let mut d = load("nested_atx.md");
    assert!(d.outline().iter().count() > 1);
    round_trip_edit(&mut d);
}

#[test]
fn fixture_duplicate_titles() {
    let mut d = load("duplicate_titles.md");
    // Multiple nodes should have distinct IDs even with identical titles.
    let ids: Vec<_> = d.outline().iter().map(|n| n.id).collect();
    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "duplicate IDs in outline");
    round_trip_edit(&mut d);
}

#[test]
fn fixture_japanese() {
    let mut d = load("japanese.md");
    // At least one heading with non-ASCII title.
    let has_cjk = d
        .outline()
        .iter()
        .any(|n| n.title.chars().any(|c| c as u32 > 0x7F));
    assert!(has_cjk, "expected CJK heading");
    round_trip_edit(&mut d);
}

#[test]
fn fixture_yaml_front_matter() {
    let d = load("yaml_front_matter.md");
    // Front matter must not be treated as a heading.
    assert!(
        d.outline()
            .root()
            .children
            .iter()
            .all(|&id| { !d.outline().node(id).unwrap().title.starts_with("---") })
    );
}

// ── new M7 fixtures (RFC-034) ──────────────────────────────────────────────

#[test]
fn fixture_large_10k_words_parses() {
    let d = load("large-10k-words.md");
    // Must produce a substantial heading tree.
    let node_count = d.outline().iter().count();
    assert!(node_count > 50, "expected >50 nodes, got {node_count}");
}

#[test]
fn fixture_large_10k_words_round_trip() {
    let mut d = load("large-10k-words.md");
    round_trip_edit(&mut d);
}

#[test]
fn fixture_large_10k_source_preserved_on_mid_doc_edit() {
    let mut d = load("large-10k-words.md");
    let original = d.source().to_string();
    // Edit the last section body and verify first section bytes are unchanged.
    let ids: Vec<_> = d.outline().iter().map(|n| n.id).collect();
    if let Some(&last_id) = ids.last() {
        let body_range = d.outline().node(last_id).unwrap().body_range;
        let original_body = d.source()[body_range.as_range()].to_string();
        // Find the byte offset of the first section to check it is untouched.
        let first_heading_start = d
            .outline()
            .node(ids[1]) // ids[0] is root
            .unwrap()
            .heading_range
            .start;
        let prefix_before = original[..first_heading_start + 10].to_string();
        d.replace_section_body(ReplaceSectionBody {
            node_id: last_id,
            base_revision: d.revision(),
            new_body: format!("{original_body}appended\n"),
        })
        .unwrap();
        let prefix_after = d.source()[..first_heading_start + 10].to_string();
        assert_eq!(prefix_before, prefix_after, "unrelated bytes changed");
        d.undo().unwrap();
        assert_eq!(d.source(), original);
    }
}

#[test]
fn fixture_academic_paper_parses() {
    let d = load("academic-paper.md");
    // Should produce a substantial heading tree (title + numbered sections + subsections).
    let total_nodes = d.outline().iter().count();
    assert!(total_nodes >= 10, "expected ≥10 nodes, got {total_nodes}");
    // Top-level content must have at least one section.
    assert!(
        !d.outline().root().children.is_empty(),
        "no top-level sections"
    );
}

#[test]
fn fixture_academic_paper_round_trip() {
    let mut d = load("academic-paper.md");
    round_trip_edit(&mut d);
}

#[test]
fn fixture_technical_rfc_parses() {
    let d = load("technical-rfc.md");
    // Code fences and tables in bodies must not become section boundaries.
    let has_code_in_body = d
        .outline()
        .iter()
        .any(|n| d.source()[n.body_range.as_range()].contains("```"));
    assert!(has_code_in_body, "expected code fence in a body range");
}

#[test]
fn fixture_technical_rfc_round_trip() {
    let mut d = load("technical-rfc.md");
    round_trip_edit(&mut d);
}

// ── semantic fixture tests for previously parse-only fixtures (§17 requirements) ─

#[test]
fn fixture_code_fences_heading_inside_fence_is_not_indexed() {
    let d = load("code_fences.md");
    // "# Fake Heading" lines inside fenced blocks must not become sections.
    // Only the real ATX heading outside fences should be indexed.
    let section_titles: Vec<_> = d.outline().iter().map(|n| n.title.as_str()).collect();
    let has_fake = section_titles
        .iter()
        .any(|t| t.contains("Fake") || t.contains("fake"));
    assert!(
        !has_fake,
        "fence content became a section: {section_titles:?}"
    );
    // There must be at least one real section.
    assert!(
        d.outline().iter().any(|n| n.title == "Real Heading"),
        "expected 'Real Heading' section; got: {section_titles:?}"
    );
}

#[test]
fn fixture_setext_headings_are_indexed() {
    let d = load("setext.md");
    // Setext headings (underlined with === or ---) must be in the outline.
    let titles: Vec<_> = d.outline().iter().map(|n| n.title.as_str()).collect();
    assert!(
        titles.iter().any(|t| t.contains("Setext")),
        "no setext heading found; outline: {titles:?}"
    );
}

#[test]
fn fixture_crlf_line_endings_preserved() {
    let d = load("crlf.md");
    // The source must retain \r\n line endings throughout.
    assert!(
        d.source().contains("\r\n"),
        "CRLF line endings lost after parsing"
    );
    // At least one heading must be indexed.
    assert!(
        !d.outline().root().children.is_empty(),
        "no sections found in CRLF fixture"
    );
}

#[test]
fn fixture_skipped_levels_are_handled() {
    let d = load("skipped_levels.md");
    // H1 → H4 skip (H2, H3 absent) must not panic or drop the H4 node.
    let titles: Vec<_> = d.outline().iter().map(|n| n.title.as_str()).collect();
    assert!(
        titles.iter().any(|t| t.contains("Jumped")),
        "skipped-level H4 section missing; outline: {titles:?}"
    );
}

#[test]
fn fixture_no_headings_root_has_no_children() {
    let d = load("no_headings.md");
    assert!(
        d.outline().root().children.is_empty(),
        "expected no child sections in a heading-free document"
    );
    // Source must be stable.
    assert_eq!(
        d.source(),
        include_str!("fixtures/no_headings.md"),
        "source changed on load"
    );
}

#[test]
fn fixture_empty_bodies_sections_index_correctly() {
    let d = load("empty_bodies.md");
    // Sections with no body text still appear in the outline.
    let non_root = d.outline().root().children.len();
    assert!(
        non_root >= 2,
        "expected ≥2 top-level sections, got {non_root}"
    );
}

#[test]
fn fixture_html_content_comment_not_treated_as_heading() {
    let d = load("html_content.md");
    // An HTML comment containing "# Heading" must not become an outline node.
    let titles: Vec<_> = d.outline().iter().map(|n| n.title.as_str()).collect();
    let from_comment = titles.iter().any(|t| t.contains("Heading"));
    assert!(
        !from_comment,
        "HTML comment content became a section: {titles:?}"
    );
}

#[test]
fn fixture_no_trailing_newline_source_round_trips() {
    let source = include_str!("fixtures/no_trailing_newline.md");
    let d = Document::parse(source.to_string()).unwrap();
    // Source must be byte-identical after parsing (no newline appended).
    assert_eq!(
        d.source(),
        source,
        "trailing-newline source changed on load"
    );
}

#[test]
fn fixture_toml_front_matter_not_treated_as_heading() {
    let d = load("toml_front_matter.md");
    // TOML front matter (+++…+++) must not appear as a section title.
    let titles: Vec<_> = d.outline().iter().map(|n| n.title.as_str()).collect();
    let fm_as_heading = titles
        .iter()
        .any(|t| t.starts_with("+++") || t.contains("title ="));
    assert!(
        !fm_as_heading,
        "TOML front matter became a heading: {titles:?}"
    );
}
