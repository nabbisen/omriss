//! RFC-073 §8, level 2: the open-failure dialog per variant, in both locales —
//! mounted in a bare Dioxus `VirtualDom`, so the tree is what the WebView is given.
//! Its text is the catalog's; its `aria-describedby` target is the same text, so what
//! reaches assistive technology is what is on screen (RFC-060 F-5).

use dioxus::dioxus_core::{DynamicNode, TemplateNode, VNode};
use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};

const STYLE: &str = include_str!("../../../assets/style.css");

use super::open_failure_message;
use crate::file::failure::OpenFailure;
use crate::shell::open_error::OpenErrorDialog;

thread_local! {
    static CASE: std::cell::RefCell<Option<(OpenFailure, Locale)>> = const { std::cell::RefCell::new(None) };
}

/// The shell's own dialog, mounted as the shell mounts it.
fn host() -> Element {
    let (failure, locale) = CASE.with(|c| c.borrow().clone().unwrap());
    rsx! {
        OpenErrorDialog {
            locale: use_signal(move || locale),
            failure,
            on_dismiss: move |()| {},
        }
    }
}

fn walk(t_: &TemplateNode, out: &mut Vec<String>) {
    match t_ {
        TemplateNode::Text { text } => out.push(text.to_string()),
        TemplateNode::Element { children, .. } => children.iter().for_each(|c| walk(c, out)),
        TemplateNode::Dynamic { .. } => {}
    }
}

fn collect(dom: &VirtualDom, node: &VNode, out: &mut Vec<String>) {
    node.template.roots.iter().for_each(|r| walk(r, out));
    for (i, d) in node.dynamic_nodes.iter().enumerate() {
        match d {
            DynamicNode::Text(t_) => out.push(t_.value.clone()),
            DynamicNode::Component(c) => {
                if let Some(scope) = c.mounted_scope(i, node, dom) {
                    collect(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Fragment(children) => children.iter().for_each(|c| collect(dom, c, out)),
            DynamicNode::Placeholder(_) => {}
        }
    }
}

fn dialog_texts(failure: OpenFailure, locale: Locale) -> Vec<String> {
    CASE.with(|c| *c.borrow_mut() = Some((failure, locale)));
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    let mut out = Vec::new();
    collect(&dom, dom.base_scope().root_node(), &mut out);
    out
}

#[test]
fn every_variant_shows_its_catalog_text_in_both_locales_and_nothing_from_the_os() {
    let failures = [
        OpenFailure::NotFound {
            path: "/tmp/gone.md".into(),
        },
        OpenFailure::NotUtf8,
        OpenFailure::Io {
            kind: std::io::ErrorKind::PermissionDenied,
        },
        OpenFailure::Io {
            kind: std::io::ErrorKind::IsADirectory,
        },
        OpenFailure::Io {
            kind: std::io::ErrorKind::TimedOut,
        },
        OpenFailure::Internal,
        OpenFailure::CommandLineOption,
    ];
    for locale in [Locale::En, Locale::Ja] {
        for failure in failures.clone() {
            let texts = dialog_texts(failure.clone(), locale);
            let cause = open_failure_message(locale, &failure);
            assert!(texts.contains(&cause), "{locale:?} {failure:?}: {texts:?}");
            assert!(texts.contains(&t(locale, "error.open_failed").to_string()));
            let all = texts.join("\n");
            assert!(
                !all.contains("os error") && !all.contains("Permission denied"),
                "no OS text: {all}"
            );
        }
    }
}

#[test]
fn the_permission_case_rfc_060_measured_says_it_in_the_users_language() {
    let en = open_failure_message(
        Locale::En,
        &OpenFailure::Io {
            kind: std::io::ErrorKind::PermissionDenied,
        },
    );
    let ja = open_failure_message(
        Locale::Ja,
        &OpenFailure::Io {
            kind: std::io::ErrorKind::PermissionDenied,
        },
    );
    assert!(en.contains("permission"), "{en}");
    assert!(ja.contains("権限"), "{ja}");
    assert_ne!(en, ja);
}

// ── RFC-073 §9: the dialog around the message ──────────────────────────────

const ALL: fn() -> Vec<OpenFailure> = || {
    vec![
        OpenFailure::NotFound {
            path: "/tmp/gone.md".into(),
        },
        OpenFailure::NotUtf8,
        OpenFailure::Io {
            kind: std::io::ErrorKind::PermissionDenied,
        },
        OpenFailure::Internal,
        OpenFailure::CommandLineOption,
    ]
};

#[test]
fn the_title_says_what_happened_and_gives_no_advice() {
    for locale in [Locale::En, Locale::Ja] {
        for failure in ALL() {
            let texts = dialog_texts(failure.clone(), locale);
            let title = t(locale, "error.open_failed");
            assert!(texts.contains(&title.to_string()), "{locale:?}: {texts:?}");
            // The advice is said once, in the message: not "Try another file" above a
            // message that may say "use Open to choose it again".
            assert!(
                !title.contains("Try") && !title.contains("お試し"),
                "{title}"
            );
        }
    }
    assert_eq!(
        t(Locale::En, "error.open_failed"),
        "This file could not be opened."
    );
}

#[test]
fn the_button_says_close_from_its_own_key_not_the_unsaved_dialogs_cancel() {
    for locale in [Locale::En, Locale::Ja] {
        let close = t(locale, "dialog.close");
        assert_ne!(close, "dialog.close", "the key is in the catalog");
        assert_ne!(
            close,
            t(locale, "dialog.discard.cancel"),
            "not borrowed from Cancel"
        );
        for failure in ALL() {
            let texts = dialog_texts(failure, locale);
            assert!(texts.contains(&close.to_string()), "{locale:?}: {texts:?}");
            assert!(
                !texts.contains(&t(locale, "dialog.discard.cancel").to_string()),
                "{texts:?}"
            );
        }
    }
    assert_eq!(t(Locale::En, "dialog.close"), "Close");
    assert_eq!(t(Locale::Ja, "dialog.close"), "閉じる");
}

/// The text of the CSS rule for `selector` (up to its closing brace).
fn rule(selector: &str) -> String {
    let start = STYLE
        .find(&format!("{selector} {{"))
        .unwrap_or_else(|| panic!("no rule for {selector}"));
    let rest = &STYLE[start..];
    rest[..rest.find('}').unwrap() + 1].to_string()
}

#[test]
fn the_message_is_body_text_not_a_technical_cause_box() {
    // The rule the message is drawn by: normal size, the full text colour, no
    // monospace, no box (the old `.error-cause` was a 12px monospace code box).
    // `.modal p` (specificity 0,1,1) mutes a paragraph; `.modal .error-message` (0,2,0)
    // outranks it, which a bare `.error-message` (0,1,0) would not.
    let css = rule(".modal .error-message");
    assert!(css.contains("color: var(--text)"), "{css}");
    for forbidden in ["monospace", "font-size", "border", "background"] {
        assert!(
            !css.contains(forbidden),
            "`{forbidden}` makes it look like a code excerpt: {css}"
        );
    }
    assert!(!STYLE.contains(".error-cause"), "the old class is gone");

    // And the dialog draws the message with that class, and describes itself by it.
    for locale in [Locale::En, Locale::Ja] {
        let source = include_str!("../../../../components/src/error_dialog.rs");
        assert!(source.contains("class: \"error-message\""));
        assert!(source.contains("\"aria-describedby\": \"err-message\""));
        let texts = dialog_texts(OpenFailure::NotUtf8, locale);
        assert!(texts.contains(&open_failure_message(locale, &OpenFailure::NotUtf8)));
    }
}
