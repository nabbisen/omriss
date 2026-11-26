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

use crate::toolbar::SettingsPopover;
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
fn the_search_panel_is_drawn_over_the_page_so_it_contains_focus() {
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
    // RFC-059 §12.3a: `.search-overlay` is `position: fixed` over the page, so
    // Tab out of it would reach controls hidden underneath. No arrows: they are
    // the results' own.
    assert_eq!(scope_of(host), (Some("contain".into()), false));
}

#[test]
fn the_settings_popover_is_drawn_over_the_page_so_it_contains_focus() {
    fn host() -> Element {
        rsx! { SettingsPopover { locale: use_signal(Locale::default), on_close: |_| {} } }
    }
    let attrs = overlay_attrs(host);
    assert_eq!(attr(&attrs, "data-focus-scope").as_deref(), Some("contain"));
    // It holds a label and a select, not menu items.
    assert_ne!(attr(&attrs, "role").as_deref(), Some("menu"));
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

// ── RFC-059 §12.3: every view renders at most one `data-focus-primary` ──────

use omriss_core::{DocumentFormat, StructureNodeKind};
use omriss_ui::{DocumentMapNode, FileTextProfile, node_id_from_raw};

use crate::{FocusedContentPane, OverviewPane, RawSourceView, WelcomeScreen};

/// Every `data-focus-primary` in the mounted tree below `node`, as
/// `tag` or `tag.class`. Walks templates, components and fragments: the whole
/// view, not one overlay.
fn collect_primaries(dom: &VirtualDom, node: &VNode, out: &mut Vec<String>) {
    fn walk(t: &TemplateNode, out: &mut Vec<String>) {
        if let TemplateNode::Element {
            tag,
            attrs,
            children,
            ..
        } = t
        {
            let get = |wanted: &str| {
                attrs.iter().find_map(|a| match a {
                    TemplateAttribute::Static { name, value, .. } if *name == wanted => {
                        Some(value.to_string())
                    }
                    _ => None,
                })
            };
            if get("data-focus-primary").is_some() {
                out.push(match get("class") {
                    Some(c) => format!("{tag}.{c}"),
                    None => tag.to_string(),
                });
            }
            for child in children.iter() {
                walk(child, out);
            }
        }
    }
    for root in node.template.roots.iter() {
        walk(root, out);
    }
    for (i, d) in node.dynamic_nodes.iter().enumerate() {
        match d {
            DynamicNode::Component(c) => {
                if let Some(scope) = c.mounted_scope(i, node, dom) {
                    collect_primaries(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Fragment(children) => {
                for child in children {
                    collect_primaries(dom, child, out);
                }
            }
            _ => {}
        }
    }
}

fn primaries(host: fn() -> Element) -> Vec<String> {
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    let mut out = Vec::new();
    collect_primaries(&dom, dom.base_scope().root_node(), &mut out);
    out
}

const MD: &str = "# One\nbody\n\n# Two\nmore\n";

fn markdown_focused(preview: bool) -> Element {
    let mut session = use_signal(|| EditorSession::open(MD.into(), None).unwrap());
    let one = session
        .read()
        .document_map_nodes()
        .children
        .iter()
        .find(|n| n.title == "One")
        .map(|n| node_id_from_raw(n.id))
        .unwrap();
    use_hook(|| session.write().focus(one).unwrap());
    rsx! {
        FocusedContentPane {
            session,
            locale: use_signal(Locale::default),
            draft: use_signal(|| String::from("body\n")),
            status: use_signal(String::new),
            preview_open: use_signal(move || preview),
        }
    }
}

fn find_kind(node: &DocumentMapNode, kind: StructureNodeKind) -> Option<u64> {
    if node.kind == kind {
        return Some(node.id);
    }
    node.children.iter().find_map(|c| find_kind(c, kind))
}

fn json_session() -> EditorSession {
    let text = "{\"name\": \"omriss\", \"tags\": [\"a\"]}\n";
    EditorSession::open_detected(
        text.into(),
        Some("x.json".into()),
        FileTextProfile::detect(text, false),
        DocumentFormat::Json,
    )
    .unwrap()
}

#[test]
fn a_focused_markdown_section_has_exactly_the_body_editor_as_primary() {
    fn host() -> Element {
        markdown_focused(false)
    }
    assert_eq!(primaries(host), ["textarea.body-editor"]);
}

#[test]
fn the_preview_has_no_primary_surface() {
    // The preview is a `panel` scope that takes its own initial focus; when it
    // closes the editor returns and is the primary again.
    fn host() -> Element {
        markdown_focused(true)
    }
    assert!(primaries(host).is_empty(), "{:?}", primaries(host));
}

#[test]
fn a_focused_json_text_value_has_its_text_area_as_primary() {
    fn host() -> Element {
        let mut session = use_signal(json_session);
        let id = find_kind(
            &session.read().document_map_nodes(),
            StructureNodeKind::Value,
        )
        .unwrap();
        use_hook(|| session.write().focus(node_id_from_raw(id)).unwrap());
        rsx! {
            FocusedContentPane {
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(String::new),
                status: use_signal(String::new),
                preview_open: use_signal(|| false),
            }
        }
    }
    // A string value is a text area since RFC-084 §10.2 (it was a one-line input).
    assert_eq!(primaries(host), ["textarea.structured-value-input"]);
}

#[test]
fn a_focused_json_number_has_its_one_line_input_as_primary() {
    fn host() -> Element {
        let mut session = use_signal(|| {
            let text = "{\"count\": 3}\n";
            EditorSession::open_detected(
                text.into(),
                Some("x.json".into()),
                FileTextProfile::detect(text, false),
                DocumentFormat::Json,
            )
            .unwrap()
        });
        let id = find_kind(
            &session.read().document_map_nodes(),
            StructureNodeKind::Value,
        )
        .unwrap();
        use_hook(|| session.write().focus(node_id_from_raw(id)).unwrap());
        rsx! {
            FocusedContentPane {
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(String::new),
                status: use_signal(String::new),
                preview_open: use_signal(|| false),
            }
        }
    }
    assert_eq!(primaries(host), ["input"]);
}

#[test]
fn a_focused_json_group_has_no_primary_until_its_text_is_opened() {
    fn host() -> Element {
        let mut session = use_signal(json_session);
        let root = find_kind(
            &session.read().document_map_nodes(),
            StructureNodeKind::List,
        )
        .unwrap();
        use_hook(|| session.write().focus(node_id_from_raw(root)).unwrap());
        rsx! {
            FocusedContentPane {
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(String::new),
                status: use_signal(String::new),
                preview_open: use_signal(|| false),
            }
        }
    }
    assert!(primaries(host).is_empty(), "{:?}", primaries(host));
}

#[test]
fn read_only_plain_file_text_has_its_pre_as_primary() {
    fn host() -> Element {
        let mut session = use_signal(|| EditorSession::open(MD.into(), None).unwrap());
        use_hook(|| session.write().show_raw());
        rsx! {
            RawSourceView {
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(String::new),
                status: use_signal(String::new),
            }
        }
    }
    assert_eq!(primaries(host), ["pre.raw-pre"]);
}

#[test]
fn editing_plain_file_text_has_its_editor_as_primary() {
    fn host() -> Element {
        let mut session = use_signal(|| EditorSession::open(MD.into(), None).unwrap());
        use_hook(|| {
            session.write().show_raw();
            session.write().begin_raw_edit();
        });
        rsx! {
            RawSourceView {
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(|| MD.to_string()),
                status: use_signal(String::new),
            }
        }
    }
    assert_eq!(primaries(host), ["textarea.raw-editor"]);
}

#[test]
fn the_outline_and_the_welcome_screen_have_no_primary_so_focus_falls_to_the_app_root() {
    fn outline() -> Element {
        let session = use_signal(|| EditorSession::open(MD.into(), None).unwrap());
        rsx! {
            OverviewPane {
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(String::new),
                status: use_signal(String::new),
                selected_card: use_signal(|| 0usize),
            }
        }
    }
    fn welcome() -> Element {
        rsx! {
            WelcomeScreen {
                locale: use_signal(Locale::default),
                recent_files: use_signal(Vec::new),
                on_open: |_| {},
                on_new: |_| {},
                on_open_recent: |_| {},
            }
        }
    }
    assert!(primaries(outline).is_empty());
    assert!(primaries(welcome).is_empty());
}

#[test]
fn the_read_only_text_has_a_translated_accessible_name_in_both_locales() {
    use omriss_ui::i18n::t;
    for locale in [Locale::En, Locale::Ja] {
        let name = t(locale, "raw.source_label");
        assert_ne!(name, "raw.source_label", "{locale:?} has no translation");
    }
    assert_ne!(
        t(Locale::En, "raw.source_label"),
        t(Locale::Ja, "raw.source_label")
    );
}
