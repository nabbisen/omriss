//! RFC-076: committing an edit of the plain file text.

use omriss_core::{DocumentFormat, EditError, NodeId};

use crate::editor::raw_text::{as_shown_in_a_text_area, raw_edit};
use crate::editor::structure_change::StructureChange;
use crate::editor::view_state::ViewMode;
use crate::file::file_profile::FileTextProfile;
use crate::interface::document_map::{DocumentMapNode, node_id_from_raw};
use crate::session::{EditorSession, RawCommit};

const TWO: &str = "# One\nbody\n\n# Two\nmore\n";

fn open(md: &str) -> EditorSession {
    EditorSession::open(md.to_string(), None).unwrap()
}

fn find_in(node: &DocumentMapNode, title: &str) -> Option<NodeId> {
    if node.title == title {
        return Some(node_id_from_raw(node.id));
    }
    node.children.iter().find_map(|c| find_in(c, title))
}

fn id_of(s: &EditorSession, title: &str) -> NodeId {
    find_in(&s.document_map_nodes(), title).unwrap_or_else(|| panic!("no section {title:?}"))
}

fn commit(s: &mut EditorSession, draft: &str) -> RawCommit {
    let rev = s.revision();
    s.commit_raw_text(draft, rev).unwrap()
}

fn warning(lost: usize, gained: usize) -> Option<StructureChange> {
    Some(StructureChange { lost, gained })
}

// ── The PR 2 table ──────────────────────────────────────────────────────────

#[test]
fn a_commit_that_changes_nothing_writes_nothing_and_records_no_history() {
    let mut s = open(TWO);
    let draft = s.raw_text();
    assert_eq!(commit(&mut s, &draft), RawCommit::Unchanged);
    assert_eq!(s.source(), TWO);
    assert!(!s.can_undo(), "no history entry for a no-op");
    assert_eq!(s.structure_change(), None);
}

#[test]
fn a_no_op_commit_neither_raises_nor_clears_a_warning() {
    let mut s = open(TWO);
    let id = id_of(&s, "One");
    let snap = s.focus(id).unwrap();
    s.commit_focused_body(&snap, "<!--".to_string()).unwrap();
    let warned = s.structure_change();
    assert!(warned.is_some());
    let draft = s.raw_text();
    assert_eq!(commit(&mut s, &draft), RawCommit::Unchanged);
    assert_eq!(s.structure_change(), warned);
}

#[test]
fn rewriting_one_body_is_one_undo_step_and_no_structure_warning() {
    let mut s = open(TWO);
    let draft = s.raw_text().replace("more", "much more");
    assert!(matches!(commit(&mut s, &draft), RawCommit::Applied(_)));
    assert_eq!(s.source(), "# One\nbody\n\n# Two\nmuch more\n");
    assert_eq!(s.structure_change(), None);
    s.undo().unwrap();
    assert_eq!(s.source(), TWO);
    assert!(!s.can_undo(), "exactly one undo step");
}

#[test]
fn removing_every_heading_warns_with_the_right_count_and_is_one_undo_step() {
    let mut s = open(TWO);
    assert!(matches!(
        commit(&mut s, "just prose now\nnothing to navigate\n"),
        RawCommit::Applied(_)
    ));
    assert_eq!(s.structure_change(), warning(2, 0));
    s.undo().unwrap();
    assert_eq!(s.source(), TWO, "one Ctrl+Z returns the whole document");
    assert!(!s.can_undo());
}

#[test]
fn adding_headings_warns() {
    let mut s = open(TWO);
    let draft = format!("{}\n# Three\n\n## Three-a\n", s.raw_text());
    commit(&mut s, &draft);
    assert_eq!(s.structure_change(), warning(0, 2));
}

#[test]
fn renaming_a_heading_in_raw_text_warns_as_both() {
    // The comparison cannot know the rename was meant, so it says so.
    let mut s = open(TWO);
    let draft = s.raw_text().replace("# Two", "# Deux");
    commit(&mut s, &draft);
    assert_eq!(s.structure_change(), warning(1, 1));
}

#[test]
fn replacing_the_whole_document_is_one_transaction() {
    let mut s = open(TWO);
    commit(&mut s, "# Entirely\n\nnew\n\n## Different\n\ntext\n");
    assert_eq!(s.source(), "# Entirely\n\nnew\n\n## Different\n\ntext\n");
    s.undo().unwrap();
    assert_eq!(s.source(), TWO);
    assert!(!s.can_undo());
    s.redo().unwrap();
    assert_eq!(s.source(), "# Entirely\n\nnew\n\n## Different\n\ntext\n");
}

