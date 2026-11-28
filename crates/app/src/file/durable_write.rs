//! The one way omriss puts bytes in a file (RFC-068 §7.1–§7.4).
//!
//! Documents and settings both go through [`write_durably`]; a source-scanning guard
//! (`write_guard_tests.rs`) fails the build if any other code in the application
//! crates calls `fs::write`, `File::create`, `fs::rename` and their kin. What the
//! caller decides (a BOM, a line ending, the bytes) is decided before the call; this
//! module only gets bytes onto disk, and says so when it could not.
//!
//! What a completed save guarantees, per platform (RFC-068 §7.2, §7.9):
//!
//! * **Everywhere.** The new bytes are written to a *new* file beside the target,
//!   flushed to the device (`sync_all`), and only then renamed over it. The target is
//!   never half-written: at every instant it is the old file or the new one. A failure
//!   at any step removes the temporary file and leaves the target as it was.
//! * **Unix.** The directory is also fsynced after the rename, so the rename itself
//!   survives a power cut (without that, the new name can be lost and the *old* file
//!   comes back, or the data is not yet where the name says). The temporary file is
//!   created readable by its owner only and given the target's permission bits *before*
//!   any byte is written, so there is no moment when the content is readable by anyone
//!   the target was not readable by. A symlink is followed: the file it points to is
//!   replaced and the link is untouched.
//! * **Windows.** There is no directory fsync to call, and permissions are the
//!   directory's inherited ones (RFC-068 §4.3); a rename is atomic, but whether it
//!   survives a power cut is the filesystem's, not ours. Symlinks are followed.
//! * **Not preserved, anywhere:** hard links (an atomic rename makes a new file; keeping
//!   the link would mean writing in place, which is the power-loss risk this module
//!   exists to remove), extended attributes, and — when the target belongs to another
//!   user — its owner and group.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// How many unused temporary names are tried before giving up. A name is taken only
/// by something that deliberately sits at it, so one retry is already unusual.
const NAME_ATTEMPTS: u32 = 16;

/// The steps of a write, in order. Tests make one of them fail, and watch the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    CreateTemp,
    /// Unix only, and only when the target exists.
    #[cfg_attr(not(unix), allow(dead_code))]
    SetMode,
    WriteBytes,
    SyncFile,
    Rename,
    /// Unix only.
    #[cfg_attr(not(unix), allow(dead_code))]
    SyncDir,
}

/// What a test can change: a failure before any step, and the temporary name's suffix.
pub(crate) struct Seams<'a> {
    pub before: &'a dyn Fn(Step, &Path) -> io::Result<()>,
    pub suffix: &'a dyn Fn() -> String,
}

/// Writes `bytes` to `target` so that `target` is, at every instant and after any
/// failure, either what it was or exactly `bytes` (see the module doc for the rest).
pub(crate) fn write_durably(target: &Path, bytes: &[u8]) -> io::Result<()> {
    write_durably_with(
        target,
        bytes,
        &Seams {
            before: &|_, _| Ok(()),
            suffix: &unique_suffix,
        },
    )
}

pub(crate) fn write_durably_with(target: &Path, bytes: &[u8], seams: &Seams) -> io::Result<()> {
    let resolved = resolve(target)?;
    if resolved.is_dir() {
        return Err(io::ErrorKind::IsADirectory.into());
    }
    let dir = resolved
        .parent()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    let file_name = resolved
        .file_name()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    // Present when the file exists. On Unix its permission bits are carried over.
    let existing = fs::metadata(&resolved).ok();

    let (mut file, temp) = create_temp(dir, file_name, existing.is_some(), seams)?;
    // From here on, the temporary file is removed on every way out but one: the rename.
    let mut guard = TempGuard {
        path: temp.clone(),
        armed: true,
    };

    #[cfg(unix)]
    if let Some(meta) = &existing {
        use std::os::unix::fs::PermissionsExt;
        (seams.before)(Step::SetMode, &temp)?;
        file.set_permissions(fs::Permissions::from_mode(
            meta.permissions().mode() & 0o7777,
        ))?;
    }
    #[cfg(not(unix))]
    let _ = &existing;

    (seams.before)(Step::WriteBytes, &temp)?;
    file.write_all(bytes)?;
    (seams.before)(Step::SyncFile, &temp)?;
    file.sync_all()?;
    drop(file);

    (seams.before)(Step::Rename, &temp)?;
    fs::rename(&temp, &resolved)?;
    guard.armed = false; // the temporary file is now the target

    #[cfg(unix)]
    {
        (seams.before)(Step::SyncDir, dir)?;
        sync_dir(dir)?;
    }
    Ok(())
}

