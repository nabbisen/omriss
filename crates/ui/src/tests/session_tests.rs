use omriss::{EditError, StructuralEditError};

use crate::editor::view_state::ViewMode;
use crate::session::EditorSession;

const DOC: &str =
    "Intro paragraph.\n\n# One\n\nBody one.\n\n## Child\n\nChild body.\n\n# Two\n\nBody two.\n";

fn session() -> EditorSession {
    EditorSession::open(DOC.to_string(), Some("draft.md".into())).unwrap()
}

fn id_of(session: &EditorSession, title: &str) -> omriss::NodeId {
    session
        .outline_items()
        .into_iter()
        .find(|item| item.title == title)
        .unwrap_or_else(|| panic!("no top-level section titled {title:?}"))
        .id
}

#[test]
fn open_session_is_clean_and_shows_top_level_outline() {
    let session = session();
    assert!(!session.is_dirty());
    assert_eq!(session.file_name(), Some("draft.md"));
    assert_eq!(session.view_mode(), ViewMode::Outline);
    let titles: Vec<_> = session
        .outline_items()
        .into_iter()
        .map(|item| item.title)
        .collect();
    assert_eq!(titles, ["One", "Two"]);
}

#[test]
fn document_without_headings_lists_the_root_as_single_layer() {
    let session = EditorSession::open("just text\n".to_string(), None).unwrap();
    let items = session.outline_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].child_count, 0);
}

#[test]
fn commit_marks_dirty_and_undo_to_saved_revision_clears_it() {
    let mut session = session();
    let one = id_of(&session, "One");
    let snapshot = session.focus(one).unwrap();
    session
        .commit_focused_body(&snapshot, "Rewritten.\n\n".to_string())
        .unwrap();
    assert!(session.is_dirty());

    session.undo().unwrap();
    assert!(
        !session.is_dirty(),
        "undoing back to the saved revision must clear the dirty flag"
    );
    assert_eq!(session.source(), DOC, "source restored byte-exactly");

    session.redo().unwrap();
    assert!(session.is_dirty());
}

#[test]
fn mark_saved_adopts_current_revision_and_file_name() {
    let mut session = session();
    let one = id_of(&session, "One");
    let snapshot = session.focus(one).unwrap();
    session
        .commit_focused_body(&snapshot, "New body.\n\n".to_string())
        .unwrap();
    session.mark_saved(Some("final.md".into()));
    assert!(!session.is_dirty());
    assert_eq!(session.file_name(), Some("final.md"));
}

#[test]
fn stale_snapshot_commit_is_rejected_without_mutation() {
    let mut session = session();
    let one = id_of(&session, "One");
    let stale = session.focus(one).unwrap();
    session
        .commit_focused_body(&stale, "First edit.\n\n".to_string())
        .unwrap();

    let before = session.source().to_string();
    let err = session
        .commit_focused_body(&stale, "Conflicting edit.\n\n".to_string())
        .unwrap_err();
    assert!(matches!(err, EditError::RevisionMismatch { .. }));
    assert_eq!(session.source(), before, "rejected edit must not mutate");
}

#[test]
fn focus_navigation_round_trips_through_back_and_forward() {
    let mut session = session();
    let one = id_of(&session, "One");
    let two = id_of(&session, "Two");
    session.focus(one).unwrap();
    session.focus(two).unwrap();

    let back = session.back().unwrap();
    assert_eq!(back.title, "One");
    let forward = session.forward().unwrap();
    assert_eq!(forward.title, "Two");
    assert!(session.can_go_back());
}

#[test]
fn breadcrumb_path_walks_root_to_focused_child() {
    let mut session = session();
    let one = id_of(&session, "One");
    let snapshot = session.focus(one).unwrap();
    let child = snapshot
        .children
        .first()
        .expect("section One has a child")
        .id;
    let child_snapshot = session.focus(child).unwrap();
    let path: Vec<_> = child_snapshot
        .path
        .iter()
        .map(|item| item.title.as_str())
        .collect();
    assert_eq!(path, ["", "One", "Child"], "root has an empty title");
}

