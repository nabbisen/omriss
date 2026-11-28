//! RFC-068 §5, §7.1–§7.4: the durable write, from real files and forced failures.
//!
//! The steps that cannot be observed from outside (a power cut) are pinned by their
//! order and by what is true at each one; the rest are real filesystem behaviour.

use std::cell::RefCell;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{Seams, Step, unique_suffix, write_durably, write_durably_with};

/// A temp directory for one test, removed when it ends, even on panic.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("omriss-durable-{}-{label}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // Canonical, so that comparing paths does not trip over a symlinked temp dir.
        Scratch(fs::canonicalize(dir).unwrap())
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // A test may leave a directory read-only on purpose.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The names in `dir`, sorted: what a save left behind.
fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn fail_at(step: Step) -> impl Fn(Step, &Path) -> io::Result<()> {
    move |s, _| {
        if s == step {
            Err(io::Error::other("injected"))
        } else {
            Ok(())
        }
    }
}

fn write_with_failure(target: &Path, bytes: &[u8], step: Step) -> io::Result<()> {
    write_durably_with(
        target,
        bytes,
        &Seams {
            before: &fail_at(step),
            suffix: &unique_suffix,
        },
    )
}

/// The steps a write goes through here, in order, for a target that exists.
fn steps_for_existing() -> Vec<Step> {
    let mut v = vec![Step::CreateTemp];
    if cfg!(unix) {
        v.push(Step::SetMode);
    }
    v.extend([Step::WriteBytes, Step::SyncFile, Step::Rename]);
    if cfg!(unix) {
        v.push(Step::SyncDir);
    }
    v
}

// ── the plain cases ───────────────────────────────────────────────────────────

#[test]
fn a_new_file_is_created_and_nothing_else_is_left() {
    let dir = Scratch::new("new");
    let target = dir.path().join("a.md");
    write_durably(&target, b"hello").unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"hello");
    assert_eq!(names(dir.path()), ["a.md"]);
}

#[test]
fn an_existing_file_is_replaced_whole_and_nothing_else_is_left() {
    let dir = Scratch::new("replace");
    let target = dir.path().join("a.md");
    fs::write(&target, "a much longer first version of the text\n").unwrap();
    write_durably(&target, b"short").unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"short");
    assert_eq!(names(dir.path()), ["a.md"]);
}

#[test]
fn empty_bytes_make_an_empty_file() {
    let dir = Scratch::new("empty");
    let target = dir.path().join("a.md");
    fs::write(&target, "x").unwrap();
    write_durably(&target, b"").unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"");
}

#[test]
fn the_steps_run_in_the_durable_order() {
    let dir = Scratch::new("order");
    let target = dir.path().join("a.md");
    fs::write(&target, "old").unwrap();
    let seen = RefCell::new(Vec::new());
    write_durably_with(
        &target,
        b"new",
        &Seams {
            before: &|s, _| {
                seen.borrow_mut().push(s);
                Ok(())
            },
            suffix: &unique_suffix,
        },
    )
    .unwrap();
    // The bytes are flushed to the device BEFORE the rename (otherwise the name can
    // point at data that never arrived), and the directory is synced AFTER it.
    assert_eq!(*seen.borrow(), steps_for_existing());
}

#[test]
fn a_new_file_has_no_mode_step() {
    let dir = Scratch::new("order-new");
    let seen = RefCell::new(Vec::new());
    write_durably_with(
        &dir.path().join("a.md"),
        b"x",
        &Seams {
            before: &|s, _| {
                seen.borrow_mut().push(s);
                Ok(())
            },
            suffix: &unique_suffix,
        },
    )
    .unwrap();
    assert!(!seen.borrow().contains(&Step::SetMode));
    assert_eq!(seen.borrow().first(), Some(&Step::CreateTemp));
}

// ── a failure at any step ─────────────────────────────────────────────────────

#[test]
fn a_failure_at_any_step_leaves_no_temp_file_and_the_old_file_intact() {
    for step in steps_for_existing() {
        let dir = Scratch::new("fail-existing");
        let target = dir.path().join("a.md");
        fs::write(&target, "old").unwrap();
        let result = write_with_failure(&target, b"new", step);
        assert!(result.is_err(), "{step:?} was meant to fail");
        // The rename is the only step after which the new bytes are the file.
        let expected: &[u8] = if step == Step::SyncDir {
            b"new"
        } else {
            b"old"
        };
        assert_eq!(
            fs::read(&target).unwrap(),
            expected,
            "target after {step:?}"
        );
        assert_eq!(names(dir.path()), ["a.md"], "litter after {step:?}");
    }
}

