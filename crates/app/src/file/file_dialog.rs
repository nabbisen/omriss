//! Native file dialogs, disk I/O, and file integrity helpers (RFC-001, RFC-015,
//! RFC-018, RFC-039).
//!
//! RFC-039 / RFC-073: a failure to open or save is a typed reason
//! ([`OpenFailure`], an `io::ErrorKind`), never a string built here: this layer has
//! no locale, so the shell says it in words (RFC-073 §3.2).

use std::fs;
use std::path::{Path, PathBuf};

use omriss_ui::{ContentStamp, FileTextProfile};

use super::durable_write::{self, Durability};
use super::failure::OpenFailure;

const MD_EXTENSIONS: &[&str] = &["md", "markdown", "mdown", "txt"];
const JSON_EXTENSIONS: &[&str] = &["json"];

// ── outcome types ─────────────────────────────────────────────────────────────

/// Result of asking the user to open a Markdown file.
pub enum OpenOutcome {
    /// The user dismissed the dialog or no path was provided.
    Cancelled,
    /// File could not be read, and why.
    Failed { failure: OpenFailure },
    /// Text (BOM stripped), the file's path, profile, and the stamp of the bytes read.
    /// The path is the file's own, byte for byte (RFC-068 §7.5): what saves it again,
    /// not what is shown. The stamp is what a later save compares the file with
    /// (RFC-068 §7.6).
    Loaded {
        text: String,
        path: PathBuf,
        profile: FileTextProfile,
        stamp: ContentStamp,
    },
}

/// Result of saving the canonical source text.
pub enum SaveOutcome {
    /// The user dismissed the Save As dialog.
    Cancelled,
    /// Write failed; in-memory state is untouched. `kind` is why (the operating
    /// system's own text is dropped).
    Failed { kind: std::io::ErrorKind },
    /// Path written, and the stamp of the bytes written. The file holds the new text
    /// whatever `durability` says; `Unconfirmed` is a warning to show, never a failure
    /// (RFC-068 §8.2).
    Saved {
        path: PathBuf,
        stamp: ContentStamp,
        durability: Durability,
    },
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Every extension the open dialog accepts: the union of the per-format
/// lists, in their order. Built from `MD_EXTENSIONS` and `JSON_EXTENSIONS`
/// rather than retyped, so a format added to one list (RFC-055: TOML) cannot
/// be missing from the default filter.
fn all_extensions() -> Vec<&'static str> {
    MD_EXTENSIONS
        .iter()
        .chain(JSON_EXTENSIONS)
        .copied()
        .collect()
}

/// The words on the dialog's filters that are not format names (RFC-073 §4.4b).
/// "Markdown" and "JSON" are what those formats are called in every locale; the
/// default filter's name is a phrase, and it is the first thing a person reads in
/// the dialog. The caller, where a locale is in scope, resolves it: text enters
/// this layer, a locale does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogLabels {
    /// The default filter: every supported extension.
    pub all_supported: String,
}

/// The shared open/save dialog. The first filter is the dialog's default, so
/// it is "all supported": a user opening a `.json` file must see it without
/// finding a filter dropdown (RFC-052 §14.4, JSON visible by default — no
/// separate "experimental formats" toggle). The per-format filters follow for
/// narrowing. RFC-054 J3 widened the *open* side to `.json`, and it costs
/// nothing to offer the same filters on the Save-As side too, rather
/// than splitting into a second dialog-builder function neither call site
/// actually needs. Save itself has no JSON-specific behavior yet — nothing
/// can edit a JSON document until RFC-054 J5/J6 — so writing one back out
/// unmodified through this dialog is the only JSON case it can reach today,
/// and that already worked before this slice (the bytes are untouched).
fn markdown_dialog(labels: &DialogLabels) -> rfd::FileDialog {
    rfd::FileDialog::new()
        .add_filter(&labels.all_supported, &all_extensions())
        .add_filter("Markdown", MD_EXTENSIONS)
        .add_filter("JSON", JSON_EXTENSIONS)
}

