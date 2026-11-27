//! RFC-073 §8.1–§8.3: every failure in words, in both locales, and none of them
//! the operating system's.

use std::io::{Error, ErrorKind};
use std::path::Path;

use omriss_ui::i18n::Locale;

use super::{open_failure_message, open_io_key, save_failure_key};
use crate::file::failure::{IoReason, OpenFailure};

const LOCALES: [Locale; 2] = [Locale::En, Locale::Ja];

fn every_open_failure() -> Vec<OpenFailure> {
    let mut v = vec![
        OpenFailure::NotFound {
            path: "/tmp/a file.md".into(),
        },
        OpenFailure::NotUtf8,
        OpenFailure::Internal,
        OpenFailure::CommandLineOption,
    ];
    for kind in [
        ErrorKind::PermissionDenied,
        ErrorKind::IsADirectory,
        ErrorKind::TimedOut,
        ErrorKind::Other,
        ErrorKind::UnexpectedEof,
    ] {
        v.push(OpenFailure::Io { kind });
    }
    v
}

#[test]
fn every_open_failure_has_a_message_in_both_locales_with_no_leftover_placeholder() {
    for locale in LOCALES {
        for failure in every_open_failure() {
            let text = open_failure_message(locale, &failure);
            assert!(
                !text.is_empty() && !text.contains('{'),
                "{failure:?}: {text}"
            );
            assert!(!text.starts_with("error."), "a key, not a message: {text}");
        }
    }
}

#[test]
fn no_operating_system_text_reaches_a_message() {
    // What the OS says for each kind, and what `Error` prints: none of it may appear.
    for kind in [
        ErrorKind::PermissionDenied,
        ErrorKind::IsADirectory,
        ErrorKind::TimedOut,
        ErrorKind::Other,
    ] {
        let os_text = Error::from(kind).to_string();
        let os_raw = Error::from_raw_os_error(13).to_string();
        for locale in LOCALES {
            let text = open_failure_message(locale, &OpenFailure::Io { kind });
            assert!(!text.contains(&os_text), "{kind:?}: {text}");
            assert!(
                !text.contains(&os_raw) && !text.contains("os error"),
                "{text}"
            );
        }
    }
}

#[test]
fn the_path_is_data_and_is_interpolated() {
    let failure = OpenFailure::NotFound {
        path: "/home/u/{x}/notes.md".into(),
    };
    for locale in LOCALES {
        let text = open_failure_message(locale, &failure);
        assert!(text.contains("/home/u/{x}/notes.md"), "{text}");
    }
}

#[test]
fn a_missing_file_is_not_found_however_it_was_discovered() {
    let e = Error::from(ErrorKind::NotFound);
    assert_eq!(
        OpenFailure::from_io(Path::new("/x/y.md"), &e),
        OpenFailure::NotFound {
            path: "/x/y.md".into()
        }
    );
    assert_eq!(
        OpenFailure::from_io(
            Path::new("/x/y.md"),
            &Error::from(ErrorKind::PermissionDenied)
        ),
        OpenFailure::Io {
            kind: ErrorKind::PermissionDenied
        }
    );
}

#[test]
fn the_kinds_a_desktop_user_meets_have_their_own_words_and_the_rest_one_general_message() {
    assert_eq!(
        open_io_key(ErrorKind::PermissionDenied),
        "error.open.io.permission_denied"
    );
    assert_eq!(
        open_io_key(ErrorKind::IsADirectory),
        "error.open.io.is_a_directory"
    );
    assert_eq!(open_io_key(ErrorKind::TimedOut), "error.open.io.general");
    assert_eq!(open_io_key(ErrorKind::Other), "error.open.io.general");
    assert_eq!(
        IoReason::from_kind(ErrorKind::StorageFull),
        IoReason::StorageFull
    );
}

#[test]
fn every_save_failure_says_why_in_both_locales() {
    use omriss_ui::i18n::t;
    let cases = [
        (ErrorKind::PermissionDenied, "error.save.permission_denied"),
        (ErrorKind::StorageFull, "error.save.storage_full"),
        (ErrorKind::ReadOnlyFilesystem, "error.save.read_only"),
        (ErrorKind::NotFound, "error.save.folder_missing"),
        (ErrorKind::TimedOut, "error.save_failed"),
        (ErrorKind::IsADirectory, "error.save_failed"),
    ];
    for (kind, key) in cases {
        assert_eq!(save_failure_key(kind), key, "{kind:?}");
        for locale in LOCALES {
            let text = t(locale, key);
            assert_ne!(text, key, "{key} is in the catalog");
            assert!(!text.contains("os error"), "{text}");
        }
    }
    // The specific ones say something the general one does not.
    assert_ne!(
        t(Locale::En, "error.save.permission_denied"),
        t(Locale::En, "error.save_failed")
    );
    assert!(t(Locale::En, "error.save.permission_denied").contains("permission"));
    assert!(t(Locale::En, "error.save.storage_full").contains("full"));
}

#[test]
fn the_english_wording_says_utf8_files_not_utf8_markdown_files() {
    let text = open_failure_message(Locale::En, &OpenFailure::NotUtf8);
    assert!(text.contains("UTF-8"));
    assert!(!text.contains("Markdown"), "JSON is opened too: {text}");
}
