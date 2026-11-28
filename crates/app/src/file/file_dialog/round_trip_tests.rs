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

use super::{DialogLabels, OpenOutcome, SaveOutcome, open_markdown_path, save_markdown};

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
    let OpenOutcome::Loaded {
        text,
        path: opened,
        profile,
        ..
    } = open_markdown_path(path)
    else {
        panic!("expected OpenOutcome::Loaded for {}", path.display());
    };
    let format = omriss_core::formats::detection::detect_format(Some(&opened), &text);
    EditorSession::open_detected(text, Some(opened), profile, format)
        .expect("EditorSession::open_detected")
}

/// Saves `session` back to `path` exactly as `handle_save` does.
fn save_session(session: &EditorSession, path: &Path) {
    let outcome = save_markdown(
        Some(path),
        session.source(),
        session.profile(),
        &DialogLabels {
            all_supported: "All supported".into(),
        },
    );
    assert!(
        matches!(outcome, SaveOutcome::Saved { .. }),
        "save_markdown must succeed for {}",
        path.display()
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

/// RFC-068: through the real save entry point, a file reached by a symbolic link is
/// saved in place — the link stays a link, the target's bytes (BOM included) are the
/// session's, and its permissions and folder hold nothing else.
#[cfg(unix)]
#[test]
fn saving_through_a_symlink_updates_the_target_and_keeps_the_link() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let scratch = TempScratch::new("symlink");
    let vault = scratch.path().join("vault");
    fs::create_dir(&vault).unwrap();
    let real = vault.join("notes.md");
    let mut original = vec![0xEF, 0xBB, 0xBF];
    original.extend_from_slice(&fixture_bytes("crlf.md"));
    fs::write(&real, &original).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o600)).unwrap();
    let link = scratch.path().join("link.md");
    symlink(&real, &link).unwrap();

    let mut session = open_session(&link);
    let child_id = find_node_id(&session, "Child");
    let base = session.focus(child_id).expect("focus the Child section");
    session
        .commit_focused_body(&base, "Edited body.\r\n".to_string())
        .expect("commit the edited body");
    let expected = session.source().to_string();
    save_session(&session, &link);

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let mut want = vec![0xEF, 0xBB, 0xBF];
    want.extend_from_slice(expected.as_bytes());
    assert_eq!(fs::read(&real).unwrap(), want);
    assert_eq!(
        fs::metadata(&real).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::read_dir(&vault).unwrap().count(),
        1,
        "no temporary file left"
    );
}

/// RFC-068 §7.5, from the command line to the disk: a path whose bytes are not text
/// opens, is saved at its own path (not at some other one the lossy text of it would
/// name), and is not added to the recent list, which is text. On Linux, whose
/// filesystems take any bytes in a name; macOS's and Windows's do not.
#[cfg(target_os = "linux")]
#[test]
fn a_path_that_is_not_utf8_opens_saves_at_the_same_path_and_is_not_listed() {
    use crate::cli::{StartupArg, interpret};
    use crate::storage::settings::AppSettings;
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let scratch = TempScratch::new("non-utf8");
    let name = OsString::from_vec(b"caf\xe9-\xff.md".to_vec());
    let path = scratch.path().join(&name);
    assert!(path.to_str().is_none(), "the fixture must not be text");
    fs::write(&path, b"# Title\n\nbody\n").unwrap();

    // What `omriss <file>` does: argv (as OsString) -> the interpreter -> open.
    let StartupArg::Path(arg) = interpret(std::iter::once(path.clone().into_os_string())) else {
        panic!("a file name is a path");
    };
    assert_eq!(arg, path);
    let mut session = open_session(&arg);
    assert_eq!(session.file_path(), Some(path.as_path()));

    let id = session.outline_items()[0].id;
    let base = session.focus(id).unwrap();
    session
        .commit_focused_body(&base, "edited\n".to_string())
        .unwrap();
    let saved_to = session.file_path().unwrap().to_path_buf();
    save_session(&session, &saved_to);

    // The bytes landed at the original path, and nowhere else.
    assert_eq!(fs::read(&path).unwrap(), session.source().as_bytes());
    let entries: Vec<OsString> = fs::read_dir(scratch.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].as_bytes(), name.as_bytes());
    // The old way (a lossy `String` and back) names a different file.
    assert_ne!(PathBuf::from(path.display().to_string()), path);

    // A path that is not text cannot be kept in the (TOML) recent list.
    let mut settings = AppSettings::default();
    assert!(!settings.push_recent_path(&path));
    assert!(settings.recent_files.is_empty());
    assert!(settings.push_recent_path(&scratch.path().join("plain.md")));
    assert_eq!(settings.recent_files.len(), 1);
}

#[cfg(not(target_os = "linux"))]
#[test]
#[ignore = "needs a filesystem that takes any bytes in a file name; the Linux test covers it"]
fn a_path_that_is_not_utf8_opens_saves_at_the_same_path_and_is_not_listed() {}
