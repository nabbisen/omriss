//! RFC-069 §2.4: there is exactly one commit sequence for a pending section
//! draft, and it lives in `draft_sync`.
//!
//! The bug class is "a second, unguarded copy of the commit sequence": inline
//! `current_snapshot()` + `commit_focused_body()` is the Markdown path with no
//! format check and no resync, and every copy has to be remembered by whoever
//! next changes the rules (RFC-069 §2.1 was one such drift). This scans the
//! sources of the whole UI layer (see `source_scan`), so a new copy fails here
//! instead of passing review. Exempt: `draft_sync` itself.

use std::fs;
use std::path::{Path, PathBuf};

use crate::source_scan::{is_test_file, production_code, scan};

const FORBIDDEN: &str = "commit_focused_body";

fn is_draft_sync(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "draft_sync")
        || path.file_name().is_some_and(|n| n == "draft_sync.rs")
}

#[test]
fn no_unguarded_copy_of_the_commit_sequence_outside_draft_sync() {
    let (scanned, offenders) = scan(&[FORBIDDEN], is_draft_sync);
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
    assert!(is_test_file(Path::new(
        "crates/app/src/file/round_trip_tests.rs"
    )));
    assert!(is_test_file(Path::new(
        "crates/components/src/draft_sync/tests.rs"
    )));
    assert!(is_draft_sync(Path::new(
        "crates/components/src/draft_sync.rs"
    )));
    assert!(!is_draft_sync(Path::new("crates/app/src/shell/app.rs")));
}