#[test]
fn undoing_a_heading_adding_edit_prunes_dead_focus_targets() {
    // Node ids are ordinal-path based (RFC-006), so an id only dies when the
    // outline shrinks at its position. Add a subsection, focus the section
    // that shifted to a new ordinal, then undo: the shifted id vanishes and
    // the navigation history must drop it (RFC-044 interaction).
    let mut session = session();
    let one = id_of(&session, "One");
    let snapshot = session.focus(one).unwrap();
    session
        .commit_focused_body(&snapshot, "Top.\n\n## Added\n\nAdded body.\n\n".to_string())
        .unwrap();

    // "One" now has two children: Added (ordinal 0) and Child (shifted to 1).
    let refreshed = session.current_snapshot().unwrap();
    let titles: Vec<_> = refreshed
        .children
        .iter()
        .map(|item| item.title.as_str())
        .collect();
    assert_eq!(titles, ["Added", "Child"]);
    let shifted_child = refreshed.children[1].id;

    session.focus(shifted_child).unwrap();
    session.back().unwrap(); // forward history now holds the shifted id

    session.undo().unwrap(); // retracts "Added"; ordinal 1 under One dies
    assert!(
        session.forward().is_none(),
        "forward entry to the vanished ordinal must be pruned"
    );
    assert_eq!(
        session.current_snapshot().unwrap().title,
        "One",
        "focus stays on the still-alive section"
    );
}

#[test]
fn new_empty_session_is_clean_and_editable() {
    let mut session = EditorSession::new_empty();
    assert!(!session.is_dirty());
    let root = session.outline_items()[0].id;
    let snapshot = session.focus(root).unwrap();
    session
        .commit_focused_body(&snapshot, "# First Idea\n".to_string())
        .unwrap();
    assert!(session.is_dirty());
    assert_eq!(session.source(), "# First Idea\n");
}

#[test]
fn zoom_out_from_child_focuses_parent() {
    let mut session = session();
    let one = id_of(&session, "One");
    let snapshot = session.focus(one).unwrap();
    let child = snapshot.children.first().expect("One has a child").id;
    session.focus(child).unwrap();
    assert!(matches!(session.view_mode(), ViewMode::Focus(_)));

    session.zoom_out();

    // Should now focus "One" (the parent), not go all the way to outline.
    assert!(
        matches!(session.view_mode(), ViewMode::Focus(id) if id == one),
        "zoom_out from child should focus the parent section"
    );
}

#[test]
fn zoom_out_from_top_level_shows_outline() {
    let mut session = session();
    let one = id_of(&session, "One");
    session.focus(one).unwrap();
    session.zoom_out();
    assert_eq!(
        session.view_mode(),
        ViewMode::Outline,
        "zoom_out from a top-level section must return to the outline overview"
    );
}

#[test]
fn zoom_out_from_outline_is_a_no_op() {
    let mut session = session();
    session.zoom_out(); // already at overview
    assert_eq!(session.view_mode(), ViewMode::Outline);
}

#[test]
fn current_children_in_outline_mode_returns_top_level_items() {
    let session = session();
    let overview = session.current_children();
    let direct = session.outline_items();
    assert_eq!(
        overview.iter().map(|i| &i.title).collect::<Vec<_>>(),
        direct.iter().map(|i| &i.title).collect::<Vec<_>>(),
        "current_children in outline mode should equal outline_items"
    );
}

#[test]
fn current_children_in_focus_mode_returns_section_children() {
    let mut session = session();
    let one = id_of(&session, "One");
    session.focus(one).unwrap();
    let children = session.current_children();
    assert_eq!(children.len(), 1, "section One has one child (Child)");
    assert_eq!(children[0].title, "Child");
}

// ── append_child_to_focused ───────────────────────────────────────────────────

#[test]
fn append_child_to_focused_adds_at_bottom_when_children_exist() {
    // Doc: # One with existing child ## Child, then # Two.
    // Focus "One", add "Second Child" — must appear AFTER "Child", not before.
    let mut session = session();
    let one = id_of(&session, "One");
    session.focus(one).unwrap();
    session
        .append_child_to_focused("Second Child", omriss::HeadingLevel::H2)
        .unwrap();
    // Re-read the snapshot for "One" after the split.
    let snap = session.current_snapshot().unwrap();
    let titles: Vec<&str> = snap.children.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(
        titles,
        vec!["Child", "Second Child"],
        "Second Child must appear after existing Child, got: {titles:?}"
    );
}

