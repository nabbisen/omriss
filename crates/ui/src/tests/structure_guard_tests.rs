//! RFC-085 §3.1, the guard: every pane reads the document's `DocumentStructure`,
//! whatever the format; the Markdown outline is read only where Markdown is
//! edited.
//!
//! With a JSON file open, half of omriss read the Markdown heading outline of the
//! file's text — a parse that means nothing for JSON — while the other half read
//! the JSON structure, and the two disagreed in front of the user (RFC-085 §2).
//! Markdown, JSON and plain-text node ids come from the same derivation, so a
//! lookup in the outline misses every JSON node but the root, and *finds the
//! root*: the wrong reader fails quietly. This scans the non-test code of
//! `omriss-ui`, `omriss-components` and `omriss`, and fails when a reader of the
//! outline, or of the section snapshots built on it, is not on the list below,
//! each with the reason it may be.

use std::fs;
use std::path::{Path, PathBuf};

/// What reads the Markdown outline, directly or through a snapshot of a section.
const READERS: &[&str] = &[
    ".outline()",
    "focus_snapshot(",
    "current_snapshot(",
    "sibling_info(",
];

/// `(file under crates/, a reader it may contain, why)`.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "ui/src/session.rs",
        ".outline()",
        "RFC-071's structure-change check, and the section commit's heading context (RFC-084): both Markdown editing",
    ),
    (
        "ui/src/session.rs",
        "current_snapshot(",
        "the Markdown section editor: its body, the commit's base, back/forward",
    ),
    (
        "ui/src/session.rs",
        "focus_snapshot(",
        "the one definition of current_snapshot",
    ),
    (
        "ui/src/session.rs",
        "sibling_info(",
        "Markdown structural editing: where a new sibling goes",
    ),
    (
        "ui/src/session/focus_bridge.rs",
        "focus_snapshot(",
        "focusing a Markdown section returns the snapshot its editor commits against",
    ),
    (
        "ui/src/session/structural.rs",
        ".outline()",
        "Markdown structural edits (add, move, split, merge, delete)",
    ),
    (
        "ui/src/session/structural.rs",
        "sibling_info(",
        "Markdown structural edits: enabling and aiming move up/down",
    ),
    (
        "ui/src/session/pane_structure.rs",
        ".outline()",
        "a card's \"H2\": the heading level, which the structure does not carry",
    ),
    (
        "ui/src/editor/navigation.rs",
        "sibling_info(",
        "the definition of sibling_info, over the outline it is given",
    ),
    (
        "components/src/draft_sync.rs",
        "current_snapshot(",
        "the Markdown half of commit and sync: the section's body",
    ),
    (
        "components/src/focused_content_pane.rs",
        "current_snapshot(",
        "the Markdown Writing Area: the section's body and children",
    ),
    (
        "components/src/preview_pane.rs",
        "current_snapshot(",
        "the Markdown preview: the section's body",
    ),
    (
        "components/src/structure_request.rs",
        "current_snapshot(",
        "Markdown structural requests (hidden for every other format)",
    ),
    (
        "app/src/shell/actions.rs",
        "current_snapshot(",
        "creating a Markdown section: the focused section's children",
    ),
    (
        "app/src/shell/actions.rs",
        "sibling_info(",
        "creating a Markdown section after the focused one",
    ),
];

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
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

/// `(file relative to crates/, reader, line)` for each reader in non-test code
/// (everything before a `#[cfg(test)]` line; comment lines skipped).
fn readers_in(roots: &[&str]) -> (usize, Vec<(String, &'static str, String)>) {
    let mut found = Vec::new();
    let mut scanned = 0;
    for root in roots {
        let mut files = Vec::new();
        rust_files(&crates_dir().join(root), &mut files);
        for path in files {
            scanned += 1;
            let text = fs::read_to_string(&path).unwrap();
            let rel = path
                .canonicalize()
                .unwrap()
                .strip_prefix(crates_dir().canonicalize().unwrap())
                .unwrap()
                .to_string_lossy()
                .into_owned();
            for line in text.lines() {
                if line.trim() == "#[cfg(test)]" {
                    break;
                }
                if line.trim_start().starts_with("//") {
                    continue;
                }
                for reader in READERS {
                    if line.contains(reader) {
                        found.push((rel.clone(), *reader, line.trim().to_string()));
                    }
                }
            }
        }
    }
    (scanned, found)
}

#[test]
fn nothing_reads_the_markdown_outline_but_the_listed_places() {
    let (scanned, found) = readers_in(&["ui/src", "components/src", "app/src"]);
    assert!(scanned > 60, "scan looks wrong: only {scanned} files read");
    let unlisted: Vec<String> = found
        .iter()
        .filter(|(file, reader, _)| !ALLOWED.iter().any(|(f, r, _)| f == file && r == reader))
        .map(|(file, _, line)| format!("{file}: {line}"))
        .collect();
    assert!(
        unlisted.is_empty(),
        "these read the Markdown outline (RFC-085 §3.1). Read the document's structure \
         instead — EditorSession's pane_structure methods; if the place really edits \
         Markdown, list it in ALLOWED with its reason:\n{}",
        unlisted.join("\n")
    );
}

#[test]
fn every_listed_place_still_reads_what_it_is_allowed_to() {
    // A reason that outlives its code is noise: each entry must match a line.
    let (_, found) = readers_in(&["ui/src", "components/src", "app/src"]);
    for (file, reader, _) in ALLOWED {
        assert!(
            found.iter().any(|(f, r, _)| f == file && r == reader),
            "ALLOWED lists {file}: `{reader}`, which is no longer there"
        );
    }
}
