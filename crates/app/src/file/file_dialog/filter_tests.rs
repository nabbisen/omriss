//! The open dialog's default filter must offer every supported extension
//! (Task 016, RFC-052 §14.4: JSON visible by default).

use super::{JSON_EXTENSIONS, MD_EXTENSIONS, all_extensions};

#[test]
fn default_filter_is_the_union_of_every_format() {
    let all = all_extensions();
    for ext in MD_EXTENSIONS.iter().chain(JSON_EXTENSIONS) {
        assert!(all.contains(ext), "default filter is missing `{ext}`");
    }
}

#[test]
fn default_filter_lists_no_extension_twice_and_none_extra() {
    let all = all_extensions();
    assert_eq!(all.len(), MD_EXTENSIONS.len() + JSON_EXTENSIONS.len());
    let mut sorted = all.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), all.len(), "duplicate extension in {all:?}");
}

/// `rfd`'s Windows backend uses the first extension of the first filter as
/// the Save-As default extension, so the chain in `all_extensions()` must
/// keep a Markdown extension first (RFC-055 §4.1).
#[test]
fn default_filter_starts_with_a_markdown_extension() {
    assert_eq!(
        all_extensions().first(),
        MD_EXTENSIONS.first(),
        "the default filter's first extension is what Windows appends to a bare \
         Save-As file name (rfd's SetDefaultExtension); it must stay Markdown's, \
         so keep MD_EXTENSIONS first in all_extensions() (RFC-055 §4.1)"
    );
}

/// RFC-073 §4.4b: the default filter's name is a phrase, so it is the catalog's; the
/// shell resolves it and hands the file layer text. "Markdown" and "JSON" are format
/// names and stay as they are.
#[test]
fn the_default_filters_name_is_translated_and_the_format_names_are_not() {
    use omriss_ui::i18n::{Locale, t};
    let en = t(Locale::En, "dialog.filter.all_supported");
    let ja = t(Locale::Ja, "dialog.filter.all_supported");
    assert_eq!(en, "All supported");
    assert_ne!(ja, en);
    assert_ne!(
        ja, "dialog.filter.all_supported",
        "the key is in the catalog"
    );
    // The file layer takes the text it is given.
    let labels = super::DialogLabels {
        all_supported: ja.to_string(),
    };
    assert_eq!(labels.all_supported, ja);
}
