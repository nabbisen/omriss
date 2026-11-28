//! RFC-068 §7.6–§7.7, from real files: another program's change is found by what the
//! file holds, and Reload followed by Ctrl+Z makes the next save ask again.
//!
//! The old check was `disk_mtime > saved_mtime`. Each case here is one it could not see.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use omriss_ui::{ContentStamp, EditorSession};

use super::{
    DialogLabels, OpenOutcome, SaveOutcome, changed_on_disk, open_markdown_path, save_markdown,
};
use crate::shell::save_outcome::apply_save_outcome;

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "omriss-diskchange-{}-{label}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn mtime(path: &Path) -> SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

fn set_mtime(path: &Path, to: SystemTime) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(to)
        .unwrap();
}

/// What the app does on open: read the file, keep its stamp on the session.
fn open(path: &Path) -> EditorSession {
    let OpenOutcome::Loaded {
        text,
        path: opened,
        profile,
        stamp,
    } = open_markdown_path(path)
    else {
        panic!("could not open {}", path.display());
    };
    let mut session = EditorSession::open_with_profile(text, Some(opened), profile).unwrap();
    session.set_disk_stamp(Some(stamp));
    session
}

fn changed(session: &EditorSession) -> bool {
    changed_on_disk(session.file_path().unwrap(), session.disk_stamp())
}

/// What the app does on Reload: read the file again and hand it to the session.
fn reload(session: &mut EditorSession) {
    let path = session.file_path().unwrap().to_path_buf();
    let OpenOutcome::Loaded {
        text,
        profile,
        stamp,
        ..
    } = open_markdown_path(&path)
    else {
        panic!("could not reload");
    };
    session.reload_from_disk(text, profile, stamp).unwrap();
}

fn type_into_first_section(session: &mut EditorSession, body: &str) {
    let id = session.outline_items()[0].id;
    let base = session.focus(id).unwrap();
    session
        .commit_focused_body(&base, body.to_string())
        .unwrap();
}

fn labels() -> DialogLabels {
    DialogLabels {
        all_supported: "All supported".into(),
    }
}

// ── the cases the mtime check could not see ──────────────────────────────────

#[test]
fn an_untouched_file_is_unchanged() {
    let dir = Scratch::new("untouched");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nbody\n").unwrap();
    assert!(!changed(&open(&path)));
}

#[test]
fn a_change_with_the_same_size_and_the_same_mtime_is_detected() {
    let dir = Scratch::new("same-mtime");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nbody 1\n").unwrap();
    let session = open(&path);
    let before = mtime(&path);

    fs::write(&path, "# A\n\nbody 2\n").unwrap(); // same length
    set_mtime(&path, before); // and the same timestamp: what a one-second clock does
    assert_eq!(mtime(&path), before);
    assert_eq!(fs::metadata(&path).unwrap().len(), 12);

    assert!(changed(&session), "equal size and time, different bytes");
}

#[test]
fn a_change_whose_mtime_was_restored_to_an_older_value_is_detected() {
    let dir = Scratch::new("older-mtime");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nbody\n").unwrap();
    let session = open(&path);
    let opened_at = mtime(&path);

    // git checkout, rsync --times and touch -d all do this: new bytes, an old time.
    fs::write(&path, "# A\n\na different body, longer\n").unwrap();
    set_mtime(&path, opened_at - Duration::from_secs(86_400));
    assert!(mtime(&path) < opened_at);

    assert!(changed(&session));
}

#[test]
fn a_newer_mtime_over_the_same_bytes_is_not_a_change() {
    let dir = Scratch::new("touch");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nbody\n").unwrap();
    let session = open(&path);
    set_mtime(&path, mtime(&path) + Duration::from_secs(3600)); // `touch`
    fs::write(&path, "# A\n\nbody\n").unwrap(); // rewritten identically
    assert!(!changed(&session), "same bytes: nothing to ask about");
}

#[test]
fn a_deleted_file_is_not_what_was_opened() {
    let dir = Scratch::new("deleted");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n").unwrap();
    let session = open(&path);
    fs::remove_file(&path).unwrap();
    assert!(changed(&session));
}

