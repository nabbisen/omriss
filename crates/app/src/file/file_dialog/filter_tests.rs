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
