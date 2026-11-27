//! The name of the window and of the page (RFC-060 §11).
//!
//! One string, [`window_title`], applied to both targets by this one component, in
//! the same render: the window through `document::Title`, which sets only the
//! native window's title, and the page through `document.title`, which is what an
//! assistive technology announces as the document's name. No other code sets
//! either; `title_path_tests` scans for it.
//!
//! The page's *first* name comes from [`STARTUP_HEAD`], run as the page is parsed,
//! so it is never the framework's own default, not even before the first effect.

use dioxus::prelude::*;
use omriss_ui::{EditorSession, window_title};

/// The script that names the page. A constant: the title is *data*, and reaches the
/// script through the eval channel (`send` here, `dioxus.recv()` there), never by
/// being written into source. A file's name is anyone's to choose (RFC-060 §12).
const TITLE_SCRIPT: &str = "document.title = await dioxus.recv();";

/// What the page's `<head>` is given at start-up: the product's name, set as the
/// page is parsed. `dioxus-desktop`'s own `<title>Dioxus app</title>` comes first
/// in the document and wins over any `<title>` added later, so this is a script.
/// Written out; a test pins that it is `PRODUCT_NAME`.
pub const STARTUP_HEAD: &str = "<script>document.title = \"Omriss\";</script>";

/// Applies the title for the open file (or none) to the window and the page.
#[component]
pub fn DocumentTitle(session: Signal<EditorSession>) -> Element {
    let title = window_title(session.read().file_name());
    let for_page = title.clone();
    use_effect(use_reactive!(|for_page| {
        let eval = document::eval(TITLE_SCRIPT);
        let _ = eval.send(for_page);
    }));
    rsx! {
        document::Title { "{title}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omriss_ui::PRODUCT_NAME;

    #[test]
    fn the_start_up_head_is_the_product_name_and_never_dioxus() {
        assert!(PRODUCT_NAME.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(
            STARTUP_HEAD,
            format!("<script>document.title = \"{PRODUCT_NAME}\";</script>")
        );
        assert!(!STARTUP_HEAD.contains("Dioxus"));
        // The same name the window gets with no file open.
        assert_eq!(window_title(None), PRODUCT_NAME);
    }

    #[test]
    fn the_title_script_holds_no_title() {
        // Data comes through the channel; the source names no file and no product.
        assert!(TITLE_SCRIPT.contains("dioxus.recv()"));
        assert!(!TITLE_SCRIPT.contains(PRODUCT_NAME));
        assert!(!TITLE_SCRIPT.contains('"') && !TITLE_SCRIPT.contains('\''));
    }
}