#[test]
fn a_failure_at_any_step_of_a_new_file_leaves_nothing_behind() {
    for step in steps_for_existing() {
        if step == Step::SetMode {
            continue; // only an existing file has its mode carried over
        }
        let dir = Scratch::new("fail-new");
        let target = dir.path().join("a.md");
        assert!(write_with_failure(&target, b"new", step).is_err());
        let expected: Vec<&str> = if step == Step::SyncDir {
            vec!["a.md"]
        } else {
            vec![]
        };
        assert_eq!(names(dir.path()), expected, "after {step:?}");
    }
}

#[test]
fn a_real_failure_a_directory_where_the_file_should_be_leaves_nothing() {
    let dir = Scratch::new("isdir");
    let target = dir.path().join("a.md");
    fs::create_dir(&target).unwrap();
    let error = write_durably(&target, b"x").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::IsADirectory);
    assert_eq!(names(dir.path()), ["a.md"]);
    assert!(target.is_dir());
}

#[test]
fn a_real_failure_a_missing_folder_is_not_found() {
    let dir = Scratch::new("nofolder");
    let target = dir.path().join("missing").join("a.md");
    let error = write_durably(&target, b"x").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(names(dir.path()).is_empty());
}

#[cfg(unix)]
#[test]
fn a_real_failure_a_folder_that_cannot_be_written_leaves_the_file_and_no_temp() {
    use std::os::unix::fs::PermissionsExt;
    let dir = Scratch::new("readonly");
    let target = dir.path().join("a.md");
    fs::write(&target, "old").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
    if fs::File::create(dir.path().join("probe")).is_ok() {
        return; // running as root: permissions do not stop us, nothing to prove
    }
    let error = write_durably(&target, b"new").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(fs::read(&target).unwrap(), b"old");
    assert_eq!(names(dir.path()), ["a.md"]);
}

// ── temporary names ───────────────────────────────────────────────────────────

#[test]
fn the_same_stem_with_another_extension_never_shares_a_temp_name() {
    let dir = Scratch::new("stem");
    let seen = RefCell::new(Vec::<OsString>::new());
    for file in ["notes.md", "notes.txt", "notes.markdown", "notes"] {
        write_durably_with(
            &dir.path().join(file),
            b"x",
            &Seams {
                before: &|s, p| {
                    if s == Step::CreateTemp {
                        seen.borrow_mut().push(p.file_name().unwrap().to_owned());
                    }
                    Ok(())
                },
                // The SAME suffix for every write: only the name can tell them apart,
                // which is the old code's collision (`with_extension` made one name).
                suffix: &|| "same".to_string(),
            },
        )
        .unwrap();
    }
    let seen = seen.borrow();
    for (i, a) in seen.iter().enumerate() {
        for b in &seen[i + 1..] {
            assert_ne!(a, b);
        }
    }
    assert!(seen[0].to_string_lossy().contains("notes.md"));
}

