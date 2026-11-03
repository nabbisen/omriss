//! RFC-071 handoff §6: every call site that rewrites the source must go
//! through the structure-change detector, or the warning silently misses cases
//! — worse than no warning, because users learn to trust it.
//!
//! This scans the session's own source rather than trusting a list in a review
//! document: add a new `Document` mutator call to `crates/ui/src/session*` and
//! this fails until it is routed through `begin_change`/`finish_change` or
//! deliberately listed as an exception below.

use std::fs;
use std::path::PathBuf;

/// The `Document` methods that rewrite the source *as a Markdown outline
/// operation*. Each must be bracketed by `begin_change`/`finish_change`.
const OUTLINE_MUTATORS: &[&str] = &[
    "replace_section_body",
    "promote_section",
    "demote_section",
    "rename_section",
    "move_section",
    "split_section",
    "delete_section",
    "merge_with_prev_sibling",
];

/// `Document` methods that rewrite the source without a comparison, each with
/// its reason: `undo`/`redo` clear the warning (they restore or deliberately
/// repeat an edit), and `replace_range` is the primitive behind non-Markdown
/// edits (via the adapter's `apply_validated_edit`, which the session calls
/// only for JSON), whose outline is not their structure (RFC-054 §0.1).
const EXEMPT: &[&str] = &["undo", "redo", "replace_range"];

fn session_sources() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![root.join("session.rs")];
    for entry in fs::read_dir(root.join("session")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
    files
        .into_iter()
        .map(|p| {
            let text = fs::read_to_string(&p).unwrap();
            // Comments may name a method; only code counts.
            let code = text
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            (p.file_name().unwrap().to_string_lossy().into_owned(), code)
        })
        .collect()
}

fn count(sources: &[(String, String)], needle: &str) -> usize {
    sources
        .iter()
        .map(|(_, code)| code.matches(needle).count())
        .sum()
}

#[test]
fn every_outline_mutating_call_site_is_bracketed_by_the_detector() {
    let sources = session_sources();
    let mutator_calls: usize = OUTLINE_MUTATORS
        .iter()
        .map(|m| count(&sources, &format!(".{m}(")))
        .sum();
    let begins = count(&sources, "self.begin_change(");
    let finishes = count(&sources, "self.finish_change(");

    assert!(
        mutator_calls >= 10,
        "scan found only {mutator_calls} call sites"
    );
    assert_eq!(
        begins, mutator_calls,
        "{mutator_calls} outline-mutating call sites but {begins} begin_change calls"
    );
    assert_eq!(
        finishes, mutator_calls,
        "{mutator_calls} outline-mutating call sites but {finishes} finish_change calls"
    );
}

#[test]
fn every_public_document_mutator_is_either_routed_or_a_listed_exception() {
    // The enumeration, from `omriss-core`'s own public surface: every
    // `pub fn ... (&mut self ...)` on `Document`. When core gains a new way to
    // rewrite the source, this fails until someone decides which kind it is.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/src/doc/document.rs");
    let text = fs::read_to_string(path).unwrap();
    let mut found: Vec<String> = Vec::new();
    for (start, _) in text.match_indices("pub fn ") {
        let signature = &text[start..];
        let signature = &signature[..signature.find('{').unwrap_or(signature.len())];
        if signature.contains("&mut self") {
            let name = signature["pub fn ".len()..]
                .split('(')
                .next()
                .unwrap()
                .trim();
            found.push(name.to_string());
        }
    }
    found.sort_unstable();

    let mut known: Vec<String> = OUTLINE_MUTATORS
        .iter()
        .chain(EXEMPT)
        .map(|s| s.to_string())
        .collect();
    known.sort_unstable();
    known.dedup();

    assert_eq!(
        found, known,
        "Document's public mutators changed: route the new one through the \
         structure-change detector, or add it to EXEMPT with a reason"
    );
}
