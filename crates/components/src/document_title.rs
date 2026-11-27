//! The name of the window and of the page (RFC-060 §11).
//!
//! One string, [`window_title`], applied to both targets by this one component, in
//! the same render: the window through `document::Title`, which sets only the
//! native window's title, and the page through `document.title`, which is what an
//! assistive technology announces as the document's name. No other code sets
//! either; `title_path_tests` scans for it.
//!
//! The page's *first* name comes from [`startup_head`], run as the page is parsed,
//! so it is never the framework's own default, not even before the first effect.

use dioxus::prelude::*;
use omriss_ui::{EditorSession, PRODUCT_NAME, window_title};

/// `text` as a JavaScript string literal, double-quoted.
fn js_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 || matches!(c, '\u{2028}' | '\u{2029}') => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The statement that names the page.
fn set_page_title(title: &str) -> String {
    format!("document.title = {};", js_string(title))
}

/// What the page's `<head>` is given at start-up: the product's name, set as the
/// page is parsed. `dioxus-desktop`'s own `<title>Dioxus app</title>` comes first
/// in the document and wins over any `<title>` added later, so this is a script.
pub fn startup_head() -> String {
    format!("<script>{}</script>", set_page_title(PRODUCT_NAME))
}

/// Applies the title for the open file (or none) to the window and the page.
#[component]
pub fn DocumentTitle(session: Signal<EditorSession>) -> Element {
    let title = window_title(session.read().file_name());
    let page = set_page_title(&title);
    use_effect(use_reactive!(|page| {
        let _ = document::eval(&page);
    }));
    rsx! {
        document::Title { "{title}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_is_one_javascript_string() {
        assert_eq!(js_string("Omriss"), "\"Omriss\"");
        assert_eq!(
            js_string("a\"b\\c\nd\u{2028}\u{2014}"),
            "\"a\\\"b\\\\c\\nd\\u2028\u{2014}\""
        );
    }

    #[test]
    fn the_start_up_head_names_the_page_and_never_dioxus() {
        let head = startup_head();
        assert_eq!(head, "<script>document.title = \"Omriss\";</script>");
        assert!(!head.contains("Dioxus"));
        // The same name the window gets with no file open.
        assert!(head.contains(&js_string(&window_title(None))));
    }
}
