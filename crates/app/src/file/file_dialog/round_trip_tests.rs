//! Byte round-trip tests through the app's real open/save layer (Task 014).
//!
//! `RELEASE_CHECKLIST.md` steps 7 and 17 used to have a human open the saved
//! file in an external editor and confirm only the edited bytes changed.
//! These tests replace that manual byte inspection with something
//! `cargo test --workspace` runs on every push, on every CI platform: they
//! drive `open_markdown_path`/`save_markdown` exactly as
//! `crates/app/src/shell/actions.rs`'s `handle_load`/`handle_save` do,
//! against a **copy** of each fixture in a fresh temp directory — the
//! repository's own fixtures are read-only inputs, never written to.
//!
//! Every comparison is raw `Vec<u8>`, never `String`: a `String` comparison
//! normalizes away exactly what this layer is responsible for (a BOM, a
//! line-ending byte) and would not be able to see a regression in it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use omriss_core::NodeId;
use omriss_ui::{EditorSession, node_id_from_raw};

use super::{OpenOutcome, SaveOutcome, open_markdown_path, save_markdown};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../core/tests/fixtures");

/// A temp directory unique to one test, removed when the test ends (even on
/// panic, via `Drop`) — constraint 2: never write into the repository, and
/// leave no litter in the OS temp directory either.
struct TempScratch(PathBuf);

impl TempScratch {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "omriss-round-trip-{}-{label}-{n}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp scratch dir");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture_bytes(fixture_name: &str) -> Vec<u8> {
    let src = Path::new(FIXTURES).join(fixture_name);
    fs::read(&src).unwrap_or_else(|e| panic!("read fixture {src:?}: {e}"))
}

/// Opens `path` exactly as `handle_load` does: `open_markdown_path`, then
/// extension/content format detection, then `EditorSession::open_detected`.
fn open_session(path: &Path) -> EditorSession {
    let path_str = path.to_str().expect("temp path is valid UTF-8").to_string();
    let OpenOutcome::Loaded {
        text,
        name,
        profile,
        ..
    } = open_markdown_path(&path_str)
    else {
        panic!("expected OpenOutcome::Loaded for {path_str}");
    };
    let format = omriss_core::formats::detection::detect_format(Some(Path::new(&name)), &text);
    EditorSession::open_detected(text, Some(name), profile, format)
        .expect("EditorSession::open_detected")
}

/// Saves `session` back to `path` exactly as `handle_save` does.
fn save_session(session: &EditorSession, path: &Path) {
    let path_str = path.to_str().expect("temp path is valid UTF-8");
    let outcome = save_markdown(Some(path_str), session.source(), session.profile());
    assert!(
        matches!(outcome, SaveOutcome::Saved { .. }),
        "save_markdown must succeed for {path_str}"
    );
}

/// Depth-first search over the whole Document Map tree for a node titled
/// `title` — not just the root's immediate children, since a Markdown
/// document's synthetic root has exactly one child (the top `#` heading);
/// a `##` section like "Abstract" is that H1's child, not the root's.
fn find_node_id(session: &EditorSession, title: &str) -> NodeId {
    fn search(node: &omriss_ui::DocumentMapNode, title: &str) -> Option<NodeId> {
        if node.title == title {
            return Some(node_id_from_raw(node.id));
        }
        node.children.iter().find_map(|c| search(c, title))
    }
    let root = session.document_map_nodes();
    search(&root, title).unwrap_or_else(|| panic!("no node titled {title:?} anywhere in the map"))
}

/// Open → save with no edit at all: the file on disk must be byte-identical
/// to the original, BOM and line endings included.
fn assert_no_edit_round_trip_is_byte_identical(fixture_name: &str, bom: bool) {
    let scratch = TempScratch::new("no-edit");
    let mut original_bytes = if bom {
        vec![0xEF, 0xBB, 0xBF]
    } else {
        Vec::new()
    };
    original_bytes.extend_from_slice(&fixture_bytes(fixture_name));
    let path = scratch.path().join(fixture_name);
    fs::write(&path, &original_bytes).expect("write temp fixture copy");

    let session = open_session(&path);
    save_session(&session, &path);

    let after = fs::read(&path).expect("read saved file");
    assert_eq!(
        after, original_bytes,
        "a save with no edit must be byte-identical to the original ({fixture_name}, bom={bom})"
    );
}

