//! RFC-084 §4.5, the guard's other half: every text area whose value reaches a
//! draft is known.
//!
//! A text area delivers `\n` whatever the file uses. Each one below commits
//! through `omriss-ui`'s newline-preserving path (`text_area_path_tests` there
//! checks the path), so adding a new one is a decision to review: it either
//! commits through that path too — and goes on this list — or it does not commit
//! source text at all.

use crate::source_scan::scan;

const SITES: &[(&str, usize)] = &[
    // The Markdown section editor.
    ("focused_content_pane.rs", 1),
    // The plain file text.
    ("raw_source.rs", 1),
    // The JSON scalar's one-line input and the JSON container's raw-text editor.
    ("structured.rs", 2),
];

#[test]
fn the_text_areas_that_feed_a_draft_are_exactly_the_known_ones() {
    let (scanned, offenders) = scan(
        &["draft.set(event.value())", "draft.set(ev.value())"],
        |_| false,
    );
    assert!(scanned > 30, "scan looks wrong: only {scanned} files read");
    let mut found: Vec<(String, usize)> = Vec::new();
    for line in &offenders {
        let file = line
            .split(':')
            .next()
            .and_then(|p| p.rsplit('/').next())
            .unwrap()
            .to_string();
        match found.iter_mut().find(|(f, _)| *f == file) {
            Some((_, n)) => *n += 1,
            None => found.push((file, 1)),
        }
    }
    found.sort();
    let mut expected: Vec<(String, usize)> =
        SITES.iter().map(|(f, n)| (f.to_string(), *n)).collect();
    expected.sort();
    assert_eq!(
        found,
        expected,
        "the set of text areas feeding a draft changed (RFC-084 §4.5). A new one must commit \
         through omriss-ui's newline-preserving path and be listed here:\n{}",
        offenders.join("\n")
    );
}
