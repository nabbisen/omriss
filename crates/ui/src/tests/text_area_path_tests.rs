//! RFC-084 §4.5, the guard: text-area text becomes source text in **one** place.
//!
//! A text area delivers `\n` whatever the file uses, so committing its value
//! as the new text rewrites every line ending of a CRLF file. RFC-076 solved
//! that for the plain file text; the section editor and a JSON container's raw
//! text did not, and the same bug lived in each. This test fails when a commit
//! path stops going through `editor::raw_text`, and when anything else starts
//! rewriting newlines in text it was given.

use std::fs;
use std::path::PathBuf;

fn ui_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "tests") {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name != "tests.rs" && !name.ends_with("_tests.rs") {
                out.push(path);
            }
        }
    }
}

/// The text of `fn name`, from its signature to the next `fn` at the same
/// indentation (or the end of the file).
fn function_text(file: &str, name: &str) -> String {
    let source = fs::read_to_string(ui_src().join(file)).unwrap();
    let marker = format!("fn {name}(");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("{file} has no `fn {name}(`"));
    let rest = &source[start + marker.len()..];
    let end = rest
        .find("\n    pub fn ")
        .into_iter()
        .chain(rest.find("\n    fn "))
        .min()
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn every_commit_path_goes_through_the_newline_preserving_function() {
    let commits = [
        // (file, function, what it must call)
        ("session.rs", "commit_focused_body", "apply_text_area_draft"),
        ("session/raw_edit.rs", "commit_raw_text", "raw_edit"),
        (
            "session/structured_edit.rs",
            "structured_draft_source",
            "apply_text_area_draft",
        ),
        (
            "session/structured_edit.rs",
            "commit_structured_draft",
            "structured_draft_source",
        ),
        (
            "session/structured_edit.rs",
            "validate_structured_draft",
            "structured_draft_source",
        ),
    ];
    for (file, function, must_call) in commits {
        assert!(
            function_text(file, function).contains(must_call),
            "{file}: `{function}` must go through `{must_call}` (RFC-084 §4.1): a text area's \
             value is never the new text"
        );
    }
}

#[test]
fn nothing_else_rewrites_newlines_in_text_it_was_given() {
    let mut files = Vec::new();
    rust_files(&ui_src(), &mut files);
    assert!(files.len() > 20, "scan looks wrong: {} files", files.len());
    let mut offenders = Vec::new();
    for path in files {
        if path.ends_with("editor/raw_text.rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (i, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            // Converting *to or from* the other line-ending style. (Replacing a
            // newline with a space for a one-line display is not source text.)
            let rewrites = [
                ".replace('\\n', \"\\r\\n\")",
                ".replace(\"\\n\", \"\\r\\n\")",
                ".replace(\"\\r\\n\"",
                ".replace('\\r'",
                ".replace(\"\\r\"",
            ]
            .iter()
            .any(|p| line.contains(p));
            if rewrites {
                offenders.push(format!("{}:{}: {}", path.display(), i + 1, code));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "newline conversion belongs to editor/raw_text.rs (RFC-084 §4.1):\n{}",
        offenders.join("\n")
    );
}
