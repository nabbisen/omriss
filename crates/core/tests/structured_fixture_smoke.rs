//! Guards the JSON smoke fixture used by `RELEASE_CHECKLIST.md` steps 13-21.
//!
//! The release checklist tells a human tester to open
//! `structured-sample.json`, edit a value, save, and confirm in an external
//! editor that nothing else moved. That instruction is only worth following if
//! the fixture actually parses and actually contains the constructs a naive
//! parse-and-reserialize would destroy. A fixture that had quietly become
//! malformed, or been "tidied" into uniform formatting, would turn step 17
//! into a test that passes for the wrong reason.
//!
//! These tests do not exercise editing -- that is covered by the JSON adapter's
//! own suites. They assert only that the smoke instrument is intact.

use omriss_core::{DocumentFormatAdapter, DocumentRevision, JsonAdapter, StructureNodeKind};

fn fixture_source() -> String {
    let path = format!(
        "{}/tests/fixtures/structured-sample.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("smoke fixture not found: {e}"))
}

#[test]
fn smoke_fixture_parses() {
    let source = fixture_source();
    let structure = JsonAdapter
        .build_structure(&source, DocumentRevision::INITIAL)
        .expect("smoke fixture must be valid strict JSON");
    assert_eq!(structure.format, omriss_core::DocumentFormat::Json);
    assert!(
        structure.nodes.len() > 10,
        "smoke fixture should project a non-trivial tree, got {} nodes",
        structure.nodes.len()
    );
}

#[test]
fn smoke_fixture_keeps_duplicate_keys_distinct() {
    let source = fixture_source();
    let structure = JsonAdapter
        .build_structure(&source, DocumentRevision::INITIAL)
        .expect("valid JSON");
    let duplicates = structure
        .nodes
        .iter()
        .filter(|n| n.title == "duplicate")
        .count();
    assert_eq!(
        duplicates, 2,
        "both duplicate keys must project as separate nodes; \
         if this drops to 1 the fixture no longer proves keys are preserved"
    );
}

#[test]
fn smoke_fixture_contains_a_null_value() {
    let source = fixture_source();
    let structure = JsonAdapter
        .build_structure(&source, DocumentRevision::INITIAL)
        .expect("valid JSON");
    let has_null = structure
        .nodes
        .iter()
        .any(|n| n.title == "maintainer" && n.kind == StructureNodeKind::Value);
    assert!(
        has_null,
        "fixture must retain a null value so the checklist can observe \
         the read-only affordance"
    );
}

/// The fixture earns its place by being formatted in ways a reserializing
/// editor would silently normalize. If someone reformats it, step 17's byte
/// check stops proving anything and this test says so.
#[test]
fn smoke_fixture_stays_adversarially_formatted() {
    let source = fixture_source();
    for (needle, why) in [
        ("1.50", "trailing zero would normalize to 1.5"),
        ("1e3", "exponent would normalize to 1000"),
        ("-0", "negative zero would normalize to 0"),
        (
            "\n    \"description\"",
            "over-indented line proves indentation is not rewritten",
        ),
        (
            ":    \"three spaces",
            "extra spaces after a colon prove whitespace is not rewritten",
        ),
        ("日本語", "non-ASCII proves no escaping pass on save"),
    ] {
        assert!(
            source.contains(needle),
            "smoke fixture lost `{needle}` -- {why}. Restore it or the release \
             checklist's byte check is vacuous."
        );
    }
}
