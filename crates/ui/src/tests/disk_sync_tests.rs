//! RFC-068 §7.6–§7.7: Reload is one undo step, and the disk stamp follows undo.
//!
//! The rule the tests pin: a save compares the file with the stamp of the file *the
//! document's text was made from*. After Reload then Undo the text is the one made
//! from the OLD file, so the stamp must be the old one, or saving it would overwrite
//! the other program's changes without asking.

use std::path::PathBuf;

use crate::{ContentStamp, EditorSession, FileTextProfile};

const OLD: &str = "# Title\n\nbody the user started from\n";
const UNSAVED: &str = "# Title\n\nbody the user started from, and more typed since\n";
const NEW_ON_DISK: &str = "# Title\n\nbody another program wrote\n\n## Added there\n\ntext\n";

fn stamp(bytes: &str) -> ContentStamp {
    ContentStamp::of(bytes.as_bytes())
}

/// A session opened from `OLD` (disk stamp of OLD) with an unsaved edit: the document
/// holds `UNSAVED`.
fn session_with_unsaved_edit() -> EditorSession {
    let mut s = EditorSession::open(OLD.to_string(), Some(PathBuf::from("a.md"))).unwrap();
    s.set_disk_stamp(Some(stamp(OLD)));
    let id = s.outline_items()[0].id;
    let base = s.focus(id).unwrap();
    s.commit_focused_body(
        &base,
        "\nbody the user started from, and more typed since\n".to_string(),
    )
    .unwrap();
    assert_eq!(s.source(), UNSAVED);
    assert!(s.is_dirty());
    s
}

fn reload(s: &mut EditorSession, text: &str) {
    let profile = FileTextProfile::detect(text, false);
    s.reload_from_disk(text.to_string(), profile, stamp(text))
        .unwrap();
}

#[test]
fn reload_replaces_the_document_and_marks_it_saved() {
    let mut s = session_with_unsaved_edit();
    reload(&mut s, NEW_ON_DISK);
    assert_eq!(s.source(), NEW_ON_DISK);
    assert!(!s.is_dirty(), "the document is saved as the disk has it");
    assert_eq!(s.disk_stamp(), Some(stamp(NEW_ON_DISK)));
    assert_eq!(s.file_path(), Some(std::path::Path::new("a.md")));
}

#[test]
fn reload_is_one_undo_step_that_brings_back_the_unsaved_text() {
    let mut s = session_with_unsaved_edit();
    reload(&mut s, NEW_ON_DISK);
    s.undo().unwrap();
    assert_eq!(
        s.source(),
        UNSAVED,
        "Ctrl+Z restores the unsaved text, exactly"
    );
    assert!(s.is_dirty());
    s.redo().unwrap();
    assert_eq!(s.source(), NEW_ON_DISK);
    assert!(!s.is_dirty(), "redoing the reload is the saved state again");
}

#[test]
fn it_is_exactly_one_step_not_several() {
    let mut s = session_with_unsaved_edit();
    // Undo the user's own edit first to count steps: after reload, one undo reaches
    // the unsaved text, and a second reaches OLD (the edit's own step).
    reload(&mut s, NEW_ON_DISK);
    s.undo().unwrap();
    assert_eq!(s.source(), UNSAVED);
    s.undo().unwrap();
    assert_eq!(s.source(), OLD);
    assert!(!s.can_undo());
}

#[test]
fn saving_after_the_undo_asks_again_because_the_stamp_goes_back() {
    let mut s = session_with_unsaved_edit();
    assert_eq!(s.disk_stamp(), Some(stamp(OLD)));
    reload(&mut s, NEW_ON_DISK);
    assert_eq!(s.disk_stamp(), Some(stamp(NEW_ON_DISK)));
    s.undo().unwrap();
    // The text is the one made from OLD, and the disk holds NEW_ON_DISK: a save
    // compares the file with this stamp, finds NEW_ON_DISK, and asks.
    assert_eq!(s.disk_stamp(), Some(stamp(OLD)));
    s.redo().unwrap();
    assert_eq!(s.disk_stamp(), Some(stamp(NEW_ON_DISK)));
}

#[test]
fn after_a_reload_and_no_undo_saving_does_not_ask() {
    let mut s = session_with_unsaved_edit();
    reload(&mut s, NEW_ON_DISK);
    // An edit made after the reload is made from what the disk holds now.
    let id = s.outline_items()[0].id;
    let base = s.focus(id).unwrap();
    s.commit_focused_body(&base, "typed after reloading\n".to_string())
        .unwrap();
    assert_eq!(s.disk_stamp(), Some(stamp(NEW_ON_DISK)));
    s.undo().unwrap(); // undoing that edit changes nothing about the disk
    assert_eq!(s.disk_stamp(), Some(stamp(NEW_ON_DISK)));
}