#[test]
fn a_stale_revision_is_refused_and_the_source_is_unchanged() {
    let mut s = open(TWO);
    let stale = s.revision();
    let id = id_of(&s, "One");
    let snap = s.focus(id).unwrap();
    s.commit_focused_body(&snap, "moved on\n".to_string())
        .unwrap();
    let after = s.source().to_string();

    let err = s
        .commit_raw_text("something else entirely\n", stale)
        .unwrap_err();
    assert!(matches!(err, EditError::RevisionMismatch { .. }), "{err:?}");
    assert_eq!(s.source(), after, "a refused raw commit changes nothing");

    // Even a draft that happens to match the current text is refused: it was
    // composed against a different document.
    let same = s.raw_text();
    assert!(s.commit_raw_text(&same, stale).is_err());
}

// ── Focus (§4.2): the outline, always ───────────────────────────────────────

#[test]
fn focus_returns_to_the_outline_even_when_the_same_section_still_exists() {
    // The tempting case: the focused section survives with the same title.
    let mut s = open(TWO);
    let one = id_of(&s, "One");
    s.focus(one).unwrap();
    let two = id_of(&s, "Two");
    s.focus(two).unwrap();
    assert!(s.can_go_back());

    let draft = s.raw_text().replace("more", "even more");
    commit(&mut s, &draft);

    assert_eq!(s.view_mode(), ViewMode::Outline, "never remapped to Two");
    assert!(s.current_snapshot().is_none());
    assert!(!s.can_go_back(), "history is cleared, not pruned");
    assert!(!s.can_go_forward());
    assert!(s.back().is_none());
}

#[test]
fn leaving_the_raw_view_after_a_commit_lands_on_the_outline() {
    let mut s = open(TWO);
    let one = id_of(&s, "One");
    s.focus(one).unwrap();
    s.show_raw();
    let draft = s.raw_text().replace("body", "BODY");
    commit(&mut s, &draft);
    assert!(s.is_raw(), "the user keeps editing where they are");
    s.leave_raw();
    assert_eq!(s.view_mode(), ViewMode::Outline);
    assert!(s.current_snapshot().is_none());
}

#[test]
fn an_unchanged_commit_leaves_focus_alone() {
    let mut s = open(TWO);
    let one = id_of(&s, "One");
    s.focus(one).unwrap();
    let draft = s.raw_text();
    commit(&mut s, &draft);
    assert_eq!(s.view_mode(), ViewMode::Focus(one), "nothing was committed");
}

// ── Line endings: the invariant the project rests on ────────────────────────

const CRLF: &str = "# One\r\nbody\r\n\r\n# Two\r\nmore\r\n";

#[test]
fn a_text_area_never_shows_a_carriage_return() {
    assert_eq!(as_shown_in_a_text_area("a\r\nb\rc\nd"), "a\nb\nc\nd");
    let s = open(CRLF);
    assert!(!s.raw_text().contains('\r'));
}

#[test]
fn opening_the_raw_view_and_committing_unchanged_leaves_a_crlf_file_byte_identical() {
    let mut s = open(CRLF);
    let shown = s.raw_text();
    assert_eq!(commit(&mut s, &shown), RawCommit::Unchanged);
    assert_eq!(s.source().as_bytes(), CRLF.as_bytes());
    assert!(!s.can_undo());
}

#[test]
fn an_edit_in_a_crlf_file_changes_only_what_was_touched() {
    let mut s = open(CRLF);
    let draft = s.raw_text().replace("more", "more\nand a new line");
    commit(&mut s, &draft);
    assert_eq!(
        s.source(),
        "# One\r\nbody\r\n\r\n# Two\r\nmore\r\nand a new line\r\n",
        "untouched endings survive and the inserted newline takes the file's style"
    );
    s.undo().unwrap();
    assert_eq!(s.source().as_bytes(), CRLF.as_bytes());
}

#[test]
fn mixed_line_endings_outside_the_edit_are_preserved_exactly() {
    let mixed = "# One\r\nbody\n\n# Two\r\nmore\n";
    let mut s = open(mixed);
    let draft = s.raw_text().replace("body", "BODY");
    commit(&mut s, &draft);
    assert_eq!(s.source(), "# One\r\nBODY\n\n# Two\r\nmore\n");
    s.undo().unwrap();
    assert_eq!(s.source().as_bytes(), mixed.as_bytes());
}

