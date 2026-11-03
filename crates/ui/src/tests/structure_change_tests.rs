//! RFC-071: the structure-change warning fires on an *unintended* outline
//! change and is silent otherwise.
//!
//! The silent cases are tested as carefully as the loud ones. A warning that
//! fires on an ordinary delete is the failure that makes users stop reading
//! warnings, which costs the one case the feature exists for.

use omriss_core::{DocumentFormat, MoveTarget, NodeId};

use crate::editor::structure_change::{
    ExpectedChange, Operation, OutlineSummary, StructureChange, StructureChangeKind, detect,
    expected_change,
};
use crate::file::file_profile::FileTextProfile;
use crate::interface::document_map::{DocumentMapNode, node_id_from_raw};
use crate::session::EditorSession;

fn open(md: &str) -> EditorSession {
    EditorSession::open(md.to_string(), None).unwrap()
}

fn find_in(node: &DocumentMapNode, title: &str) -> Option<NodeId> {
    if node.title == title {
        return Some(node_id_from_raw(node.id));
    }
    node.children.iter().find_map(|c| find_in(c, title))
}

fn id_of(session: &EditorSession, title: &str) -> NodeId {
    find_in(&session.document_map_nodes(), title)
        .unwrap_or_else(|| panic!("no section titled {title:?}"))
}

/// Focuses `title` and commits `body` as its body.
fn commit(session: &mut EditorSession, title: &str, body: &str) {
    let id = id_of(session, title);
    let snapshot = session.focus(id).unwrap();
    session
        .commit_focused_body(&snapshot, body.to_string())
        .unwrap();
}

fn change(lost: usize, gained: usize) -> Option<StructureChange> {
    Some(StructureChange { lost, gained })
}

const TWO: &str = "# One\nbody\n\n# Two\nmore\n";

// ── Body commits ────────────────────────────────────────────────────────────

#[test]
fn a_comment_opener_above_a_heading_warns_that_headings_disappeared() {
    let mut s = open(TWO);
    commit(&mut s, "One", "<!--");
    assert_eq!(s.structure_change(), change(1, 0));
    assert_eq!(
        s.structure_change().unwrap().kind(),
        StructureChangeKind::Disappeared
    );
}

#[test]
fn typing_a_heading_warns_that_a_section_appeared() {
    let mut s = open(TWO);
    commit(&mut s, "One", "# Foo\n");
    assert_eq!(s.structure_change(), change(0, 1));
    assert_eq!(
        s.structure_change().unwrap().kind(),
        StructureChangeKind::Appeared
    );
}

#[test]
fn an_ordinary_body_commit_is_silent() {
    let mut s = open(TWO);
    commit(&mut s, "One", "rewritten body\n");
    assert_eq!(s.structure_change(), None);
}

#[test]
fn a_commit_that_both_hides_and_creates_headings_warns_both() {
    // `# New` appears, and the unterminated comment after it swallows `# Two`.
    let mut s = open(TWO);
    commit(&mut s, "One", "# New\n<!--");
    let c = s.structure_change().expect("structure changed");
    assert_eq!((c.lost, c.gained), (1, 1));
    assert_eq!(c.kind(), StructureChangeKind::Both);
}

#[test]
fn closing_the_comment_leaves_the_outline_intact_and_is_silent() {
    let mut s = open(TWO);
    commit(&mut s, "One", "<!-- note -->\n");
    assert_eq!(s.structure_change(), None);
}

#[test]
fn the_next_silent_edit_clears_the_warning() {
    let mut s = open(TWO);
    commit(&mut s, "One", "<!--");
    assert!(s.structure_change().is_some());
    // `# Two` is now comment text, so edit the one section that is left.
    commit(&mut s, "One", "plain\n");
    assert_eq!(s.structure_change(), None);
}

// ── Structural operations: the "intended" rows ──────────────────────────────

