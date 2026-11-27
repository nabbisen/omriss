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