/// Open → one real edit → save: the file on disk must be exactly the BOM
/// (if any) plus the post-edit `EditorSession::source()` — proving the I/O
/// layer wrote precisely what the core session produced, no more and no
/// less — and `unrelated_needle`, taken from a part of the original the
/// edit never touches, must still be present untouched.
fn assert_edited_round_trip_preserves_everything_else(
    fixture_name: &str,
    bom: bool,
    edit: impl FnOnce(&mut EditorSession),
    unrelated_needle: &str,
) {
    let scratch = TempScratch::new("edit");
    let mut original_bytes = if bom {
        vec![0xEF, 0xBB, 0xBF]
    } else {
        Vec::new()
    };
    original_bytes.extend_from_slice(&fixture_bytes(fixture_name));
    let path = scratch.path().join(fixture_name);
    fs::write(&path, &original_bytes).expect("write temp fixture copy");

    let mut session = open_session(&path);
    edit(&mut session);
    let expected_source = session.source().to_string();
    save_session(&session, &path);

    let after = fs::read(&path).expect("read saved file");
    let mut expected_bytes = if bom {
        vec![0xEF, 0xBB, 0xBF]
    } else {
        Vec::new()
    };
    expected_bytes.extend_from_slice(expected_source.as_bytes());
    assert_eq!(
        after, expected_bytes,
        "disk bytes must be exactly the BOM (if any) plus the edited session source ({fixture_name}, bom={bom})"
    );
    assert_ne!(
        after, original_bytes,
        "the edit must actually have changed the bytes on disk ({fixture_name})"
    );
    assert!(
        String::from_utf8_lossy(&after).contains(unrelated_needle),
        "unrelated content {unrelated_needle:?} must survive the edit untouched ({fixture_name})"
    );
}

// ── crlf.md — Markdown, CRLF line endings ───────────────────────────────────

#[test]
fn crlf_md_round_trips_byte_identical_with_no_edit() {
    assert_no_edit_round_trip_is_byte_identical("crlf.md", false);
}

#[test]
fn crlf_md_round_trips_with_one_edit() {
    assert_edited_round_trip_preserves_everything_else(
        "crlf.md",
        false,
        |session| {
            let child_id = find_node_id(session, "Child");
            let base = session.focus(child_id).expect("focus the Child section");
            session
                .commit_focused_body(&base, "Edited body.\r\n".to_string())
                .expect("commit the edited body");
        },
        "Line one.",
    );
}

// ── academic-paper.md — the checklist's own Markdown fixture ───────────────

#[test]
fn academic_paper_round_trips_byte_identical_with_no_edit() {
    assert_no_edit_round_trip_is_byte_identical("academic-paper.md", false);
}

#[test]
fn academic_paper_round_trips_with_one_edit() {
    assert_edited_round_trip_preserves_everything_else(
        "academic-paper.md",
        false,
        |session| {
            let abstract_id = find_node_id(session, "Abstract");
            let base = session
                .focus(abstract_id)
                .expect("focus the Abstract section");
            session
                .commit_focused_body(&base, "Edited abstract text.\n".to_string())
                .expect("commit the edited body");
        },
        "sacrifices navigability",
    );
}

// ── structured-sample.json — the checklist's own JSON fixture ──────────────

#[test]
fn structured_sample_round_trips_byte_identical_with_no_edit() {
    assert_no_edit_round_trip_is_byte_identical("structured-sample.json", false);
}

#[test]
fn structured_sample_round_trips_with_one_edit() {
    assert_edited_round_trip_preserves_everything_else(
        "structured-sample.json",
        false,
        |session| {
            let title_id = find_node_id(session, "title");
            session.focus(title_id).expect("focus the title value");
            session
                .commit_structured_draft("Edited title")
                .expect("commit the edited value");
        },
        "Indentation on this line is deliberately wrong",
    );
}

// ── BOM-prefixed file — no fixture on disk carries one, per the task's own
//    instruction not to add one; built here by prefixing an existing
//    fixture's bytes with EF BB BF. ───────────────────────────────────────

#[test]
fn bom_prefixed_file_round_trips_byte_identical_with_no_edit() {
    assert_no_edit_round_trip_is_byte_identical("academic-paper.md", true);
}

#[test]
fn bom_prefixed_file_round_trips_with_one_edit() {
    assert_edited_round_trip_preserves_everything_else(
        "academic-paper.md",
        true,
        |session| {
            let abstract_id = find_node_id(session, "Abstract");
            let base = session
                .focus(abstract_id)
                .expect("focus the Abstract section");
            session
                .commit_focused_body(&base, "Edited abstract text.\n".to_string())
                .expect("commit the edited body");
        },
        "sacrifices navigability",
    );
}
