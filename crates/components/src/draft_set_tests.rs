//! RFC-085 §3.2, the guard: the draft is set from the document in one place.
//!
//! A surface that opens a node and builds the draft itself — from the Markdown
//! snapshot, from a whole file — makes a draft that is not the node's committed
//! content, and the next navigation commits the difference as if the user had
//! typed it (RFC-085 F2). Opening goes through `draft_sync::open_node` (and
//! `open_overview`, `zoom_out`), which ends in `draft_sync::sync`. Every other
//! `draft.set(…)` in the UI layer is listed here with its reason, so a new one
//! is a decision, not an accident. The existing guards do not cover this:
//! `commit_path_tests` scans for the commit call, `text_area_sites_tests` for
//! text areas that deliver a typed value.

use crate::source_scan::scan;

/// `(file, text that must be in the line, why it may set the draft)`.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "focused_content_pane.rs",
        "draft.set(event.value())",
        "the section text area: what the user typed",
    ),
    (
        "raw_source.rs",
        "draft.set(event.value())",
        "the plain-file-text area: what the user typed",
    ),
    (
        "raw_source.rs",
        "draft.set(text)",
        "entering raw edit: the raw view's own draft (RFC-076), not a node's",
    ),
    (
        "structured.rs",
        "draft.set(ev.value())",
        "the JSON fields: what the user typed (three of them)",
    ),
    (
        "structured.rs",
        "draft.set(editable_text)",
        "after this field's own commit: the same node's committed text",
    ),
    (
        "structured.rs",
        "draft.set(literal.to_string())",
        "the on/off buttons: the literal just committed to this node",
    ),
    (
        "structured.rs",
        "draft.set(text)",
        "after the raw-text field's own commit: the same node's committed text",
    ),
    (
        "actions.rs",
        "draft.set(String::new())",
        "a new document: no node is open",
    ),
];

fn file_of(offender: &str) -> &str {
    offender
        .split(':')
        .next()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("")
}

fn is_allowed(offender: &str) -> bool {
    let file = file_of(offender);
    ALLOWED
        .iter()
        .any(|(f, text, _)| *f == file && offender.contains(text))
}

#[test]
fn nothing_sets_the_draft_but_draft_sync_and_the_listed_places() {
    let (scanned, offenders) = scan(&["draft.set("], |p| {
        p.components().any(|c| c.as_os_str() == "draft_sync")
            || p.file_name().is_some_and(|n| n == "draft_sync.rs")
    });
    assert!(scanned > 30, "scan looks wrong: only {scanned} files read");
    let unlisted: Vec<_> = offenders.iter().filter(|o| !is_allowed(o)).collect();
    assert!(
        unlisted.is_empty(),
        "these set the draft outside draft_sync (RFC-085 §3.2). Open a node with \
         draft_sync::open_node / open_overview / zoom_out; if the place really is not \
         opening a node, list it in ALLOWED with its reason:\n{}",
        unlisted
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn every_listed_place_still_exists() {
    // A reason that outlives its code is noise: each entry must match a line.
    let (_, offenders) = scan(&["draft.set("], |_| false);
    for (file, text, _) in ALLOWED {
        assert!(
            offenders
                .iter()
                .any(|o| file_of(o) == *file && o.contains(text)),
            "ALLOWED lists {file}: `{text}`, which is no longer in the code"
        );
    }
}