#[test]
fn undoing_through_edits_made_after_the_reload_still_lands_on_the_old_stamp() {
    let mut s = session_with_unsaved_edit();
    reload(&mut s, NEW_ON_DISK);
    let id = s.outline_items()[0].id;
    let base = s.focus(id).unwrap();
    s.commit_focused_body(&base, "typed after reloading\n".to_string())
        .unwrap();
    s.undo().unwrap(); // the typing
    s.undo().unwrap(); // the reload
    assert_eq!(s.source(), UNSAVED);
    assert_eq!(s.disk_stamp(), Some(stamp(OLD)));
}

#[test]
fn a_save_makes_the_disk_what_the_document_says_and_forgets_earlier_reloads() {
    let mut s = session_with_unsaved_edit();
    reload(&mut s, NEW_ON_DISK);
    // The user types, saves (the file now holds SAVED), then undoes everything.
    let id = s.outline_items()[0].id;
    let base = s.focus(id).unwrap();
    s.commit_focused_body(&base, "final\n".to_string()).unwrap();
    let saved = s.source().to_string();
    s.mark_saved(None);
    s.set_disk_stamp(Some(stamp(&saved)));
    s.undo().unwrap();
    s.undo().unwrap();
    assert_eq!(s.source(), UNSAVED);
    // The disk holds what omriss saved; the stamp of the OLD file must not come back.
    assert_eq!(s.disk_stamp(), Some(stamp(&saved)));
}

#[test]
fn the_profile_follows_the_reload_and_its_undo() {
    let mut s = EditorSession::open_with_profile(
        "# T\n\nx\n".to_string(),
        Some(PathBuf::from("a.md")),
        FileTextProfile::detect("# T\n\nx\n", false),
    )
    .unwrap();
    s.set_disk_stamp(Some(stamp("# T\n\nx\n")));
    assert!(!s.profile().had_utf8_bom);
    let disk = "# T\r\n\r\ny\r\n";
    s.reload_from_disk(
        disk.to_string(),
        FileTextProfile::detect(disk, true),
        stamp(disk),
    )
    .unwrap();
    assert!(s.profile().had_utf8_bom, "the file now has a BOM");
    s.undo().unwrap();
    assert!(!s.profile().had_utf8_bom, "undo restores what the text had");
    s.redo().unwrap();
    assert!(s.profile().had_utf8_bom);
}

#[test]
fn a_reload_of_identical_text_writes_no_history_but_records_the_file() {
    let mut s = EditorSession::open(OLD.to_string(), Some(PathBuf::from("a.md"))).unwrap();
    s.set_disk_stamp(Some(stamp("something else, such as a BOM variant")));
    assert!(!s.can_undo());
    reload(&mut s, OLD);
    assert!(
        !s.can_undo(),
        "nothing changed, so there is nothing to undo"
    );
    assert_eq!(s.disk_stamp(), Some(stamp(OLD)));
    assert!(!s.is_dirty());
}

#[test]
fn reload_returns_focus_to_the_outline_and_raises_no_structure_notice() {
    let mut s = session_with_unsaved_edit();
    reload(&mut s, NEW_ON_DISK);
    assert!(s.structure_change().is_none());
    assert!(!s.raw_editing());
}

#[test]
fn a_reload_works_on_text_with_multibyte_characters_and_crlf_at_the_seams() {
    let old = "# \u{65e5}\u{672c}\r\n\r\ncaf\u{e9} one\r\n";
    let new = "# \u{65e5}\u{672c}\r\n\r\ncaf\u{e8} two\r\n";
    let mut s = EditorSession::open(old.to_string(), Some(PathBuf::from("a.md"))).unwrap();
    s.set_disk_stamp(Some(stamp(old)));
    reload(&mut s, new);
    assert_eq!(s.source(), new);
    s.undo().unwrap();
    assert_eq!(s.source(), old);
}

#[test]
fn equal_bytes_have_equal_stamps_and_a_one_byte_change_does_not() {
    assert_eq!(ContentStamp::of(b"abc"), ContentStamp::of(b"abc"));
    assert_ne!(ContentStamp::of(b"abc"), ContentStamp::of(b"abd"));
    assert_ne!(ContentStamp::of(b""), ContentStamp::of(b"\0"));
    // Same length, different bytes: the case a size-and-time check cannot see.
    assert_ne!(
        ContentStamp::of(b"0123456789"),
        ContentStamp::of(b"0123456780")
    );
}
