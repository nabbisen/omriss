//! Why a file could not be opened or saved, as data (RFC-073 §4.3, §8).
//!
//! This layer has no `Locale` and must not acquire one (RFC-073 §3.2): it says
//! *what* went wrong, as a typed reason; `shell/failure_text.rs`, where a locale is
//! in scope, says it in words. No operating-system message crosses this
//! boundary: an `io::Error`'s text is for a log, never for a person (RFC-060 §5).

use std::io;
use std::path::Path;

/// A failure to open a file (or to start with the command line given). Only the
/// failures that can happen have a variant, found by construction: every `Err`
/// that reaches `OpenOutcome::Failed` or `Modal::OpenError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenFailure {
    /// No file at `path` (a recent-files entry that moved, a mistyped argument).
    NotFound { path: String },
    /// The bytes are not UTF-8 text.
    NotUtf8,
    /// The operating system refused the read, for a reason other than the file
    /// not being there. The kind decides the words; the OS's own text is dropped.
    Io { kind: io::ErrorKind },
    /// The text was read, but turning it into a document failed: an internal
    /// invariant of the outline (a bug in omriss, never in the user's file). The
    /// user is told nothing was changed, and that this is worth reporting.
    Internal,
    /// The command line gave an option; omriss takes a file path and no options.
    CommandLineOption,
}

impl OpenFailure {
    /// The failure for `error`, met while reading `path`. A missing file is
    /// [`OpenFailure::NotFound`] however it was discovered, so the same words
    /// serve a recent-files entry and a file deleted between two clicks.
    pub fn from_io(path: &Path, error: &io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => OpenFailure::NotFound {
                path: path.display().to_string(),
            },
            kind => OpenFailure::Io { kind },
        }
    }
}

/// The reasons an I/O failure is told apart for a person: the kinds a desktop
/// user actually meets, and one for everything else.
///
/// **The one wildcard.** `io::ErrorKind` is `#[non_exhaustive]` (the standard
/// library adds kinds), so [`IoReason::from_kind`] must have a general arm. It is
/// the only one in the failure maps: every map from a reason to words matches
/// `IoReason` exhaustively, so a new reason is a compile error there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoReason {
    PermissionDenied,
    NotFound,
    IsADirectory,
    StorageFull,
    ReadOnlyFilesystem,
    /// Any other kind.
    Other,
}

impl IoReason {
    pub fn from_kind(kind: io::ErrorKind) -> Self {
        match kind {
            io::ErrorKind::PermissionDenied => IoReason::PermissionDenied,
            io::ErrorKind::NotFound => IoReason::NotFound,
            io::ErrorKind::IsADirectory => IoReason::IsADirectory,
            io::ErrorKind::StorageFull => IoReason::StorageFull,
            io::ErrorKind::ReadOnlyFilesystem => IoReason::ReadOnlyFilesystem,
            // `ErrorKind` is `#[non_exhaustive]`: the one wildcard (see above).
            _ => IoReason::Other,
        }
    }
}