#[test]
fn deleting_a_three_node_subtree_is_silent_because_it_is_intended() {
    let md = "# Keep\nk\n\n# Gone\n\n## Child A\n\n## Child B\n\n# Also\n";
    let mut s = open(md);
    let id = id_of(&s, "Gone");
    s.focus(id).unwrap();
    s.delete_focused().unwrap();
    assert_eq!(
        s.structure_change(),
        None,
        "removing a section and its two children is what delete means"
    );
}

#[test]
fn a_move_that_changes_nothing_structurally_is_silent() {
    let mut s = open("# A\nalpha\n\n# B\nbeta\n\n# C\ngamma\n");
    let id = id_of(&s, "C");
    s.focus(id).unwrap();
    s.move_focused_up().unwrap();
    assert_eq!(s.structure_change(), None);
}

#[test]
fn moving_a_section_whose_body_holds_an_unterminated_html_block_warns() {
    // RFC-066 §3.1a: `<?` opens an HTML block that, once the section is moved
    // above the setext heading `A`, swallows it. Every byte survives; the
    // outline loses a section the user did not select.
    let mut s = open("##### A\nA\n===\n#### a\n<?\n");
    let a_lower = id_of(&s, "a");
    s.focus(a_lower).unwrap();
    let atx_a = {
        let root = s.document_map_nodes();
        root.children
            .first()
            .map(|n| node_id_from_raw(n.id))
            .unwrap()
    };
    s.move_focused(MoveTarget::After(atx_a)).unwrap();
    assert_eq!(
        s.structure_change(),
        change(1, 0),
        "source now {:?}",
        s.source()
    );
}

#[test]
fn splitting_in_exactly_one_node_is_silent() {
    let mut s = open("# One\nbody text here\n");
    let id = id_of(&s, "One");
    s.focus(id).unwrap();
    s.split_focused(5, "Split", omriss_core::HeadingLevel::H2)
        .unwrap();
    assert_eq!(s.structure_change(), None);
}

#[test]
fn adding_a_section_after_and_a_top_level_section_are_silent() {
    let mut s = open(TWO);
    let id = id_of(&s, "One");
    s.focus(id).unwrap();
    s.add_after_focused("Inserted").unwrap();
    assert_eq!(s.structure_change(), None);
    s.add_top_level_section("Last").unwrap();
    assert_eq!(s.structure_change(), None);
    s.focus(id).unwrap();
    s.append_child_to_focused("Kid", omriss_core::HeadingLevel::H2)
        .unwrap();
    assert_eq!(s.structure_change(), None);
}

#[test]
fn rename_is_silent() {
    let mut s = open("# One\n\n## A\n\n## B\n");
    let a = id_of(&s, "A");
    s.focus(a).unwrap();
    s.rename_focused("Alpha").unwrap();
    assert_eq!(s.structure_change(), None);
}

#[test]
fn promote_and_demote_are_silent() {
    let mut s = open("# One\n\n## A\n\n# Two\n");
    let a = id_of(&s, "A");
    s.focus(a).unwrap();
    s.promote_focused().unwrap();
    assert_eq!(s.structure_change(), None, "promote");
    let a = id_of(&s, "A");
    s.focus(a).unwrap();
    s.demote_focused().unwrap();
    assert_eq!(s.structure_change(), None, "demote");
}

#[test]
fn merge_is_silent() {
    let mut s = open("# One\n\n## A\n\n## B\n");
    let b = id_of(&s, "B");
    s.focus(b).unwrap();
    s.merge_focused_up().unwrap();
    assert_eq!(s.structure_change(), None);
}

// ── Undo, redo, failure, and other formats ──────────────────────────────────

#[test]
fn undo_restores_the_outline_and_clears_the_warning() {
    let mut s = open(TWO);
    let before = s.source().to_string();
    commit(&mut s, "One", "<!--");
    assert!(s.structure_change().is_some());
    s.undo().unwrap();
    assert_eq!(s.structure_change(), None);
    assert_eq!(s.source(), before, "undo is byte-exact");
    assert!(
        find_in(&s.document_map_nodes(), "Two").is_some(),
        "Two is a heading again"
    );
}

