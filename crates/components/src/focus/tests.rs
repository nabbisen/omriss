//! RFC-059 §12.1, level 2: every overlay declares its focus scope in markup,
//! and the script that acts on it exists only in `focus.rs`.
//!
//! The scope attributes are static, so they live in each component's *template*
//! rather than in any mutation. Each test mounts the overlay in a bare Dioxus
//! `VirtualDom` and reads its root element's attributes from the mounted
//! template, which is exactly what the WebView would be given.

use dioxus::dioxus_core::{DynamicNode, Template, TemplateAttribute, TemplateNode, VNode};
use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::Locale;

use crate::{
    CommandPalette, ConfirmDeleteDialog, ErrorDialog, ExtModifiedDialog, PreviewPane, SearchPanel,
    SectionTitleAction, SectionTitleDialog, UnsavedDialog,
};

/// The static attributes of the first element a template renders.
fn static_attrs(template: &Template) -> Vec<(String, String)> {
    match template.roots.first() {
        Some(TemplateNode::Element { attrs, .. }) => attrs
            .iter()
            .filter_map(|a| match a {
                TemplateAttribute::Static { name, value, .. } => {
                    Some((name.to_string(), value.to_string()))
                }
                TemplateAttribute::Dynamic { .. } => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The first VNode below `host` whose template starts with a real element.
/// Dioxus wraps the root in a suspense and an error boundary, each of which
/// holds its output in a fragment, and the macro generates a wrapper component
/// around every `#[component]`; this looks through all of them.
pub(crate) fn first_element_root<'a>(dom: &'a VirtualDom, host: &'a VNode) -> Option<&'a VNode> {
    if matches!(
        host.template.roots.first(),
        Some(TemplateNode::Element { .. })
    ) {
        return Some(host);
    }
    for (i, node) in host.dynamic_nodes.iter().enumerate() {
        let found = match node {
            DynamicNode::Component(c) => c
                .mounted_scope(i, host, dom)
                .and_then(|scope| first_element_root(dom, scope.root_node())),
            DynamicNode::Fragment(children) => children
                .iter()
                .find_map(|child| first_element_root(dom, child)),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Mounts `host` (a component that renders one overlay) and returns the static
/// attributes of the overlay's root element.
fn overlay_attrs(host: fn() -> Element) -> Vec<(String, String)> {
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    let overlay = first_element_root(&dom, dom.base_scope().root_node())
        .expect("the overlay renders an element");
    static_attrs(&overlay.template)
}

fn attr(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.clone())
}

/// `(scope, has data-focus-arrows)` of an overlay.
fn scope_of(host: fn() -> Element) -> (Option<String>, bool) {
    let attrs = overlay_attrs(host);
    (
        attr(&attrs, "data-focus-scope"),
        attr(&attrs, "data-focus-arrows").is_some(),
    )
}

fn contain_with_arrows() -> (Option<String>, bool) {
    (Some("contain".into()), true)
}

#[test]
fn the_unsaved_dialog_contains_focus_and_takes_arrows() {
    fn host() -> Element {
        rsx! { UnsavedDialog { locale: use_signal(Locale::default), on_choice: |_| {} } }
    }
    assert_eq!(scope_of(host), contain_with_arrows());
}

#[test]
fn the_external_modification_dialog_contains_focus_and_takes_arrows() {
    fn host() -> Element {
        rsx! { ExtModifiedDialog { locale: use_signal(Locale::default), on_choice: |_| {} } }
    }
    assert_eq!(scope_of(host), contain_with_arrows());
}

#[test]
fn the_confirm_delete_dialog_contains_focus_and_takes_arrows() {
    fn host() -> Element {
        rsx! {
            ConfirmDeleteDialog {
                locale: use_signal(Locale::default),
                section_title: "One".to_string(),
                child_count: 0usize,
                on_choice: |_| {},
            }
        }
    }
    assert_eq!(scope_of(host), contain_with_arrows());
}

#[test]
fn the_error_dialog_contains_focus_and_takes_arrows() {
    fn host() -> Element {
        rsx! {
            ErrorDialog {
                locale: use_signal(Locale::default),
                title: "x".to_string(),
                cause: "y".to_string(),
                dismiss_label: "OK".to_string(),
                secondary_label: None,
                on_dismiss: |_| {},
                on_secondary: None,
            }
        }
    }
    assert_eq!(scope_of(host), contain_with_arrows());
}

#[test]
fn the_section_title_dialog_contains_focus_and_takes_arrows() {
    fn host() -> Element {
        rsx! {
            SectionTitleDialog {
                locale: use_signal(Locale::default),
                action: SectionTitleAction::Rename,
                initial_title: String::new(),
                on_choice: |_| {},
            }
        }
    }
    assert_eq!(scope_of(host), contain_with_arrows());
}

#[test]
fn quick_actions_contains_focus_but_keeps_its_arrows_for_the_active_result() {
    fn host() -> Element {
        rsx! {
            CommandPalette {
                locale: use_signal(Locale::default),
                on_close: |_| {},
                on_execute: |_| {},
            }
        }
    }
    // §12.1: no `data-focus-arrows` on Quick Actions — its arrows move the
    // active result, in Rust.
    assert_eq!(scope_of(host), (Some("contain".into()), false));
}

#[test]
fn the_search_panel_takes_initial_focus_but_does_not_contain_tab() {
    fn host() -> Element {
        let session = use_signal(|| EditorSession::open("# One\nbody\n".into(), None).unwrap());
        rsx! {
            SearchPanel {
                session,
                locale: use_signal(Locale::default),
                on_close: |_| {},
                on_navigate: |_| {},
            }
        }
    }
    assert_eq!(scope_of(host), (Some("panel".into()), false));
}

#[test]
fn the_preview_takes_initial_focus_but_does_not_contain_tab() {
    fn host() -> Element {
        let session = use_signal(|| EditorSession::open("# One\nbody\n".into(), None).unwrap());
        rsx! {
            PreviewPane {
                session,
                locale: use_signal(Locale::default),
                on_close: |_| {},
            }
        }
    }
    assert_eq!(scope_of(host), (Some("panel".into()), false));
}

#[test]
fn the_scanner_reads_a_scope_and_misses_one_that_is_absent() {
    // The harness itself: a component with no scope reports none, so the tests
    // above are not passing vacuously.
    fn plain() -> Element {
        rsx! { div { class: "modal-overlay" } }
    }
    fn host() -> Element {
        rsx! { Plain {} }
    }
    #[component]
    fn Plain() -> Element {
        plain()
    }
    assert_eq!(scope_of(host), (None, false));
}
