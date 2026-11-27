//! RFC-060 §11.3: the window's title and the page's name are set in one place.
//!
//! Two titles that disagree — the window saying one thing and the screen reader
//! another — are the fault this prevents, and a second call that sets either is
//! how it would come back. This scans the sources of the whole UI layer (see
//! `source_scan`). Exempt: `document_title.rs`, the one component, which also holds
//! the start-up script. `main.rs` may state the window's *first* title, and only
//! from the product-name constant.

use std::path::Path;

use crate::source_scan::{is_test_file, production_code, scan};

/// Every way this code base could name the window or the page.
const FORBIDDEN: &[&str] = &[
    "document.title",
    "document::Title",
    "::Title",
    "set_title",
    "with_title(",
    "<title",
];

fn is_title_component(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n == "document_title.rs")
}

/// Lines `main.rs` is allowed: the window's start-up title, from the constant.
fn is_start_up_line(path: &Path, line: &str) -> bool {
    path.file_name().is_some_and(|n| n == "main.rs")
        && line.contains("with_title(")
        && line.contains("PRODUCT_NAME")
}

#[test]
fn no_title_is_set_outside_the_title_component() {
    let (scanned, offenders) = scan(FORBIDDEN, is_title_component);
    assert!(scanned > 30, "scan looks wrong: only {scanned} files read");
    let offenders: Vec<_> = offenders
        .into_iter()
        .filter(|o| {
            let path = o.split(':').next().unwrap_or("");
            !is_start_up_line(Path::new(path), o)
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "the window's and the page's names are set by crates/components/src/document_title.rs \
         (RFC-060 §11); `main.rs` may give the first window title from PRODUCT_NAME:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_component_is_where_the_calls_are() {
    // The guard is only meaningful while the component holds what it protects.
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/document_title.rs"),
    )
    .unwrap();
    assert!(text.contains("document::Title"));
    assert!(text.contains("document.title"));
}

#[test]
fn the_start_up_title_is_the_constant() {
    let main =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src/main.rs"))
            .unwrap();
    let with: Vec<_> = production_code(&main)
        .into_iter()
        .filter(|(_, l)| l.contains("with_title("))
        .collect();
    assert_eq!(with.len(), 1, "{with:?}");
    assert!(with[0].1.contains("PRODUCT_NAME"), "{with:?}");
}

#[test]
fn the_scanner_sees_each_way_and_the_exemptions_are_narrow() {
    for sample in [
        "let _ = document::eval(\"document.title = 'x'\");",
        "document::Title { \"x\" }",
        "ctx.set_title(\"x\");",
        "WindowBuilder::new().with_title(\"Omriss\")",
        "<title>Omriss</title>",
    ] {
        let hit = production_code(sample)
            .into_iter()
            .any(|(_, l)| FORBIDDEN.iter().any(|f| l.contains(f)));
        assert!(hit, "{sample}");
    }
    assert!(is_title_component(Path::new(
        "crates/components/src/document_title.rs"
    )));
    assert!(!is_title_component(Path::new("crates/app/src/main.rs")));
    let main = Path::new("crates/app/src/main.rs");
    assert!(is_start_up_line(main, ".with_title(PRODUCT_NAME)"));
    assert!(!is_start_up_line(main, ".with_title(\"Omriss\")"));
    assert!(!is_start_up_line(
        Path::new("crates/app/src/shell/app.rs"),
        ".with_title(PRODUCT_NAME)"
    ));
    assert!(is_test_file(Path::new(
        "crates/components/src/title_path_tests.rs"
    )));
}
