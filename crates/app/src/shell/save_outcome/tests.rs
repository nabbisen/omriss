//! RFC-068 §8.2: a file that was written is saved, whatever the disk confirmed.

#[cfg(unix)]
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use omriss_ui::i18n::{Locale, t};
use omriss_ui::{ContentStamp, EditorSession};

use super::apply_save_outcome;
use crate::file::durable_write::Durability;
use crate::file::file_dialog::SaveOutcome;

fn dirty_session(path: &Path) -> EditorSession {
    let mut session =
        EditorSession::open("# A\nbody\n".to_string(), Some(path.to_path_buf())).unwrap();
    let id = session.outline_items()[0].id;
    let base = session.focus(id).unwrap();
    session
        .commit_focused_body(&base, "changed\n".to_string())
        .unwrap();
    assert!(session.is_dirty());
    session
}

#[cfg(unix)]
fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "omriss-save-outcome-{}-{label}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_confirmed_save_marks_the_document_saved_and_says_saved() {
    let mut session = dirty_session(Path::new("a.md"));
    let at = ContentStamp::of(b"the bytes written");
    let key = apply_save_outcome(
        &mut session,
        SaveOutcome::Saved {
            path: PathBuf::from("a.md"),
            stamp: at,
            durability: Durability::Confirmed,
        },
    );
    assert_eq!(key, Some("status.saved"));
    assert!(!session.is_dirty());
    assert_eq!(session.disk_stamp(), Some(at));
}

#[test]
fn an_unconfirmed_save_is_saved_with_a_warning_not_a_failure() {
    let mut session = dirty_session(Path::new("a.md"));
    let at = ContentStamp::of(b"the bytes written");
    let key = apply_save_outcome(
        &mut session,
        SaveOutcome::Saved {
            path: PathBuf::from("a.md"),
            stamp: at,
            durability: Durability::Unconfirmed {
                kind: ErrorKind::Other,
            },
        },
    );
    assert_eq!(key, Some("status.saved_unconfirmed"));
    assert!(
        !key.unwrap().starts_with("error."),
        "a warning is not an error key"
    );
    assert!(!session.is_dirty(), "the document is marked saved");
    assert_eq!(
        session.disk_stamp(),
        Some(at),
        "and the file's stamp is recorded"
    );
}

#[test]
fn a_failed_save_changes_nothing_and_says_why() {
    let mut session = dirty_session(Path::new("a.md"));
    let key = apply_save_outcome(
        &mut session,
        SaveOutcome::Failed {
            kind: ErrorKind::PermissionDenied,
        },
    );
    assert_eq!(key, Some("error.save.permission_denied"));
    assert!(session.is_dirty());
    assert_eq!(session.disk_stamp(), None);
}

#[test]
fn a_cancelled_save_is_silent() {
    let mut session = dirty_session(Path::new("a.md"));
    assert_eq!(
        apply_save_outcome(&mut session, SaveOutcome::Cancelled),
        None
    );
    assert!(session.is_dirty());
}

/// The whole chain with a real file: the folder sync is made to fail, the real write
/// runs, the real outcome is built from it, and the document ends up saved with the
/// warning, the new text on disk and no temporary file.
#[cfg(unix)]
#[test]
fn a_failed_directory_sync_through_the_real_write_is_saved_with_a_warning() {
    use crate::file::durable_write::{Seams, Step, unique_suffix, write_durably_with};
    use crate::file::file_dialog::save_to_path;
    let dir = scratch("dirsync");
    let path = dir.join("a.md");
    fs::write(&path, "# A\nbody\n").unwrap();
    let mut session = dirty_session(&path);

    let write = |target: &Path, bytes: &[u8]| {
        write_durably_with(
            target,
            bytes,
            &Seams {
                before: &|step, _| match step {
                    Step::SyncDir => Err(std::io::Error::other("the disk did not answer")),
                    _ => Ok(()),
                },
                suffix: &unique_suffix,
            },
        )
    };
    let outcome = save_to_path(&path, session.source(), session.profile(), &write);
    assert!(matches!(
        outcome,
        SaveOutcome::Saved {
            durability: Durability::Unconfirmed { .. },
            ..
        }
    ));

    let key = apply_save_outcome(&mut session, outcome);
    assert_eq!(key, Some("status.saved_unconfirmed"));
    assert!(!session.is_dirty());
    assert_eq!(
        session.disk_stamp(),
        Some(ContentStamp::of(&fs::read(&path).unwrap())),
        "the stamp is that of the bytes now on disk"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), session.source());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1, "no temporary file");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn the_warning_has_words_in_both_locales_and_says_both_things() {
    let en = t(Locale::En, "status.saved_unconfirmed");
    let ja = t(Locale::Ja, "status.saved_unconfirmed");
    assert_ne!(en, "status.saved_unconfirmed", "missing in English");
    assert_ne!(ja, "status.saved_unconfirmed", "missing in Japanese");
    assert_ne!(en, ja);
    // It says it was saved, that the disk did not confirm it, and what to do.
    assert!(en.starts_with("Saved"), "{en}");
    assert!(
        en.contains("did not confirm") && en.contains("check the disk"),
        "{en}"
    );
    assert!(
        ja.contains("保存しました") && ja.contains("ディスクを確認"),
        "{ja}"
    );
    assert!(!en.to_lowercase().contains("fail") && !en.to_lowercase().contains("could not"));
    assert!(!ja.contains("失敗"));
}
