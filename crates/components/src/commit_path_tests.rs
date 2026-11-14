//! RFC-069 §2.4: there is exactly one commit sequence for a pending section
//! draft, and it lives in `draft_sync`.
//!
//! The bug class is "a second, unguarded copy of the commit sequence": inline
//! `current_snapshot()` + `commit_focused_body()` is the Markdown path with no
//! format check and no resync, and every copy has to be remembered by whoever
//! next changes the rules (RFC-069 §2.1 was one such drift). This scans the
//! sources of the whole UI layer, so a new copy fails here instead of passing
//! review.
//!
//! Scope: `crates/components/src` and `crates/app/src`. Exempt: `draft_sync`
//! itself, and test code (files named `tests.rs` / `*_tests.rs`, and anything
//! after a `#[cfg(test)]` line, by the repository's convention that test
//! modules close a file).

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &str = "commit_focused_body";

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

fn is_exempt(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    let in_draft_sync =
        path.components().any(|c| c.as_os_str() == "draft_sync") || name == "draft_sync.rs";
    in_draft_sync || name == "tests.rs" || name.ends_with("_tests.rs")
}

/// Non-comment, non-test code of `text`.
fn production_code(text: &str) -> Vec<(usize, &str)> {
    text.lines()
        .enumerate()
        .take_while(|(_, l)| l.trim() != "#[cfg(test)]")
        .filter(|(_, l)| !l.trim_start().starts_with("//"))
        .map(|(i, l)| (i + 1, l))
        .collect()
}

fn scan(roots: &[PathBuf]) -> (usize, Vec<String>) {
    let mut files = Vec::new();
    for root in roots {
        rust_files(root, &mut files);
    }
    let mut scanned = 0;
    let mut offenders = Vec::new();
    for path in files.iter().filter(|p| !is_exempt(p)) {
        scanned += 1;
        let text = fs::read_to_string(path).unwrap();
        for (line, code) in production_code(&text) {
            if code.contains(FORBIDDEN) {
                offenders.push(format!("{}:{line}: {}", path.display(), code.trim()));
            }
        }
    }
    (scanned, offenders)
}

fn ui_layer_roots() -> Vec<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![manifest.join("src"), manifest.join("../app/src")]
}

#[test]
fn no_unguarded_copy_of_the_commit_sequence_outside_draft_sync() {
    let (scanned, offenders) = scan(&ui_layer_roots());
    // A wrong path would make the scan vacuous; the UI layer has far more files.
    assert!(scanned > 30, "scan looks wrong: only {scanned} files read");
    assert!(
        offenders.is_empty(),
        "commit_focused_body must only be called from draft_sync (RFC-069 §2.4); \
         route these through draft_sync::commit_or_block / commit_and_resync:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_one_commit_path_exists_in_draft_sync() {
    // The guard is only meaningful while the sequence it protects is there.
    let text =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/draft_sync.rs"))
            .unwrap();
    assert!(text.contains(FORBIDDEN));
}

#[test]
fn the_scanner_sees_a_copy_and_ignores_comments_and_test_code() {
    let src = "// commit_focused_body in a comment\nfn a() { s.commit_focused_body(&x, d); }\n#[cfg(test)]\nmod t { fn b() { s.commit_focused_body(&x, d); } }\n";
    let code = production_code(src);
    assert_eq!(code.len(), 1, "comment and test module are dropped");
    assert!(code[0].1.contains(FORBIDDEN));
    assert!(is_exempt(Path::new(
        "crates/app/src/file/round_trip_tests.rs"
    )));
    assert!(is_exempt(Path::new(
        "crates/components/src/draft_sync/tests.rs"
    )));
    assert!(is_exempt(Path::new("crates/components/src/draft_sync.rs")));
    assert!(!is_exempt(Path::new("crates/app/src/shell/app.rs")));
}