#[test]
fn redo_clears_the_warning_too() {
    let mut s = open(TWO);
    commit(&mut s, "One", "<!--");
    s.undo().unwrap();
    s.redo().unwrap();
    assert_eq!(s.structure_change(), None);
}

#[test]
fn a_failed_operation_leaves_the_previous_warning_alone() {
    let mut s = open(TWO);
    commit(&mut s, "One", "<!--");
    let warned = s.structure_change();
    assert!(warned.is_some());
    // A structural operation with no section focused is refused.
    s.show_outline();
    assert!(s.rename_focused("x").is_err());
    assert_eq!(
        s.structure_change(),
        warned,
        "a refused edit changed nothing"
    );
}

#[test]
fn a_json_document_is_never_checked() {
    // The Markdown outline `Document` builds over JSON text is not its
    // structure (RFC-054 §0.1); a value that looks like a heading must not warn.
    let mut s = EditorSession::open_detected(
        "{\"a\": \"x\", \"b\": 1}\n".to_string(),
        None,
        FileTextProfile::detect("", false),
        DocumentFormat::Json,
    )
    .unwrap();
    assert_eq!(s.format(), DocumentFormat::Json);
    assert_eq!(s.structure_change(), None);
    let id = find_in(&s.document_map_nodes(), "a").expect("key a");
    s.focus(id).unwrap();
    s.commit_structured_draft("# not a heading\n<!--").unwrap();
    assert_eq!(s.structure_change(), None);
}

// ── The comparator on its own ───────────────────────────────────────────────

/// The summary of a document made of the given top-level titles.
fn summary(titles: &[&str]) -> OutlineSummary {
    let md: String = titles.iter().map(|t| format!("# {t}\n")).collect();
    let doc = omriss_core::Document::parse(md).unwrap();
    OutlineSummary::of(doc.outline())
}

#[test]
fn detect_is_silent_when_the_outline_is_exactly_as_intended() {
    let before = summary(&["A", "B"]);
    assert_eq!(detect(&before, &before, &ExpectedChange::default()), None);
}

#[test]
fn detect_counts_duplicates_as_a_multiset() {
    let before = summary(&["A", "A", "B"]);
    let after = summary(&["A", "B"]);
    assert_eq!(
        detect(&before, &after, &ExpectedChange::default()),
        change(1, 0),
        "one of two identical titles vanishing is a loss"
    );
}

#[test]
fn an_intended_removal_is_not_a_loss_but_an_extra_one_is() {
    let md = "# A\n\n# B\n\n# C\n";
    let doc = omriss_core::Document::parse(md.to_string()).unwrap();
    let b = doc.outline().iter().find(|n| n.title == "B").unwrap().id;
    let expected = expected_change(Operation::Delete(b), doc.outline());
    let before = OutlineSummary::of(doc.outline());

    assert_eq!(detect(&before, &summary(&["A", "C"]), &expected), None);
    assert_eq!(detect(&before, &summary(&["A"]), &expected), change(1, 0));
}

#[test]
fn an_allowed_addition_is_not_a_gain_but_an_extra_one_is() {
    let before = summary(&["A"]);
    let doc = omriss_core::Document::parse("# A\n".to_string()).unwrap();
    let expected = expected_change(Operation::Add, doc.outline());
    assert_eq!(detect(&before, &summary(&["A", "New"]), &expected), None);
    assert_eq!(
        detect(&before, &summary(&["A", "New", "Surprise"]), &expected),
        change(0, 1)
    );
}

#[test]
fn a_summary_counts_sections_not_the_root() {
    assert_eq!(summary(&["A", "B", "B"]).len(), 3);
    assert!(summary(&[]).is_empty());
}
