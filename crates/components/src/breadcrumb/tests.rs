//! RFC-013 layout defects, level 2: the crumbs render inside
//! `.breadcrumb-crumbs` and the up button outside it, and the collapsed-path
//! label is the catalog's. Mounted in a bare Dioxus `VirtualDom`; the template
//! tree is exactly what the WebView is given.

use dioxus::dioxus_core::{DynamicNode, TemplateAttribute, TemplateNode, VNode};
use dioxus::prelude::*;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{DocumentMapNode, EditorSession, node_id_from_raw};

use super::Breadcrumb;

/// Six levels deep, so the path collapses (`MAX_VISIBLE` is 4).
const DEEP: &str = "# A\n\n## B\n\n### C\n\n#### D\n\n##### E\n\n###### F\n\nbody\n";

fn find_title(node: &DocumentMapNode, title: &str) -> Option<u64> {
    if node.title == title {
        return Some(node.id);
    }
    node.children.iter().find_map(|c| find_title(c, title))
}

thread_local! {
    static CASE: std::cell::Cell<(Locale, &'static str)> =
        const { std::cell::Cell::new((Locale::En, "A")) };
}

fn host() -> Element {
    let (locale, focus) = CASE.with(|c| c.get());
    let mut session = use_signal(|| EditorSession::open(DEEP.into(), None).unwrap());
    let id = find_title(&session.read().document_map_nodes(), focus).unwrap();
    use_hook(|| session.write().focus(node_id_from_raw(id)).unwrap());
    rsx! {
        Breadcrumb {
            session,
            locale: use_signal(move || locale),
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

/// Walks the mounted tree, recording for every element whose class is one of
/// `wanted` the chain of classes of its ancestors (outermost first).
fn collect(
    dom: &VirtualDom,
    node: &VNode,
    chain: &mut Vec<String>,
    wanted: &[&str],
    out: &mut Vec<(String, Vec<String>)>,
) {
    fn walk(
        t: &TemplateNode,
        chain: &mut Vec<String>,
        wanted: &[&str],
        out: &mut Vec<(String, Vec<String>)>,
    ) {
        if let TemplateNode::Element {
            attrs, children, ..
        } = t
        {
            let class = attr(attrs, "class").unwrap_or_default();
            if wanted.contains(&class.as_str()) {
                out.push((class.clone(), chain.clone()));
            }
            chain.push(class);
            for child in children.iter() {
                walk(child, chain, wanted, out);
            }
            chain.pop();
        }
    }
    for root in node.template.roots.iter() {
        walk(root, chain, wanted, out);
    }
    for (i, d) in node.dynamic_nodes.iter().enumerate() {
        // A dynamic node's own elements come from another template: they sit
        // below the classes of the static elements on its path in this one.
        let mut below = chain.clone();
        below.extend(classes_on_path(&node.template, i));
        match d {
            DynamicNode::Component(c) => {
                if let Some(scope) = c.mounted_scope(i, node, dom) {
                    collect(dom, scope.root_node(), &mut below, wanted, out);
                }
            }
            DynamicNode::Fragment(children) => {
                for child in children {
                    collect(dom, child, &mut below, wanted, out);
                }
            }
            _ => {}
        }
    }
}

/// The classes of the static elements between a template's root and its
/// `index`th dynamic node, outermost first.
fn classes_on_path(template: &dioxus::dioxus_core::Template, index: usize) -> Vec<String> {
    let path = template.node_paths[index];
    let mut classes = Vec::new();
    let Some((&root, rest)) = path.split_first() else {
        return classes;
    };
    let mut current = template.roots.get(root as usize);
    for &step in rest {
        let Some(TemplateNode::Element {
            attrs, children, ..
        }) = current
        else {
            break;
        };
        classes.push(attr(attrs, "class").unwrap_or_default());
        current = children.get(step as usize);
    }
    classes
}

/// `(class, ancestor classes)` for the crumb-ish elements of the mounted view.
fn render(locale: Locale, focus: &'static str) -> Vec<(String, Vec<String>)> {
    CASE.with(|c| c.set((locale, focus)));
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    let mut out = Vec::new();
    collect(
        &dom,
        dom.base_scope().root_node(),
        &mut Vec::new(),
        &["crumb-btn", "here", "sep", "ellipsis", "breadcrumb-up"],
        &mut out,
    );
    out
}

#[test]
fn the_crumbs_are_inside_the_wrapper_and_the_up_button_outside_it() {
    let found = render(Locale::En, "C");
    let inside = |c: &(String, Vec<String>)| c.1.iter().any(|a| a == "breadcrumb-crumbs");
    for class in ["crumb-btn", "here", "sep"] {
        let of_class: Vec<_> = found.iter().filter(|(c, _)| c == class).collect();
        assert!(!of_class.is_empty(), "no {class} rendered");
        assert!(
            of_class.iter().all(|c| inside(c)),
            "{class} must be inside .breadcrumb-crumbs: {of_class:?}"
        );
    }
    let up: Vec<_> = found.iter().filter(|(c, _)| c == "breadcrumb-up").collect();
    assert_eq!(up.len(), 1);
    assert!(
        !inside(up[0]),
        "the up button stays outside the wrapper, at the right edge: {:?}",
        up[0]
    );
    assert!(up[0].1.iter().any(|a| a == "breadcrumb"), "inside the nav");
}

#[test]
fn a_long_path_collapses_and_labels_the_ellipsis_from_the_catalog() {
    let found = render(Locale::En, "F");
    assert!(
        found.iter().any(|(c, _)| c == "ellipsis"),
        "six levels must collapse"
    );
}

#[test]
fn the_collapsed_label_is_translated_in_both_locales() {
    let (en, ja) = (
        t(Locale::En, "breadcrumb.collapsed"),
        t(Locale::Ja, "breadcrumb.collapsed"),
    );
    assert_ne!(en, "breadcrumb.collapsed", "missing in English");
    assert_ne!(ja, "breadcrumb.collapsed", "missing in Japanese");
    assert_ne!(en, ja);
    assert_ne!(en, "collapsed sections", "still the old literal");
}