#[test]
fn saves_of_same_stem_files_from_several_threads_do_not_collide() {
    let dir = Scratch::new("threads");
    let root = dir.path().to_path_buf();
    let handles: Vec<_> = ["notes.md", "notes.txt", "notes.markdown", "notes.mdown"]
        .into_iter()
        .map(|file| {
            let target = root.join(file);
            std::thread::spawn(move || {
                for round in 0..50 {
                    write_durably(&target, format!("{file} {round}").as_bytes()).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    for file in ["notes.md", "notes.txt", "notes.markdown", "notes.mdown"] {
        assert_eq!(
            fs::read(root.join(file)).unwrap(),
            format!("{file} 49").as_bytes()
        );
    }
    assert_eq!(
        names(&root).len(),
        4,
        "temporary files left: {:?}",
        names(&root)
    );
}

#[test]
fn a_file_already_at_the_temp_name_is_never_overwritten() {
    let dir = Scratch::new("occupied");
    let target = dir.path().join("a.md");
    fs::write(&target, "old").unwrap();
    let taken = dir.path().join(".a.md.omriss-taken.tmp");
    fs::write(&taken, "precious").unwrap();

    // The first two names offered are taken; the third is free.
    let offered = RefCell::new(vec!["fresh", "taken", "taken"]);
    write_durably_with(
        &target,
        b"new",
        &Seams {
            before: &|_, _| Ok(()),
            suffix: &|| offered.borrow_mut().pop().unwrap().to_string(),
        },
    )
    .unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new");
    assert_eq!(fs::read(&taken).unwrap(), b"precious");
    assert_eq!(names(dir.path()), [".a.md.omriss-taken.tmp", "a.md"]);
}

#[test]
fn when_every_name_is_taken_it_fails_without_touching_anything() {
    let dir = Scratch::new("all-taken");
    let target = dir.path().join("a.md");
    fs::write(&target, "old").unwrap();
    let taken = dir.path().join(".a.md.omriss-taken.tmp");
    fs::write(&taken, "precious").unwrap();
    let error = write_durably_with(
        &target,
        b"new",
        &Seams {
            before: &|_, _| Ok(()),
            suffix: &|| "taken".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&target).unwrap(), b"old");
    assert_eq!(fs::read(&taken).unwrap(), b"precious");
}

#[test]
fn suffixes_differ_between_calls() {
    assert_ne!(unique_suffix(), unique_suffix());
}

// ── Unix: permissions, symlinks, the directory ───────────────────────────────

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o7777
    }

    #[test]
    fn the_mode_of_the_target_is_kept() {
        for original in [0o600, 0o640, 0o644, 0o755, 0o444] {
            let dir = Scratch::new("mode");
            let target = dir.path().join("a.md");
            fs::write(&target, "old").unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(original)).unwrap();
            write_durably(&target, b"new").unwrap();
            assert_eq!(mode(&target), original, "mode {original:o}");
            assert_eq!(fs::read(&target).unwrap(), b"new");
        }
    }

    #[test]
    fn the_content_is_never_readable_by_more_than_the_target_was() {
        // The window the old code had: a temp file at a predictable path, readable by
        // others, holding the content. Now it starts owner-only and has its final bits
        // before the first byte is written.
        let dir = Scratch::new("window");
        let target = dir.path().join("a.md");
        fs::write(&target, "old").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        let seen = RefCell::new(Vec::new());
        write_durably_with(
            &target,
            b"secret new text",
            &Seams {
                before: &|step, temp| {
                    if matches!(step, Step::SetMode | Step::WriteBytes) {
                        let meta = fs::metadata(temp).unwrap();
                        seen.borrow_mut().push((
                            step,
                            meta.permissions().mode() & 0o7777,
                            meta.len(),
                        ));
                    }
                    Ok(())
                },
                suffix: &unique_suffix,
            },
        )
        .unwrap();
        assert_eq!(
            *seen.borrow(),
            [(Step::SetMode, 0o600, 0), (Step::WriteBytes, 0o640, 0)],
            "owner-only at creation, the target's bits before any byte"
        );
    }

    #[test]
    fn a_target_of_0600_is_never_wider_than_0600() {
        let dir = Scratch::new("private");
        let target = dir.path().join("a.md");
        fs::write(&target, "old").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        let widest = RefCell::new(0u32);
        write_durably_with(
            &target,
            b"new",
            &Seams {
                before: &|step, temp| {
                    if step != Step::CreateTemp && step != Step::SyncDir {
                        let m = mode(temp);
                        *widest.borrow_mut() |= m;
                    }
                    Ok(())
                },
                suffix: &unique_suffix,
            },
        )
        .unwrap();
        assert_eq!(
            *widest.borrow() & 0o077,
            0,
            "group/other bits were set on the temp file"
        );
    }

    #[test]
    fn a_new_file_gets_the_mode_any_new_file_gets() {
        let dir = Scratch::new("newmode");
        let plain = dir.path().join("plain");
        fs::File::create(&plain).unwrap();
        let ours = dir.path().join("ours.md");
        write_durably(&ours, b"x").unwrap();
        assert_eq!(mode(&ours), mode(&plain));
    }

    #[test]
    fn a_symlink_survives_and_the_file_it_points_to_is_updated() {
        let dir = Scratch::new("link");
        let vault = dir.path().join("vault");
        let home = dir.path().join("home");
        fs::create_dir(&vault).unwrap();
        fs::create_dir(&home).unwrap();
        let real = vault.join("notes.md");
        fs::write(&real, "old").unwrap();
        fs::set_permissions(&real, fs::Permissions::from_mode(0o640)).unwrap();
        let link = home.join("notes.md");
        symlink(&real, &link).unwrap();

        write_durably(&link, b"new").unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_link(&link).unwrap(), real);
        assert_eq!(fs::read(&real).unwrap(), b"new");
        assert_eq!(mode(&real), 0o640);
        assert_eq!(
            names(&vault),
            ["notes.md"],
            "temp file beside the real file, removed"
        );
        assert_eq!(names(&home), ["notes.md"]);
    }

    #[test]
    fn a_relative_link_a_chain_of_links_and_a_linked_folder_are_all_followed() {
        let dir = Scratch::new("chain");
        let vault = dir.path().join("vault");
        fs::create_dir(&vault).unwrap();
        let real = vault.join("n.md");
        fs::write(&real, "old").unwrap();
        symlink("vault/n.md", dir.path().join("rel.md")).unwrap();
        symlink("rel.md", dir.path().join("chain.md")).unwrap();
        symlink("vault", dir.path().join("folder")).unwrap();

        write_durably(&dir.path().join("chain.md"), b"one").unwrap();
        assert_eq!(fs::read(&real).unwrap(), b"one");
        write_durably(&dir.path().join("folder").join("n.md"), b"two").unwrap();
        assert_eq!(fs::read(&real).unwrap(), b"two");

        for link in ["rel.md", "chain.md", "folder"] {
            assert!(
                fs::symlink_metadata(dir.path().join(link))
                    .unwrap()
                    .file_type()
                    .is_symlink(),
                "{link} is no longer a link"
            );
        }
        assert_eq!(names(&vault), ["n.md"]);
    }

    #[test]
    fn a_link_to_a_file_that_does_not_exist_yet_creates_it_and_keeps_the_link() {
        let dir = Scratch::new("dangling");
        let link = dir.path().join("link.md");
        symlink("made.md", &link).unwrap();
        write_durably(&link, b"new").unwrap();
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(dir.path().join("made.md")).unwrap(), b"new");
    }

    #[test]
    fn a_link_into_a_missing_folder_fails_and_changes_nothing() {
        let dir = Scratch::new("dangling-folder");
        let link = dir.path().join("link.md");
        symlink("nowhere/made.md", &link).unwrap();
        let error = write_durably(&link, b"new").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(names(dir.path()), ["link.md"]);
    }

    #[test]
    fn a_loop_of_links_fails_and_changes_nothing() {
        let dir = Scratch::new("loop");
        symlink("b.md", dir.path().join("a.md")).unwrap();
        symlink("a.md", dir.path().join("b.md")).unwrap();
        assert!(write_durably(&dir.path().join("a.md"), b"x").is_err());
        assert_eq!(names(dir.path()), ["a.md", "b.md"]);
    }

    #[test]
    fn only_a_device_error_fails_a_directory_sync() {
        use io::ErrorKind::*;
        for kind in [Unsupported, InvalidInput, PermissionDenied] {
            assert!(super::super::dir_sync_not_possible(kind), "{kind:?}");
        }
        for kind in [Other, UnexpectedEof, StorageFull, NotFound] {
            assert!(!super::super::dir_sync_not_possible(kind), "{kind:?}");
        }
    }
}