#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_read_is_asked_about_not_overwritten_unseen() {
    use std::os::unix::fs::PermissionsExt;
    let dir = Scratch::new("unreadable");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n").unwrap();
    let session = open(&path);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&path).is_ok() {
        return; // running as root: nothing stops the read, nothing to show
    }
    assert!(changed(&session));
}

#[test]
fn a_document_with_no_stamp_cannot_be_compared_and_is_asked_about() {
    let dir = Scratch::new("nostamp");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n").unwrap();
    assert!(changed_on_disk(&path, None));
}

#[test]
fn the_stamp_of_a_save_is_the_stamp_of_the_file_it_wrote() {
    let dir = Scratch::new("saved");
    let path = dir.file("a.md");
    fs::write(&path, "\u{feff}# A\r\n\r\nbody\r\n").unwrap(); // a BOM and CRLF
    let mut session = open(&path);
    type_into_first_section(&mut session, "\r\nedited\r\n");
    let outcome = save_markdown(Some(&path), session.source(), session.profile(), &labels());
    assert!(matches!(outcome, SaveOutcome::Saved { .. }));
    apply_save_outcome(&mut session, outcome);
    assert_eq!(
        session.disk_stamp(),
        Some(ContentStamp::of(&fs::read(&path).unwrap())),
        "BOM included: the stamp is of the bytes, not of the text"
    );
    assert!(
        !changed(&session),
        "our own save is not another program's change"
    );
}

// ── Reload, and Ctrl+Z after it ──────────────────────────────────────────────

#[test]
fn reload_then_undo_makes_the_next_save_ask_again() {
    let dir = Scratch::new("reload-undo");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nstarted\n").unwrap();
    let mut session = open(&path);
    type_into_first_section(&mut session, "\nmy unsaved words\n");
    let unsaved = session.source().to_string();
    assert!(session.is_dirty());

    // Another program rewrites the file, keeping its timestamp.
    let before = mtime(&path);
    fs::write(&path, "# A\n\nwritten elsewhere\n").unwrap();
    set_mtime(&path, before);
    assert!(changed(&session), "the save would have asked");

    // The user chooses Reload from disk.
    reload(&mut session);
    assert_eq!(session.source(), "# A\n\nwritten elsewhere\n");
    assert!(!session.is_dirty());
    assert!(
        !changed(&session),
        "the document is the file again: a save would not ask"
    );

    // Ctrl+Z: the unsaved words come back...
    session.undo().unwrap();
    assert_eq!(session.source(), unsaved);
    assert!(session.is_dirty());
    // ...and saving them would overwrite what the other program wrote, so it asks again.
    assert!(changed(&session), "saving the restored text must ask again");

    // Redo is the reloaded state, which is the file: no ask.
    session.redo().unwrap();
    assert!(!changed(&session));
    session.undo().unwrap();
    assert!(changed(&session));
}

#[test]
fn choosing_overwrite_after_the_ask_saves_the_users_text_and_stops_asking() {
    let dir = Scratch::new("overwrite");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nstarted\n").unwrap();
    let mut session = open(&path);
    type_into_first_section(&mut session, "\nmy unsaved words\n");
    fs::write(&path, "# A\n\nwritten elsewhere\n").unwrap();
    assert!(changed(&session));

    // Overwrite is the same save, taken after the question.
    let outcome = save_markdown(Some(&path), session.source(), session.profile(), &labels());
    apply_save_outcome(&mut session, outcome);
    assert_eq!(fs::read_to_string(&path).unwrap(), session.source());
    assert!(!changed(&session));
    assert!(!session.is_dirty());
}

#[test]
fn reload_of_a_file_that_is_gone_leaves_the_document_alone() {
    let dir = Scratch::new("reload-gone");
    let path = dir.file("a.md");
    fs::write(&path, "# A\n\nstarted\n").unwrap();
    let mut session = open(&path);
    type_into_first_section(&mut session, "\nmy unsaved words\n");
    let unsaved = session.source().to_string();
    fs::remove_file(&path).unwrap();
    // The handler reads first and replaces only on success: a failed read is not a reload.
    assert!(matches!(
        open_markdown_path(&path),
        OpenOutcome::Failed { .. }
    ));
    assert_eq!(session.source(), unsaved);
    assert!(session.is_dirty());
}
