//! RFC-084 §10.2, level 2: which field each kind of JSON value is given. A string
//! is edited in a text area (a line break can be shown, kept and typed), a
//! number in the one-line field. Mounted in a bare Dioxus `VirtualDom`; the
//! template tree is exactly what the WebView is given.

use dioxus::dioxus_core::{DynamicNode, TemplateAttribute, TemplateNode, VNode};
use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::Locale;
use omriss_ui::{DocumentMapNode, EditorSession, FileTextProfile, node_id_from_raw};

use super::StructuredFocusView;

const DOC: &str = r#"{"text": "line1\nline2", "count": 3, "on": true, "none": null, "list": [1]}"#;

thread_local! {
    static CASE: std::cell::Cell<&'static str> = const { std::cell::Cell::new("text") };
}

fn find_title(node: &DocumentMapNode, title: &str) -> Option<u64> {
    if node.title == title {
        return Some(node.id);
    }
    node.children.iter().find_map(|c| find_title(c, title))
}

fn host() -> Element {
    let focus = CASE.with(|c| c.get());
    let mut session = use_signal(|| {
        EditorSession::open_detected(
            DOC.to_string(),
            Some("x.json".into()),
            FileTextProfile::detect(DOC, false),
            DocumentFormat::Json,
        )
        .unwrap()
    });
    let id = find_title(&session.read().document_map_nodes(), focus).unwrap();
    use_hook(|| session.write().focus(node_id_from_raw(id)).unwrap());
    rsx! {
        StructuredFocusView {
            session,
            locale: use_signal(|| Locale::En),
            draft: use_signal(String::new),
            status: use_signal(String::new),
        }
    }
}

fn attr(attrs: &[TemplateAttribute], name: &str) -> Option<String> {
    attrs.iter().find_map(|a| match a {
        TemplateAttribute::Static { name: n, value, .. } if *n == name => Some(value.to_string()),
        _ => None,
    })
}

/// `(tag, class)` of every element in the mounted tree.
fn elements(dom: &VirtualDom, node: &VNode, out: &mut Vec<(String, String)>) {
    fn walk(t: &TemplateNode, out: &mut Vec<(String, String)>) {
        if let TemplateNode::Element {
            tag,
            attrs,
            children,
            ..
        } = t
        {
            out.push((tag.to_string(), attr(attrs, "class").unwrap_or_default()));
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
                    elements(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Fragment(children) => {
                for child in children {
                    elements(dom, child, out);
                }
            }
            _ => {}
        }
    }
}

fn mounted(focus: &'static str) -> Vec<(String, String)> {
    CASE.with(|c| c.set(focus));
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    let mut out = Vec::new();
    elements(&dom, dom.base_scope().root_node(), &mut out);
    out
}

/// The tags of the text field(s) the view gave the focused node (the view has
/// no other `input` or `textarea` for a value).
fn value_fields(focus: &'static str) -> Vec<String> {
    mounted(focus)
        .into_iter()
        .map(|(tag, _)| tag)
        .filter(|tag| tag == "input" || tag == "textarea")
        .collect()
}

#[test]
fn a_string_value_is_a_text_area() {
    assert_eq!(value_fields("text"), ["textarea"]);
    // Inside the wrapper that makes it grow with its content.
    assert!(
        mounted("text")
            .iter()
            .any(|(tag, class)| tag == "div" && class == "structured-value-grow")
    );
}

#[test]
fn a_number_keeps_its_one_line_field() {
    assert_eq!(value_fields("count"), ["input"]);
    assert!(
        !mounted("count")
            .iter()
            .any(|(_, class)| class == "structured-value-grow")
    );
}

#[test]
fn an_on_off_value_and_a_null_have_no_text_field() {
    assert!(value_fields("on").is_empty());
    assert!(value_fields("none").is_empty());
}

// ── the heading of a list member, in the user's language (RFC-073 §8.6) ─────

#[test]
fn the_writing_areas_heading_for_a_list_member_is_in_the_users_language() {
    use omriss_ui::{EditorSession, FileTextProfile};
    thread_local! {
        static LANG: std::cell::Cell<Locale> = const { std::cell::Cell::new(Locale::En) };
    }
    fn host() -> Element {
        let locale = LANG.with(|l| l.get());
        let mut session = use_signal(|| {
            EditorSession::open_detected(
                "[true]\n".into(),
                None,
                FileTextProfile::detect("[true]\n", false),
                DocumentFormat::Json,
            )
            .unwrap()
        });
        let member = session.read().outline_items()[0].id;
        use_hook(|| session.write().focus(member).unwrap());
        rsx! {
            StructuredFocusView {
                session,
                locale: use_signal(move || locale),
                draft: use_signal(String::new),
                status: use_signal(String::new),
            }
        }
    }
    let texts = |locale| {
        LANG.with(|l| l.set(locale));
        crate::test_texts::texts_of(host)
    };
    assert!(texts(Locale::En).iter().any(|t| t == "Item 1"));
    let ja = texts(Locale::Ja);
    assert!(ja.iter().any(|t| t == "項目 1"), "{ja:?}");
    assert!(!ja.iter().any(|t| t == "Item 1"));
}
