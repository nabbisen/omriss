//! Section-title dialog for Document Map structure actions (RFC-049).

use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionTitleChoice {
    Confirm(String),
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionTitleAction {
    AddTopLevel,
    AddInside,
    AddAfter,
    Rename,
}

impl SectionTitleAction {
    fn title_key(self) -> &'static str {
        match self {
            Self::AddTopLevel => "dialog.section_title.add_top_level.title",
            Self::AddInside => "dialog.section_title.add_inside.title",
            Self::AddAfter => "dialog.section_title.add_after.title",
            Self::Rename => "dialog.section_title.rename.title",
        }
    }

    fn confirm_key(self) -> &'static str {
        match self {
            Self::AddTopLevel | Self::AddInside | Self::AddAfter => {
                "dialog.section_title.add.confirm"
            }
            Self::Rename => "dialog.section_title.rename.confirm",
        }
    }
}

#[component]
pub fn SectionTitleDialog(
    locale: Signal<Locale>,
    action: SectionTitleAction,
    initial_title: String,
    on_choice: EventHandler<SectionTitleChoice>,
) -> Element {
    let lang = *locale.read();
    let mut title = use_signal(move || initial_title);

    rsx! {
        div {
            class: "modal-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "section-title-label",
            tabindex: "-1",
            div { class: "modal",
                h2 { id: "section-title-label", {t(lang, action.title_key())} }
                input {
                    class: "search-input split-dialog-input",
                    r#type: "text",
                    placeholder: t(lang, "dialog.section_title.placeholder"),
                    value: "{title}",
                    oninput: move |evt| title.set(evt.value()),
                    onkeydown: move |evt| {
                        use keyboard_types::Code;
                        match evt.data().code() {
                            Code::Enter if !title.read().trim().is_empty() => {
                                let t = title.read().trim().to_string();
                                on_choice.call(SectionTitleChoice::Confirm(t));
                            }
                            Code::Escape => on_choice.call(SectionTitleChoice::Cancel),
                            _ => {}
                        }
                    },
                }
                div { class: "modal-actions",
                    button {
                        class: "primary",
                        disabled: title.read().trim().is_empty(),
                        onclick: {
                            move |_| {
                                let t = title.read().trim().to_string();
                                if !t.is_empty() {
                                    on_choice.call(SectionTitleChoice::Confirm(t));
                                }
                            }
                        },
                        {t(lang, action.confirm_key())}
                    }
                    button {
                        onclick: move |_| on_choice.call(SectionTitleChoice::Cancel),
                        {t(lang, "dialog.section_title.cancel")}
                    }
                }
            }
        }
    }
}
