//! Shared by the source-scanning guard tests (`commit_path_tests`,
//! `focus_path_tests`): walk the UI layer's sources and report where a
//! forbidden call appears outside the one module that may make it.
//!
//! Scope: `crates/components/src` and `crates/app/src`. Test code is exempt —
//! files named `tests.rs` / `*_tests.rs`, anything under a directory named
//! `tests`, and everything after a `#[cfg(test)]` line, by the repository's
//! convention that test modules close a file. Comment lines are ignored.

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Test code, by file name or location.
pub(crate) fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    name == "tests.rs"
        || name.ends_with("_tests.rs")
        || path.components().any(|c| c.as_os_str() == "tests")
}

/// Non-comment, non-test code of `text`, as `(line number, line)`.
pub(crate) fn production_code(text: &str) -> Vec<(usize, &str)> {
    text.lines()
        .enumerate()
        .take_while(|(_, l)| l.trim() != "#[cfg(test)]")
        .filter(|(_, l)| !l.trim_start().starts_with("//"))
        .map(|(i, l)| (i + 1, l))
        .collect()
}

pub(crate) fn ui_layer_roots() -> Vec<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![manifest.join("src"), manifest.join("../app/src")]
}

/// Every non-test source file of the UI layer, for a guard that reads more than one
/// line at a time.
pub(crate) fn ui_layer_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in ui_layer_roots() {
        rust_files(&root, &mut files);
    }
    files.retain(|p| !is_test_file(p));
    files
}

/// Scans the UI layer for lines containing any of `forbidden`, skipping test
/// code and every file `exempt` accepts. Returns the number of files scanned
/// (so a wrong path cannot make a scan vacuous) and `path:line: code` for each
/// offence.
pub(crate) fn scan(forbidden: &[&str], exempt: impl Fn(&Path) -> bool) -> (usize, Vec<String>) {
    let mut files = Vec::new();
    for root in ui_layer_roots() {
        rust_files(&root, &mut files);
    }
    let mut scanned = 0;
    let mut offenders = Vec::new();
    for path in files.iter().filter(|p| !is_test_file(p) && !exempt(p)) {
        scanned += 1;
        let text = fs::read_to_string(path).unwrap();
        for (line, code) in production_code(&text) {
            if forbidden.iter().any(|f| code.contains(f)) {
                offenders.push(format!("{}:{line}: {}", path.display(), code.trim()));
            }
        }
    }
    (scanned, offenders)
}