// ── the calls themselves, in order ────────────────────────────────────────────

/// The hook-order test above watches the announcements, which a refactor could keep
/// while dropping the call they announce. This reads the function and requires each
/// step's announcement to come right before its real call, and the calls in the
/// durable order: bytes are flushed (`sync_all`) before the rename, the directory is
/// synced after it.
#[test]
fn each_real_call_follows_its_announcement_in_the_durable_order() {
    let src = include_str!("../durable_write.rs");
    let start = src.find("pub(crate) fn write_durably_with").unwrap();
    let end = src[start..].find("/// Removes the temporary file").unwrap() + start;
    let body = &src[start..end];
    let in_order = [
        "(seams.before)(Step::SetMode",
        ".set_permissions(",
        "(seams.before)(Step::WriteBytes",
        ".write_all(bytes)",
        "(seams.before)(Step::SyncFile",
        ".sync_all()",
        "(seams.before)(Step::Rename",
        "fs::rename(&temp, &resolved)",
        "guard.armed = false",
        "(seams.before)(Step::SyncDir",
        "sync_dir(dir)",
    ];
    let mut at = 0;
    for token in in_order {
        match body[at..].find(token) {
            Some(i) => at += i + token.len(),
            None => panic!("`{token}` is missing, or out of order, in write_durably_with"),
        }
    }
    assert!(
        body.contains("resolve(target)"),
        "the target must be resolved (symlinks) first"
    );
}

#[cfg(unix)]
#[test]
fn a_relative_path_new_or_existing_is_saved_where_it_points() {
    // `omriss notes.md` hands the layer a relative path. The test's working directory
    // is shared by every test, so it is not changed: the path is made relative to it.
    let dir = Scratch::new("relative");
    let cwd = std::env::current_dir().unwrap();
    let up: PathBuf = cwd.components().skip(1).map(|_| "..").collect();
    let relative_dir = up.join(dir.path().strip_prefix("/").unwrap());
    assert!(relative_dir.is_relative());

    let existing = relative_dir.join("old.md");
    fs::write(dir.path().join("old.md"), "old").unwrap();
    write_durably(&existing, b"new").unwrap();
    assert_eq!(fs::read(dir.path().join("old.md")).unwrap(), b"new");

    write_durably(&relative_dir.join("fresh.md"), b"made").unwrap();
    assert_eq!(fs::read(dir.path().join("fresh.md")).unwrap(), b"made");
    assert_eq!(names(dir.path()), ["fresh.md", "old.md"]);
}

#[test]
fn a_bare_file_name_that_does_not_exist_resolves_into_the_working_directory() {
    // The branch `omriss new.md` takes; resolved only, nothing is written.
    let resolved = super::resolve(Path::new("omriss-bare-name-that-does-not-exist.md")).unwrap();
    assert_eq!(
        resolved,
        fs::canonicalize(std::env::current_dir().unwrap())
            .unwrap()
            .join("omriss-bare-name-that-does-not-exist.md")
    );
}
