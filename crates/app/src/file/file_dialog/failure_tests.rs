//! RFC-073 §8.1: the file layer's failures are typed reasons, from real files.

use std::fs;

use super::{OpenOutcome, open_markdown_path};
use crate::file::failure::OpenFailure;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("omriss-failure-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn failure_of(path: &std::path::Path) -> OpenFailure {
    match open_markdown_path(path.to_str().unwrap()) {
        OpenOutcome::Failed { failure } => failure,
        _ => panic!("expected a failure for {}", path.display()),
    }
}

#[test]
fn a_missing_file_is_not_found_with_its_path() {
    let dir = scratch("missing");
    let path = dir.join("nope.md");
    assert_eq!(
        failure_of(&path),
        OpenFailure::NotFound {
            path: path.display().to_string()
        }
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn bytes_that_are_not_utf8_are_not_utf8() {
    let dir = scratch("latin1");
    let path = dir.join("a.md");
    fs::write(&path, [b'#', b' ', 0xE9, b'\n']).unwrap();
    assert_eq!(failure_of(&path), OpenFailure::NotUtf8);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_directory_is_a_directory_not_an_os_message() {
    let dir = scratch("dir");
    assert_eq!(
        failure_of(&dir),
        OpenFailure::Io {
            kind: std::io::ErrorKind::IsADirectory
        }
    );
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn an_unreadable_file_is_permission_denied() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("chmod");
    let path = dir.join("secret.md");
    fs::write(&path, "# x\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    // Root reads anything: nothing to assert there.
    if fs::read(&path).is_err() {
        assert_eq!(
            failure_of(&path),
            OpenFailure::Io {
                kind: std::io::ErrorKind::PermissionDenied
            }
        );
    }
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let _ = fs::remove_dir_all(dir);
}
