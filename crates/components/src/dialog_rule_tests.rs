//! RFC-068 §10, level 2: in every dialog, a choice that can lose the user's text, or
//! another program's text, is drawn as danger and is never primary. Primary is only
//! ever a safe choice.
//!
//! Focus starting on Cancel is not enough: the eye goes to the filled button. (*File
//! Changed on Disk* drew Overwrite as the filled blue one, the choice Undo cannot repair.)
//!
//! Each dialog is mounted bare, in both languages, and its buttons read with their
//! class and their words. Two lists decide what each button is, and a button in neither
//! fails the test: a new dialog, or a new choice, has to be classified by whoever adds it.

use std::cell::Cell;

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

use crate::test_texts::{Rendered, elements_of_settled};
use crate::{
    ConfirmDeleteDialog, ErrorDialog, ExtModifiedDialog, SectionTitleAction, SectionTitleDialog,
    UnsavedDialog,
};

/// The labels (catalog keys) of the choices that can lose text: danger, never primary.
const LOSES_TEXT: &[&str] = &[
    "dialog.ext_modified.overwrite",
    "dialog.unsaved.discard",
    "dialog.confirm_delete.confirm",
];

/// The labels of the choices that are safe, and so may be primary. A primary button not
/// listed here, or in `LOSES_TEXT`, fails: it has not been classified.
const SAFE_PRIMARY: &[&str] = &[
    "dialog.unsaved.save",
    "dialog.section_title.add.confirm",
    "dialog.section_title.rename.confirm",
];

#[derive(Clone, Copy, Debug)]
enum Dialog {
    Unsaved,
    ExtModified,
    ConfirmDelete,
    SectionTitle(SectionTitleAction),
    /// An error with a recovery action beside Dismiss ("Save As").
    ErrorWithRecovery,
    /// An error with only its Close button.
    ErrorPlain,
}

const DIALOGS: [Dialog; 9] = [
    Dialog::Unsaved,
    Dialog::ExtModified,
    Dialog::ConfirmDelete,
    Dialog::SectionTitle(SectionTitleAction::AddTopLevel),
    Dialog::SectionTitle(SectionTitleAction::AddInside),
    Dialog::SectionTitle(SectionTitleAction::AddAfter),
    Dialog::SectionTitle(SectionTitleAction::Rename),
    Dialog::ErrorWithRecovery,
    Dialog::ErrorPlain,
];

thread_local! {
    static CASE: Cell<(Dialog, Locale)> = const { Cell::new((Dialog::Unsaved, Locale::En)) };
}

fn host() -> Element {
    let (dialog, lang) = CASE.get();
    let locale = use_signal(|| lang);
    match dialog {
        Dialog::Unsaved => rsx! { UnsavedDialog { locale, on_choice: |_| {} } },
        Dialog::ExtModified => rsx! { ExtModifiedDialog { locale, on_choice: |_| {} } },
        Dialog::ConfirmDelete => rsx! {
            ConfirmDeleteDialog {
                locale,
                section_title: "One".to_string(),
                child_count: 2usize,
                on_choice: |_| {},
            }
        },
        Dialog::SectionTitle(action) => rsx! {
            SectionTitleDialog {
                locale,
                action,
                initial_title: String::new(),
                on_choice: |_| {},
            }
        },
        Dialog::ErrorWithRecovery => rsx! {
            ErrorDialog {
                locale,
                title: "error.open_failed".to_string(),
                message: "A message.".to_string(),
                dismiss_label: "Close".to_string(),
                secondary_label: Some("Save As".to_string()),
                on_dismiss: |_| {},
                on_secondary: Some(EventHandler::new(|_| {})),
            }
        },
        Dialog::ErrorPlain => rsx! {
            ErrorDialog {
                locale,
                title: "error.open_failed".to_string(),
                message: "A message.".to_string(),
                dismiss_label: "Close".to_string(),
                secondary_label: None,
                on_dismiss: |_| {},
                on_secondary: None,
            }
        },
    }
}

fn buttons(dialog: Dialog, lang: Locale) -> Vec<Rendered> {
    CASE.set((dialog, lang));
    elements_of_settled(host)
        .into_iter()
        .filter(|e| e.tag == "button")
        .collect()
}

fn is_primary(b: &Rendered) -> bool {
    b.attr("class")
        .is_some_and(|c| c.split_whitespace().any(|w| w == "primary"))
}

fn is_danger(b: &Rendered) -> bool {
    b.attr("class")
        .is_some_and(|c| c.split_whitespace().any(|w| w == "btn-danger"))
}

#[test]
fn no_choice_that_can_lose_text_is_primary_and_each_is_danger() {
    let mut seen = Vec::new();
    for dialog in DIALOGS {
        for lang in [Locale::En, Locale::Ja] {
            for b in buttons(dialog, lang) {
                for key in LOSES_TEXT {
                    if b.text == t(lang, key) {
                        seen.push(*key);
                        assert!(!is_primary(&b), "{key} is primary in {dialog:?} ({lang:?})");
                        assert!(
                            is_danger(&b),
                            "{key} is not danger in {dialog:?} ({lang:?})"
                        );
                    }
                }
            }
        }
    }
    // Every destructive choice was found in some dialog: the list names real buttons.
    for key in LOSES_TEXT {
        assert!(
            seen.contains(key),
            "no dialog shows {key}: is the list stale?"
        );
    }
}

#[test]
fn primary_is_only_ever_a_safe_choice() {
    for dialog in DIALOGS {
        for lang in [Locale::En, Locale::Ja] {
            for b in buttons(dialog, lang) {
                if !is_primary(&b) {
                    continue;
                }
                let safe = SAFE_PRIMARY.iter().any(|key| b.text == t(lang, key))
                    // The recovery action an error offers, "Save As": it writes
                    // somewhere else and loses nothing.
                    || (matches!(dialog, Dialog::ErrorWithRecovery) && b.text == "Save As");
                assert!(
                    safe,
                    "{:?} is primary in {dialog:?} ({lang:?}) but is not a classified safe choice",
                    b.text
                );
            }
        }
    }
}

#[test]
fn a_danger_button_is_never_the_one_focus_starts_on() {
    // Focus starts on the choice that changes nothing.
    for dialog in DIALOGS {
        for b in buttons(dialog, Locale::En) {
            if b.attr("autofocus").is_some() {
                assert!(!is_danger(&b), "{:?} takes focus in {dialog:?}", b.text);
            }
        }
    }
}

#[test]
fn the_unsaved_dialog_keeps_save_primary_and_the_disk_dialog_has_no_primary_at_all() {
    for lang in [Locale::En, Locale::Ja] {
        let unsaved = buttons(Dialog::Unsaved, lang);
        let save = unsaved
            .iter()
            .find(|b| b.text == t(lang, "dialog.unsaved.save"))
            .unwrap();
        assert!(is_primary(save));
        // Overwrite was the filled button; with it danger, no choice there is filled blue.
        assert!(
            buttons(Dialog::ExtModified, lang)
                .iter()
                .all(|b| !is_primary(b))
        );
    }
}
