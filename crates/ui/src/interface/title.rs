//! The one function that says a node's title (RFC-073 §8.6).
//!
//! A title from core is *data*: a heading's text, an object member's key, or — for
//! a list's member — its **number alone**, since core is catalog-free and must not
//! write English. What a person reads is made here, in their language: the Document
//! Map, the overview's cards, the breadcrumb, the Writing Area's heading and a
//! search result's path all show a title through this, so they cannot disagree.

use crate::i18n::{Locale, t, t_args};

/// `title` as shown. A positional title (a list member's number) gets its word from
/// the catalog ("Item 3", "項目 3"); the document's root, which has no title, is
/// "Document"; any other title is the user's own text, unchanged.
pub fn display_title(lang: Locale, title: &str, positional: bool) -> String {
    if positional {
        t_args(
            lang,
            "focused_content.json.list_member",
            &[("number", title)],
        )
    } else if title.is_empty() {
        t(lang, "breadcrumb.root").to_string()
    } else {
        title.to_string()
    }
}

/// The product's name: the one place it is written for anything that names the
/// application — the window, the page's document, the welcome heading.
pub const PRODUCT_NAME: &str = "Omriss";

/// The name of the window and of the page, one string for both (RFC-060 §11): the
/// product's name with no file open, and "notes.md — Omriss" with one. It is the
/// file's *name*, not its path — the status bar shows the path — and the product
/// name is not translated. Unsaved changes are not in it (RFC-080 §5.4 rule 1).
///
/// The path is the file's own (RFC-068 §7.5); a name that is not valid text is shown
/// with its unreadable bytes replaced, which is all a title can do with it.
pub fn window_title(file: Option<&std::path::Path>) -> String {
    let name = file
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy())
        .filter(|n| !n.is_empty());
    match name {
        Some(name) => format!("{name} \u{2014} {PRODUCT_NAME}"),
        None => PRODUCT_NAME.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn no_file_is_the_product_name() {
        assert_eq!(window_title(None), "Omriss");
        assert_eq!(window_title(None), PRODUCT_NAME);
        assert_eq!(window_title(Some(Path::new(""))), "Omriss");
    }

    #[test]
    fn a_file_is_its_name_not_its_path() {
        assert_eq!(
            window_title(Some(Path::new("/home/a/docs/notes.md"))),
            "notes.md \u{2014} Omriss"
        );
        assert_eq!(
            window_title(Some(Path::new("notes.md"))),
            "notes.md \u{2014} Omriss"
        );
        // A name with a dash, a quote and Japanese is shown as it is.
        assert_eq!(
            window_title(Some(Path::new("/x/\u{30e1}\u{30e2} \"a\" \u{2014} b.md"))),
            "\u{30e1}\u{30e2} \"a\" \u{2014} b.md \u{2014} Omriss"
        );
    }

    #[test]
    fn a_path_with_no_file_name_is_the_product_name() {
        assert_eq!(window_title(Some(Path::new("/"))), "Omriss");
        assert_eq!(window_title(Some(Path::new(".."))), "Omriss");
    }

    /// RFC-068 §7.5: a file name that is not valid text still names the window after
    /// the file (its unreadable bytes replaced), never the product alone as if no file
    /// were open.
    #[cfg(unix)]
    #[test]
    fn a_name_that_is_not_valid_text_is_still_the_files_name() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let path = std::path::PathBuf::from(OsString::from_vec(b"/x/caf\xe9.md".to_vec()));
        assert_eq!(window_title(Some(&path)), "caf\u{fffd}.md \u{2014} Omriss");
    }
}
