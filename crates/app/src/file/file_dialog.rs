//! Native file dialogs, disk I/O, and file integrity helpers (RFC-001, RFC-015,
//! RFC-018, RFC-039).
//!
//! RFC-039 / RFC-073: a failure to open or save is a typed reason
//! ([`OpenFailure`], an `io::ErrorKind`), never a string built here: this layer has
//! no locale, so the shell says it in words (RFC-073 §3.2).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use omriss_ui::FileTextProfile;

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
    /// Text (BOM stripped), display name, profile, and disk mtime.
    Loaded {
        text: String,
        name: String,
        profile: FileTextProfile,
        mtime: Option<SystemTime>,
    },
}

/// Result of saving the canonical source text.
pub enum SaveOutcome {
    /// The user dismissed the Save As dialog.
    Cancelled,
    /// Write failed; in-memory state is untouched. `kind` is why (the operating
    /// system's own text is dropped).
    Failed { kind: std::io::ErrorKind },
    /// Path written, new disk mtime recorded.
    Saved {
        name: String,
        mtime: Option<SystemTime>,
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

/// Reads `path`, strips a UTF-8 BOM if present, and returns text + profile +
/// mtime, or the typed reason it could not.
fn read_markdown(
    path: &Path,
) -> Result<(String, FileTextProfile, Option<SystemTime>), OpenFailure> {
    let bytes = fs::read(path).map_err(|e| OpenFailure::from_io(path, &e))?;
    let mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());

    let (text_bytes, had_bom) = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (&bytes[3..], true)
    } else {
        (&bytes[..], false)
    };

    let text = String::from_utf8(text_bytes.to_vec()).map_err(|_| OpenFailure::NotUtf8)?;
    let profile = FileTextProfile::detect(&text, had_bom);
    Ok((text, profile, mtime))
}

/// Writes `source` atomically: writes a temp file then renames (NFR-REL-003).
fn write_atomic(
    path: &Path,
    source: &str,
    profile: &FileTextProfile,
) -> Result<SystemTime, std::io::Error> {
    let temp = path.with_extension("tmp.omriss");
    if profile.had_utf8_bom {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(source.as_bytes());
        fs::write(&temp, &bytes)?;
    } else {
        fs::write(&temp, source)?;
    }
    fs::rename(&temp, path)?;
    let mtime = fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    Ok(mtime)
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
/// Used by the recent-files list (RFC-036).
pub fn open_markdown_path(path_str: &str) -> OpenOutcome {
    let path = PathBuf::from(path_str);
    if !path.exists() {
        return OpenOutcome::Failed {
            failure: OpenFailure::NotFound {
                path: path_str.to_string(),
            },
        };
    }
    open_path(&path)
}

fn open_path(path: &Path) -> OpenOutcome {
    match read_markdown(path) {
        Ok((text, profile, mtime)) => OpenOutcome::Loaded {
            text,
            name: path.display().to_string(),
            profile,
            mtime,
        },
        Err(failure) => OpenOutcome::Failed { failure },
    }
}

/// Writes `source` to the session's existing path, or prompts for one.
pub fn save_markdown(
    existing: Option<&str>,
    source: &str,
    profile: &FileTextProfile,
    labels: &DialogLabels,
) -> SaveOutcome {
    let target = match existing {
        Some(name) => Some(PathBuf::from(name)),
        None => markdown_dialog(labels).save_file(),
    };
    let Some(path) = target else {
        return SaveOutcome::Cancelled;
    };
    match write_atomic(&path, source, profile) {
        Ok(mtime) => SaveOutcome::Saved {
            name: path.display().to_string(),
            mtime: Some(mtime),
        },
        Err(error) => SaveOutcome::Failed { kind: error.kind() },
    }
}

/// Returns `true` if the file was modified externally since `saved_mtime`.
pub fn was_modified_externally(path: &str, saved_mtime: SystemTime) -> bool {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|disk_mtime| disk_mtime > saved_mtime)
        .unwrap_or(false)
}

#[cfg(test)]
mod failure_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod round_trip_tests;
