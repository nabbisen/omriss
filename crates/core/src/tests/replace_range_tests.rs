//! RFC-054 J1: `Document::replace_range` — the format-neutral mutation
//! primitive JSON (and any future non-Markdown format) needs. Proves it
//! routes through the same transactional path `replace_section_body`
//! uses: undo/redo, revision increment, and stale-revision rejection all
//! behave identically, just without resolving the range from a `NodeId`.

use crate::{ByteRange, Document, DocumentRevision, EditError};

fn doc(source: &str) -> Document {
    Document::parse(source.to_string()).expect("parse")
}

#[test]
fn replace_range_replaces_only_the_given_range() {
    // Plain-text-shaped source (a JSON-like line) — replace_range does not
    // care that "headings" happen to parse over it; it addresses bytes
    // directly, unlike replace_section_body.
    let source = r#"{"name": "old", "version": 1}"#;
    let mut document = doc(source);
    let start = source.find("old").unwrap();
    let range = ByteRange {
        start,
        end: start + "old".len(),
    };

    document
        .replace_range(range, "new".to_string(), document.revision())
        .unwrap();

    assert_eq!(document.source(), r#"{"name": "new", "version": 1}"#);
}

#[test]
fn unrelated_bytes_are_preserved_outside_the_range() {
    let source = "before[[[TARGET]]]after";
    let mut document = doc(source);
    let start = source.find("[[[TARGET]]]").unwrap();
    let range = ByteRange {
        start,
        end: start + "[[[TARGET]]]".len(),
    };

    document
        .replace_range(range, "X".to_string(), document.revision())
        .unwrap();

    assert_eq!(document.source(), "beforeXafter");
}

#[test]
fn revision_increments_exactly_once_per_successful_replace_range() {
    let mut document = doc("abc");
    assert_eq!(document.revision(), DocumentRevision::INITIAL);
    let result = document
        .replace_range(
            ByteRange { start: 0, end: 1 },
            "z".to_string(),
            document.revision(),
        )
        .unwrap();
    assert_eq!(result.old_revision, DocumentRevision(0));
    assert_eq!(result.new_revision, DocumentRevision(1));
    assert_eq!(document.revision(), DocumentRevision(1));
}

#[test]
fn stale_base_revision_is_rejected_before_any_mutation() {
    let mut document = doc("abc");
    let stale = document.revision();
    document
        .replace_range(
            ByteRange { start: 0, end: 1 },
            "z".to_string(),
            document.revision(),
        )
        .unwrap();

    let err = document
        .replace_range(ByteRange { start: 1, end: 2 }, "y".to_string(), stale)
        .unwrap_err();
    assert!(matches!(err, EditError::RevisionMismatch { .. }));
    // Rejected before mutation: source is exactly the post-first-edit state.
    assert_eq!(document.source(), "zbc");
}

#[test]
fn undo_restores_pre_edit_source_byte_exactly() {
    let original = r#"{"a": 1}"#;
    let mut document = doc(original);
    let start = original.find('1').unwrap();
    document
        .replace_range(
            ByteRange {
                start,
                end: start + 1,
            },
            "2".to_string(),
            document.revision(),
        )
        .unwrap();
    assert_ne!(document.source(), original);

    document.undo().unwrap();
    assert_eq!(document.source(), original);
}

#[test]
fn redo_restores_post_edit_source_byte_exactly() {
    let mut document = doc("abc");
    document
        .replace_range(
            ByteRange { start: 0, end: 1 },
            "z".to_string(),
            document.revision(),
        )
        .unwrap();
    let edited = document.source().to_string();

    document.undo().unwrap();
    document.redo().unwrap();
    assert_eq!(document.source(), edited);
}

#[test]
fn undo_after_replace_range_produces_a_fresh_revision_not_a_rewind() {
    // RFC-044: undo is itself a forward-moving, revision-incrementing edit,
    // not a pointer rewind -- true for replace_range exactly as it is for
    // replace_section_body.
    let mut document = doc("abc");
    document
        .replace_range(
            ByteRange { start: 0, end: 1 },
            "z".to_string(),
            document.revision(),
        )
        .unwrap(); // rev 1
    let undo_result = document.undo().unwrap(); // rev 2
    assert_eq!(undo_result.new_revision, document.revision());
    assert!(undo_result.new_revision > DocumentRevision(1));
}

#[test]
fn invalid_range_is_rejected_without_mutating() {
    let mut document = doc("abc");
    let err = document
        .replace_range(
            ByteRange { start: 0, end: 100 },
            "z".to_string(),
            document.revision(),
        )
        .unwrap_err();
    assert!(matches!(err, EditError::InvalidRange(_)));
    assert_eq!(document.source(), "abc");
}

#[test]
fn multibyte_ranges_replace_safely() {
    let source = "日本語のテスト";
    let mut document = doc(source);
    // Replace the second character ("本"), addressed by its exact byte span.
    let start = "日".len();
    let end = start + "本".len();
    document
        .replace_range(
            ByteRange { start, end },
            "🎉".to_string(),
            document.revision(),
        )
        .unwrap();
    assert_eq!(document.source(), "日🎉語のテスト");
}

/// RFC-054 §0.1's whole point: shipped Markdown mutation must behave
/// identically after `replace_range` is added -- it is a new, independent
/// operation, not a rewrite of `replace_section_body`'s path.
#[test]
fn shipped_section_operations_are_unaffected_by_replace_range_existing() {
    use crate::ReplaceSectionBody;

    let mut document = doc("# A\nold body\n\n# B\nbody b\n");
    let a = document.outline().root().children[0];
    document
        .replace_section_body(ReplaceSectionBody {
            node_id: a,
            base_revision: document.revision(),
            new_body: "new body\n\n".to_string(),
        })
        .unwrap();
    assert_eq!(document.source(), "# A\nnew body\n\n# B\nbody b\n");
}
