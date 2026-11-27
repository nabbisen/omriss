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
pub fn window_title(file: Option<&str>) -> String {
    let name = file
        .map(std::path::Path::new)
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty());
    match name {
        Some(name) => format!("{name} \u{2014} {PRODUCT_NAME}"),
        None => PRODUCT_NAME.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_file_is_the_product_name() {
        assert_eq!(window_title(None), "Omriss");
        assert_eq!(window_title(None), PRODUCT_NAME);
        assert_eq!(window_title(Some("")), "Omriss");
    }

    #[test]
    fn a_file_is_its_name_not_its_path() {
        assert_eq!(
            window_title(Some("/home/a/docs/notes.md")),
            "notes.md \u{2014} Omriss"
        );
        assert_eq!(window_title(Some("notes.md")), "notes.md \u{2014} Omriss");
        // A name with a dash, a quote and Japanese is shown as it is.
        assert_eq!(
            window_title(Some("/x/\u{30e1}\u{30e2} \"a\" \u{2014} b.md")),
            "\u{30e1}\u{30e2} \"a\" \u{2014} b.md \u{2014} Omriss"
        );
    }

    #[test]
    fn a_path_with_no_file_name_is_the_product_name() {
        assert_eq!(window_title(Some("/")), "Omriss");
        assert_eq!(window_title(Some("..")), "Omriss");
    }
}
