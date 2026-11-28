//! External file modification dialog (RFC-015).
//!
//! Shown when the file on disk is no longer what the document was made from (its bytes
//! differ from those omriss last read or wrote, RFC-068 §7.6), before Omriss overwrites
//! it with the in-memory canonical text. Focus starts on Cancel, the choice that changes
//! nothing; Reload is never the default (RFC-060 §9.3).

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

/// The user's choice when an external modification is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtModifiedChoice {
    /// Overwrite the disk file with in-memory canonical text.
    Overwrite,
    /// Replace the document with the file as it is on disk (RFC-068 §7.7). One undo
    /// step: Ctrl+Z brings the unsaved text back. Never the default.
    Reload,
    /// Open a Save As dialog to choose a different path.
    SaveAs,
    /// Cancel the save operation; do not touch the disk file.
    Cancel,
}

/// Modal dialog displayed when the file on disk changed externally.
#[component]
pub fn ExtModifiedDialog(
    locale: Signal<Locale>,
    on_choice: EventHandler<ExtModifiedChoice>,
) -> Element {
    let lang = *locale.read();
    rsx! {
        div {
            class: "modal-overlay",
            "data-focus-scope": "contain",
            "data-focus-arrows": "true",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "extmod-title",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "extmod-title", {t(lang, "dialog.ext_modified.title")} }
                p { {t(lang, "dialog.ext_modified.body")} }
                div { class: "modal-actions",
                    button {
                        class: "primary",
                        onclick: move |_| on_choice.call(ExtModifiedChoice::Overwrite),
                        {t(lang, "dialog.ext_modified.overwrite")}
                    }
                    button {
                        onclick: move |_| on_choice.call(ExtModifiedChoice::Reload),
                        {t(lang, "dialog.ext_modified.reload")}
                    }
                    button {
                        onclick: move |_| on_choice.call(ExtModifiedChoice::SaveAs),
                        {t(lang, "dialog.ext_modified.save_as")}
                    }
                    button {
                        autofocus: true,
                        onclick: move |_| on_choice.call(ExtModifiedChoice::Cancel),
                        {t(lang, "dialog.ext_modified.cancel")}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use omriss_ui::i18n::{Locale, t};

    /// This file's own source: the dialog is small and its rule is about which button
    /// is which, so the rule is read where it is written.
    const SOURCE: &str = include_str!("ext_modified_dialog.rs");

    /// Only the component, not this module (which names the strings it looks for).
    fn code() -> &'static str {
        SOURCE.split("#[cfg(test)]").next().unwrap()
    }

    fn button_block(choice: &str) -> &'static str {
        let code = code();
        let at = code
            .find(&format!("ExtModifiedChoice::{choice}),\n"))
            .and_then(|i| code[..i].rfind("button {"))
            .unwrap_or_else(|| panic!("no button for {choice}"));
        let rest = &code[at..];
        &rest[..rest.find("\n                    }").unwrap()]
    }

    #[test]
    fn focus_starts_on_cancel_and_nowhere_else() {
        assert_eq!(code().matches("autofocus: true").count(), 1);
        assert!(button_block("Cancel").contains("autofocus: true"));
    }

    #[test]
    fn reload_is_a_choice_but_never_the_default_or_the_primary() {
        let reload = button_block("Reload");
        assert!(reload.contains("dialog.ext_modified.reload"));
        assert!(!reload.contains("autofocus"));
        assert!(!reload.contains("primary"));
    }

    #[test]
    fn the_words_exist_in_both_locales_and_say_what_reload_does_and_that_undo_returns_the_text() {
        for lang in [Locale::En, Locale::Ja] {
            for key in [
                "dialog.ext_modified.reload",
                "dialog.ext_modified.body",
                "status.reloaded",
            ] {
                assert_ne!(t(lang, key), key, "{key} missing in {lang:?}");
            }
            assert!(t(lang, "dialog.ext_modified.body").contains("Ctrl+Z"));
            assert!(t(lang, "status.reloaded").contains("Ctrl+Z"));
        }
        assert_eq!(
            t(Locale::En, "dialog.ext_modified.reload"),
            "Reload from disk"
        );
        assert!(t(Locale::Ja, "dialog.ext_modified.reload").contains("再読み込み"));
        // The status is not an error: the document is fine and saved as the disk has it.
        assert!(!"status.reloaded".starts_with("error."));
    }
}