#[test]
fn a_missing_trailing_newline_is_kept_and_can_be_added() {
    let mut s = open("# One\nbody");
    let unchanged = s.raw_text();
    assert_eq!(commit(&mut s, &unchanged), RawCommit::Unchanged);
    commit(&mut s, "# One\nbody\n");
    assert_eq!(s.source(), "# One\nbody\n");
    s.undo().unwrap();
    assert_eq!(s.source(), "# One\nbody");
}

// ── Other formats: the raw view is the recovery path ────────────────────────

#[test]
fn a_malformed_json_file_can_be_repaired_in_the_raw_view() {
    let broken = "{\"a\": 1,}\n";
    let mut s = EditorSession::open_detected(
        broken.to_string(),
        None,
        FileTextProfile::detect(broken, false),
        DocumentFormat::Json,
    )
    .unwrap();
    assert!(
        s.document_map_nodes().children.is_empty(),
        "not strict JSON"
    );

    commit(&mut s, "{\"a\": 1}\n");

    assert_eq!(s.source(), "{\"a\": 1}\n");
    assert_eq!(
        s.structure_change(),
        None,
        "no Markdown outline to warn about"
    );
    assert!(
        find_in(&s.document_map_nodes(), "a").is_some(),
        "the repaired file now has a structure"
    );
    s.undo().unwrap();
    assert_eq!(s.source(), broken);
}

// ── The edit itself (pure) ──────────────────────────────────────────────────

/// Every single-span edit of a document's text-area view, applied through
/// `raw_edit`: the document must show exactly the draft afterwards, the bytes
/// outside the span must be untouched, and undoing (re-applying the inverse)
/// must restore every byte.
#[test]
fn every_single_span_edit_realises_the_draft_and_preserves_everything_else() {
    let corpus = [
        "",
        "x",
        "# A\nbody\n",
        "# A\r\nbody\r\n\r\n# B\r\nx\r\n",
        "# A\r\nbody\n\n# B\nx\r\n",
        "no newline at all",
        "tail without newline\r\nsecond",
        "日本語\r\nの見出し\n",
        "A\n===\nsetext\n---\n<!--\n",
        "\r\n\r\n\r\n",
        "lone\rcr\rhere",
    ];
    let snippets = ["", "Z", "\n", "\n\n", "é\nü", "# H\n"];

    for source in corpus {
        let shown = as_shown_in_a_text_area(source);
        let cuts: Vec<usize> = shown
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(shown.len()))
            .collect();
        for &i in &cuts {
            for &j in cuts.iter().filter(|&&j| j >= i) {
                for snippet in snippets {
                    let draft = format!("{}{}{}", &shown[..i], snippet, &shown[j..]);
                    let Some(edit) = raw_edit(source, &draft) else {
                        assert_eq!(
                            as_shown_in_a_text_area(&draft),
                            shown,
                            "None means the draft shows what the source shows"
                        );
                        continue;
                    };
                    let mut applied = source.to_string();
                    applied.replace_range(edit.range.clone(), &edit.replacement);

                    assert_eq!(
                        as_shown_in_a_text_area(&applied),
                        as_shown_in_a_text_area(&draft),
                        "{source:?} -> {draft:?}: the document must show the draft"
                    );
                    assert_eq!(
                        &applied[..edit.range.start],
                        &source[..edit.range.start],
                        "{source:?} -> {draft:?}: bytes before the edit changed"
                    );
                    let tail_len = source.len() - edit.range.end;
                    assert_eq!(
                        &applied[applied.len() - tail_len..],
                        &source[edit.range.end..],
                        "{source:?} -> {draft:?}: bytes after the edit changed"
                    );
                    let mut undone = applied.clone();
                    let new_end = edit.range.start + edit.replacement.len();
                    undone.replace_range(edit.range.start..new_end, &source[edit.range.clone()]);
                    assert_eq!(undone, source, "{source:?} -> {draft:?}: undo is not exact");
                }
            }
        }
    }
}

#[test]
fn the_edit_is_minimal_not_the_whole_document() {
    let source = "# One\nbody\n\n# Two\nmore\n";
    let edit = raw_edit(source, &source.replace("body", "BODY")).unwrap();
    assert_eq!(&source[edit.range], "body");
    assert_eq!(edit.replacement, "BODY");
}
