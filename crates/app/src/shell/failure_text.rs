//! Failures in words (RFC-073 §8): the file layer says what went wrong as data
//! ([`OpenFailure`], an `io::ErrorKind`); this is where a locale is in scope and
//! the catalog is read.
//!
//! Each message says what happened and what to do, and never blames the user
//! (RFC-080 §5.4 rule 8). None carries operating-system text: the screen and the
//! dialog's accessible description (`aria-describedby`) say only what is here
//! (RFC-060 §5, F-5).

use std::io::ErrorKind;

use omriss_ui::i18n::{Locale, t, t_args};

use crate::file::failure::{IoReason, OpenFailure};

/// The message under an open failure's title.
pub(crate) fn open_failure_message(locale: Locale, failure: &OpenFailure) -> String {
    match failure {
        OpenFailure::NotFound { path } => t_args(locale, "error.open.not_found", &[("path", path)]),
        OpenFailure::NotUtf8 => t(locale, "error.open.not_utf8").to_string(),
        OpenFailure::Io { kind } => t(locale, open_io_key(*kind)).to_string(),
        OpenFailure::Internal => t(locale, "error.open.internal").to_string(),
        OpenFailure::CommandLineOption => t(locale, "error.open.command_line_option").to_string(),
    }
}

/// The catalog key for a read failure of this kind. Exhaustive over [`IoReason`]
/// with no wildcard: a new reason is a compile error here.
pub(crate) fn open_io_key(kind: ErrorKind) -> &'static str {
    match IoReason::from_kind(kind) {
        IoReason::PermissionDenied => "error.open.io.permission_denied",
        IoReason::IsADirectory => "error.open.io.is_a_directory",
        // `NotFound` never arrives here (`OpenFailure::from_io` makes it
        // `NotFound { path }`); the others cannot come from a read. All are the
        // general message, named so the map reads as complete.
        IoReason::NotFound
        | IoReason::StorageFull
        | IoReason::ReadOnlyFilesystem
        | IoReason::Other => "error.open.io.general",
    }
}

/// The status message (a catalog key) for a save that failed with `kind`. It
/// says why where it can, and that the writing is safe. Exhaustive, no wildcard.
pub(crate) fn save_failure_key(kind: ErrorKind) -> &'static str {
    match IoReason::from_kind(kind) {
        IoReason::PermissionDenied => "error.save.permission_denied",
        IoReason::StorageFull => "error.save.storage_full",
        IoReason::ReadOnlyFilesystem => "error.save.read_only",
        IoReason::NotFound => "error.save.folder_missing",
        // A directory where the file should be, and every other kind.
        IoReason::IsADirectory | IoReason::Other => "error.save_failed",
    }
}

#[cfg(test)]
mod dialog_tests;
#[cfg(test)]
mod tests;
