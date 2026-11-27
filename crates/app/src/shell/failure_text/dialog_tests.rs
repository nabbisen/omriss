//! RFC-073 §8, level 2: the open-failure dialog per variant, in both locales —
//! mounted in a bare Dioxus `VirtualDom`, so the tree is what the WebView is given.
//! Its text is the catalog's; its `aria-describedby` target is the same text, so what
//! reaches assistive technology is what is on screen (RFC-060 F-5).

use dioxus::dioxus_core::{DynamicNode, TemplateNode, VNode};
use dioxus::prelude::*;
use omriss_components::ErrorDialog;
use omriss_ui::i18n::{Locale, t};

use super::open_failure_message;
use crate::file::failure::OpenFailure;

thread_local! {
    static CASE: std::cell::RefCell<Option<(OpenFailure, Locale)>> = const { std::cell::RefCell::new(None) };
}

fn host() -> Element {
    let (failure, locale) = CASE.with(|c| c.borrow().clone().unwrap());
    rsx! {
        ErrorDialog {
            locale: use_signal(move || locale),
            title: "error.open_failed".to_string(),
            cause: open_failure_message(locale, &failure),
            dismiss_label: t(locale, "dialog.discard.cancel").to_string(),
            secondary_label: None,
            on_dismiss: move |()| {},
            on_secondary: None,
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