#[test]
fn append_child_to_focused_adds_at_bottom_when_no_children() {
    // Focus "Two" (no children), add "New" — should become first and only child.
    let mut session = session();
    let two = id_of(&session, "Two");
    session.focus(two).unwrap();
    session
        .append_child_to_focused("New", omriss::HeadingLevel::H2)
        .unwrap();
    let snap = session.current_snapshot().unwrap();
    let titles: Vec<&str> = snap.children.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(titles, vec!["New"]);
}

// ── add_top_level_section ─────────────────────────────────────────────────────

#[test]
fn add_top_level_section_appends_after_existing_sections() {
    // Doc already has "One" and "Two". Adding "Three" must appear last.
    let mut session = session();
    session.add_top_level_section("Three").unwrap();
    let items = session.outline_items();
    let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(
        titles,
        vec!["One", "Two", "Three"],
        "Three must appear after Two, got: {titles:?}"
    );
}

#[test]
fn add_top_level_section_twice_preserves_order() {
    let mut session = session();
    session.add_top_level_section("Alpha").unwrap();
    session.add_top_level_section("Beta").unwrap();
    let items = session.outline_items();
    let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(
        titles,
        vec!["One", "Two", "Alpha", "Beta"],
        "Alpha then Beta must appear in addition order, got: {titles:?}"
    );
}

#[test]
fn add_after_focused_adds_sibling_after_subtree() {
    let mut session = session();
    let one = id_of(&session, "One");
    session.focus(one).unwrap();
    session.add_after_focused("Between").unwrap();

    let items = session.outline_items();
    let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles, vec!["One", "Between", "Two"]);
}

#[test]
fn rename_focused_preserves_body_and_children() {
    let mut session = session();
    let one = id_of(&session, "One");
    session.focus(one).unwrap();
    session.rename_focused("Renamed").unwrap();

    let snap = session.current_snapshot().unwrap();
    assert_eq!(snap.title, "Renamed");
    assert_eq!(snap.body, "\nBody one.\n\n");
    assert_eq!(snap.children.len(), 1);
    assert!(session.source().contains("# Renamed\n\nBody one."));
    assert!(session.source().contains("## Child\n\nChild body."));
}

#[test]
fn committing_new_child_body_preserves_following_top_level_sibling() {
    let mut session =
        EditorSession::open("# One\n\n# Two\n\nBody two.\n".to_string(), None).unwrap();
    let one = id_of(&session, "One");
    session.focus(one).unwrap();
    session
        .append_child_to_focused("One One", omriss::HeadingLevel::H2)
        .unwrap();

    let child = session.current_snapshot().unwrap().children[0].id;
    let child_snapshot = session.focus(child).unwrap();
    session
        .commit_focused_body(&child_snapshot, "Child body".to_string())
        .unwrap();

    let titles: Vec<String> = session
        .outline_items()
        .into_iter()
        .map(|item| item.title)
        .collect();
    assert_eq!(titles, vec!["One", "Two"]);
    let one_snapshot = session.focus(one).unwrap();
    assert_eq!(one_snapshot.children[0].title, "One One");
    assert!(
        session.source().contains("Child body\n# Two"),
        "following top-level heading must stay on its own line:\n{}",
        session.source()
    );
}

#[test]
fn committing_last_section_body_without_trailing_newline_remains_verbatim() {
    let mut session = EditorSession::open(
        "# One\n\nBody one.\n\n# Two\n\nBody two.\n".to_string(),
        None,
    )
    .unwrap();
    let two = id_of(&session, "Two");
    let two_snapshot = session.focus(two).unwrap();
    session
        .commit_focused_body(&two_snapshot, "Tail body".to_string())
        .unwrap();

    assert!(
        session.source().ends_with("# Two\nTail body"),
        "last section should not gain a synthetic trailing newline:\n{}",
        session.source()
    );
}

#[test]
fn focused_structural_actions_report_missing_focus() {
    let mut session = session();

    assert!(matches!(
        session.add_after_focused("After").unwrap_err(),
        StructuralEditError::NoFocusedSection
    ));
    assert!(matches!(
        session.rename_focused("Renamed").unwrap_err(),
        StructuralEditError::NoFocusedSection
    ));
}
