//! RFC-059 §12.1: every programmatic focus move is in `focus.rs`.
//!
//! Components declare focus in markup (`data-focus-scope`, `autofocus`, …) and
//! one installed script acts on it. A second place that calls `.focus()` — in
//! a `document::eval` string or anywhere else — is a second focus model that
//! the script knows nothing about, which is exactly how the project came to
//! have four. This scans the sources of the whole UI layer (see `source_scan`),
//! so a new one fails here. Exempt: `focus.rs` itself.

use std::path::Path;

use crate::source_scan::{is_test_file, production_code, scan};

/// `.focus()` and `.focus({…})`. The Rust session method takes an id
/// (`session.focus(id)`), so it matches neither.
const FORBIDDEN: &[&str] = &[".focus()", ".focus({"];

fn is_focus_module(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n == "focus.rs")
        || path.components().any(|c| c.as_os_str() == "focus")
}

#[test]
fn no_focus_call_outside_focus_rs() {
    let (scanned, offenders) = scan(FORBIDDEN, is_focus_module);
    assert!(scanned > 30, "scan looks wrong: only {scanned} files read");
    assert!(
        offenders.is_empty(),
        "focus moves belong to crates/components/src/focus.rs (RFC-059 §12.1); \
         declare it in markup (data-focus-scope / autofocus) instead:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_script_is_where_the_focus_calls_are() {
    // The guard is only meaningful while focus.rs holds the calls it protects.
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/focus.rs"))
        .unwrap();
    assert!(text.contains(".focus()"));
}

#[test]
fn the_scanner_sees_a_call_and_ignores_the_session_method() {
    let src = "fn a() { let _ = session.write().focus(id); }\nconst JS: &str = \"el.focus()\";\n// el.focus() in a comment\n";
    let hits: Vec<_> = production_code(src)
        .into_iter()
        .filter(|(_, l)| FORBIDDEN.iter().any(|f| l.contains(f)))
        .collect();
    assert_eq!(hits.len(), 1, "only the JavaScript call: {hits:?}");
    assert!(is_focus_module(Path::new("crates/components/src/focus.rs")));
    assert!(is_test_file(Path::new(
        "crates/components/src/focus/tests.rs"
    )));
    assert!(!is_focus_module(Path::new("crates/app/src/shell/app.rs")));
}