/// Removes the temporary file unless the rename has happened.
struct TempGuard {
    path: PathBuf,
    armed: bool,
}

impl Drop for TempGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Where the bytes really go: the file `path` names, following every symlink, so a
/// link is left as it is and the file it points to is replaced. A path that does not
/// exist yet is a new file in its (existing) folder; a symlink whose target does not
/// exist yet resolves to that target, as `echo > link` would create it.
fn resolve(path: &Path) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(real) => Ok(real),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let mut current = path.to_path_buf();
            for _ in 0..40 {
                match fs::symlink_metadata(&current) {
                    Ok(meta) if meta.file_type().is_symlink() => {
                        let link = fs::read_link(&current)?;
                        current = if link.is_absolute() {
                            link
                        } else {
                            current.parent().unwrap_or(Path::new("")).join(link)
                        };
                    }
                    // It exists, yet could not be resolved: not ours to guess at.
                    Ok(_) => return Err(e),
                    Err(m) if m.kind() == io::ErrorKind::NotFound => {
                        let name = current
                            .file_name()
                            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?
                            .to_owned();
                        let parent = match current.parent() {
                            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
                            _ => PathBuf::from("."),
                        };
                        return Ok(fs::canonicalize(parent)?.join(name));
                    }
                    Err(m) => return Err(m),
                }
            }
            Err(e)
        }
        Err(e) => Err(e),
    }
}

/// Creates the temporary file beside the target: a name no other file has (made from
/// the target's *whole* name, so `notes.md` and `notes.txt` cannot meet), created with
/// create-new semantics, so an existing file is never reused or overwritten.
fn create_temp(
    dir: &Path,
    file_name: &std::ffi::OsStr,
    target_exists: bool,
    seams: &Seams,
) -> io::Result<(File, PathBuf)> {
    for _ in 0..NAME_ATTEMPTS {
        let mut name = OsString::from(".");
        name.push(file_name);
        name.push(format!(".omriss-{}.tmp", (seams.suffix)()));
        let temp = dir.join(name);

        (seams.before)(Step::CreateTemp, &temp)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if target_exists {
            // Owner-only until the target's own bits are set, before any byte is written.
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(not(unix))]
        let _ = target_exists;
        match options.open(&temp) {
            Ok(file) => return Ok((file, temp)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::ErrorKind::AlreadyExists.into())
}

/// Makes the rename itself durable. A filesystem that cannot fsync a directory
/// (`Unsupported`, `InvalidInput`) or a folder that cannot be opened for it
/// (`PermissionDenied`) is not a failed save: the file is written and renamed, and
/// there is nothing stronger to ask. Anything else (a device error) is reported.
#[cfg(unix)]
fn sync_dir(dir: &Path) -> io::Result<()> {
    match File::open(dir).and_then(|d| d.sync_all()) {
        Err(e) if dir_sync_not_possible(e.kind()) => Ok(()),
        other => other,
    }
}

/// The kinds that mean "this directory cannot be fsynced", as opposed to "the device
/// failed" (see [`sync_dir`]).
#[cfg(unix)]
fn dir_sync_not_possible(kind: io::ErrorKind) -> bool {
    use io::ErrorKind::{InvalidInput, PermissionDenied, Unsupported};
    matches!(kind, Unsupported | InvalidInput | PermissionDenied)
}

/// A suffix no two writes of this process share, and no two processes are likely to:
/// process id, a counter, and the clock. It names a temporary file, not a secret.
fn unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!(
        "{}-{}-{nanos:x}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod write_guard_tests;
