//! Native file dialogs, disk I/O, and file integrity helpers (RFC-001, RFC-015,
//! RFC-018, RFC-039).
//!
//! RFC-039: OpenOutcome::Failed now carries a human-readable `cause` string so
//! the UI can display *why* a file could not be opened (permission denied, not
//! valid UTF-8, etc.) in an actionable error message.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use omriss_ui::FileTextProfile;

const MD_EXTENSIONS: &[&str] = &["md", "markdown", "mdown", "txt"];
const JSON_EXTENSIONS: &[&str] = &["json"];

// ── outcome types ─────────────────────────────────────────────────────────────

/// Result of asking the user to open a Markdown file.
pub enum OpenOutcome {
    /// The user dismissed the dialog or no path was provided.
    Cancelled,
    /// File could not be read; `cause` is a plain-language reason (RFC-039).
    Failed { cause: String },
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
    /// Write failed; in-memory state is untouched.
    Failed,
    /// Path written, new disk mtime recorded.
    Saved {
        name: String,
        mtime: Option<SystemTime>,
    },
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// The shared open/save dialog. Both filters are always offered: RFC-054 J3
/// widens the *open* side to `.json` (RFC-052 §14.4, JSON visible by
/// default — no separate "experimental formats" toggle), and it costs
/// nothing to offer the same two filters on the Save-As side too, rather
/// than splitting into a second dialog-builder function neither call site
/// actually needs. Save itself has no JSON-specific behavior yet — nothing
/// can edit a JSON document until RFC-054 J5/J6 — so writing one back out
/// unmodified through this dialog is the only JSON case it can reach today,
/// and that already worked before this slice (the bytes are untouched).
fn markdown_dialog() -> rfd::FileDialog {
    rfd::FileDialog::new()
        .add_filter("Markdown", MD_EXTENSIONS)
        .add_filter("JSON", JSON_EXTENSIONS)
}

/// Reads `path`, strips a UTF-8 BOM if present, and returns text + profile +
/// mtime, or a descriptive error string for RFC-039 error messages.
fn read_markdown(path: &Path) -> Result<(String, FileTextProfile, Option<SystemTime>), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());

    let (text_bytes, had_bom) = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (&bytes[3..], true)
    } else {
        (&bytes[..], false)
    };

    let text = String::from_utf8(text_bytes.to_vec()).map_err(|_| {
        "File is not valid UTF-8. Omriss requires UTF-8 Markdown files.".to_string()
    })?;
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
pub fn open_markdown() -> OpenOutcome {
    let Some(path) = markdown_dialog().pick_file() else {
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
            cause: format!("File not found: {path_str}"),
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
        Err(cause) => OpenOutcome::Failed { cause },
    }
}

/// Writes `source` to the session's existing path, or prompts for one.
pub fn save_markdown(
    existing: Option<&str>,
    source: &str,
    profile: &FileTextProfile,
) -> SaveOutcome {
    let target = match existing {
        Some(name) => Some(PathBuf::from(name)),
        None => markdown_dialog().save_file(),
    };
    let Some(path) = target else {
        return SaveOutcome::Cancelled;
    };
    match write_atomic(&path, source, profile) {
        Ok(mtime) => SaveOutcome::Saved {
            name: path.display().to_string(),
            mtime: Some(mtime),
        },
        Err(_) => SaveOutcome::Failed,
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
