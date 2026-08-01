//! RFC-052 §5.1 / RFC-053 §10: extension-based format detection.

use std::path::Path;

use crate::DocumentFormat;
use crate::formats::detection::{DetectionConfidence, confidence_for, detect_format};

fn detect(path: &str) -> DocumentFormat {
    detect_format(Some(Path::new(path)), "")
}

#[test]
fn md_markdown_mdown_txt_all_classify_as_markdown() {
    for path in ["a.md", "a.markdown", "a.mdown", "a.txt"] {
        assert_eq!(detect(path), DocumentFormat::Markdown, "{path}");
    }
}

#[test]
fn json_classifies_as_json() {
    assert_eq!(detect("a.json"), DocumentFormat::Json);
}

#[test]
fn toml_classifies_as_toml() {
    assert_eq!(detect("a.toml"), DocumentFormat::Toml);
}

#[test]
fn yaml_and_yml_classify_as_yaml_experimental() {
    assert_eq!(detect("a.yaml"), DocumentFormat::YamlExperimental);
    assert_eq!(detect("a.yml"), DocumentFormat::YamlExperimental);
}

#[test]
fn unknown_extension_classifies_as_plain_text() {
    assert_eq!(detect("a.xyz"), DocumentFormat::PlainText);
    assert_eq!(detect("a"), DocumentFormat::PlainText);
}

#[test]
fn extension_matching_is_case_insensitive() {
    assert_eq!(detect("a.MD"), DocumentFormat::Markdown);
    assert_eq!(detect("a.Json"), DocumentFormat::Json);
    assert_eq!(detect("a.YML"), DocumentFormat::YamlExperimental);
}

#[test]
fn no_path_falls_back_to_plain_text_without_panicking() {
    assert_eq!(
        detect_format(None, "# looks like markdown"),
        DocumentFormat::PlainText
    );
    assert_eq!(detect_format(None, ""), DocumentFormat::PlainText);
}

#[test]
fn confidence_is_certain_for_an_extension_backed_match() {
    let path = Path::new("a.md");
    assert_eq!(
        confidence_for(DocumentFormat::Markdown, Some(path), ""),
        DetectionConfidence::Certain
    );
}

#[test]
fn confidence_is_no_for_a_mismatched_format() {
    let path = Path::new("a.md");
    assert_eq!(
        confidence_for(DocumentFormat::Json, Some(path), ""),
        DetectionConfidence::No
    );
}

#[test]
fn confidence_is_certain_for_an_unmapped_extension() {
    // An extension that is present but not in the RFC-052 §5.1 mapping is
    // positive evidence the file is not a format omriss knows, so it grades
    // the same as any other extension-backed match — unlike the no-path
    // fallback, which is `Maybe` (see `confidence_is_maybe_for_the_no_path_plain_text_fallback`).
    let path = Path::new("a.xyz");
    assert_eq!(
        confidence_for(DocumentFormat::PlainText, Some(path), ""),
        DetectionConfidence::Certain
    );
}

#[test]
fn confidence_is_maybe_for_the_no_path_plain_text_fallback() {
    assert_eq!(
        confidence_for(DocumentFormat::PlainText, None, ""),
        DetectionConfidence::Maybe
    );
}