/// Reads `path`, strips a UTF-8 BOM if present, and returns text + profile + the
/// stamp of the bytes as read (BOM included), or the typed reason it could not.
fn read_markdown(path: &Path) -> Result<(String, FileTextProfile, ContentStamp), OpenFailure> {
    let mut bytes = fs::read(path).map_err(|e| OpenFailure::from_io(path, &e))?;
    let stamp = ContentStamp::of(&bytes);

    // The text is made from the same allocation the bytes were read into: the BOM is
    // taken off in place and the buffer is validated and moved, never copied. It used to
    // be copied (`to_vec`), so opening held the file twice at once (RFC-068 §7.8).
    let had_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    if had_bom {
        bytes.drain(..3);
    }
    let text = String::from_utf8(bytes).map_err(|_| OpenFailure::NotUtf8)?;
    let profile = FileTextProfile::detect(&text, had_bom);
    Ok((text, profile, stamp))
}

/// How bytes reach the disk: the one durable write in production, a variant of it in
/// the tests that need a step to fail.
type Writer<'a> = &'a dyn Fn(&Path, &[u8]) -> std::io::Result<Durability>;

/// Writes `source` (with its BOM, if the file had one) through `write`
/// ([`durable_write::write_durably`], RFC-068): decided here is what the bytes are,
/// there is how they reach the disk. Returns the stamp of the bytes written and whether
/// the disk confirmed the write.
fn write_atomic(
    path: &Path,
    source: &str,
    profile: &FileTextProfile,
    write: Writer,
) -> Result<(ContentStamp, Durability), std::io::Error> {
    let mut bytes = Vec::with_capacity(source.len() + 3);
    if profile.had_utf8_bom {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    bytes.extend_from_slice(source.as_bytes());
    let durability = write(path, &bytes)?;
    Ok((ContentStamp::of(&bytes), durability))
}

// ── public API ────────────────────────────────────────────────────────────────

/// Shows the open dialog and reads the chosen Markdown file.
pub fn open_markdown(labels: &DialogLabels) -> OpenOutcome {
    let Some(path) = markdown_dialog(labels).pick_file() else {
        return OpenOutcome::Cancelled;
    };
    open_path(&path)
}

/// Opens a Markdown file at a known path without showing a dialog.
/// Used by the recent-files list (RFC-036) and the command line (RFC-063).
pub fn open_markdown_path(path: &Path) -> OpenOutcome {
    if !path.exists() {
        return OpenOutcome::Failed {
            failure: OpenFailure::NotFound {
                path: path.display().to_string(),
            },
        };
    }
    open_path(path)
}

fn open_path(path: &Path) -> OpenOutcome {
    match read_markdown(path) {
        Ok((text, profile, stamp)) => OpenOutcome::Loaded {
            text,
            path: path.to_path_buf(),
            profile,
            stamp,
        },
        Err(failure) => OpenOutcome::Failed { failure },
    }
}

/// Writes `source` to the session's existing path, or prompts for one.
pub fn save_markdown(
    existing: Option<&Path>,
    source: &str,
    profile: &FileTextProfile,
    labels: &DialogLabels,
) -> SaveOutcome {
    let target = match existing {
        Some(path) => Some(path.to_path_buf()),
        None => markdown_dialog(labels).save_file(),
    };
    let Some(path) = target else {
        return SaveOutcome::Cancelled;
    };
    save_to_path(&path, source, profile, &durable_write::write_durably)
}

/// The save itself, past the dialog: what [`save_markdown`] does with a path, with the
/// write passed in so a test can make a step of it fail.
pub(crate) fn save_to_path(
    path: &Path,
    source: &str,
    profile: &FileTextProfile,
    write: Writer,
) -> SaveOutcome {
    match write_atomic(path, source, profile, write) {
        Ok((stamp, durability)) => SaveOutcome::Saved {
            path: path.to_path_buf(),
            stamp,
            durability,
        },
        Err(error) => SaveOutcome::Failed { kind: error.kind() },
    }
}

/// Whether the file at `path` is no longer what this document was made from
/// (RFC-068 §7.6): its bytes, read now, do not have the stamp `known` of the bytes last
/// read from it or written to it. Another program changed it, whatever its timestamp
/// says, and a change that left the same bytes is not a change.
///
/// **On doubt, true**, so the user is asked and nothing is overwritten unseen: a file
/// that has been deleted or cannot be read is not what was opened, and a document with
/// no stamp (not made from this file) cannot be compared.
pub fn changed_on_disk(path: &Path, known: Option<ContentStamp>) -> bool {
    let Some(known) = known else {
        return true;
    };
    match fs::read(path) {
        Ok(bytes) => ContentStamp::of(&bytes) != known,
        Err(_) => true,
    }
}

#[cfg(test)]
mod disk_change_tests;
#[cfg(test)]
mod failure_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod round_trip_tests;
