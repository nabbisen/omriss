//! Level-2 test support: the text a mounted component shows, in document order.
//!
//! A bare Dioxus `VirtualDom`, no renderer: the template tree is what the WebView
//! is given. Static text and dynamic text nodes both count; elements' attributes
//! do not (a test that needs one reads it from the template itself).

use dioxus::dioxus_core::{DynamicNode, TemplateNode, VNode};
use dioxus::prelude::*;

fn walk_template(t: &TemplateNode, out: &mut Vec<String>) {
    match t {
        TemplateNode::Text { text } => out.push(text.to_string()),
        TemplateNode::Element { children, .. } => {
            for child in children.iter() {
                walk_template(child, out);
            }
        }
        TemplateNode::Dynamic { .. } => {}
    }
}

/// Every text in `node`, descending through components and fragments. A dynamic
/// node is placed after its static siblings, which is enough for membership and
/// relative-order checks inside one element.
fn collect(dom: &VirtualDom, node: &VNode, out: &mut Vec<String>) {
    for root in node.template.roots.iter() {
        walk_template(root, out);
    }
    for (i, d) in node.dynamic_nodes.iter().enumerate() {
        match d {
            DynamicNode::Text(t) => out.push(t.value.clone()),
            DynamicNode::Component(c) => {
                if let Some(scope) = c.mounted_scope(i, node, dom) {
                    collect(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Fragment(children) => {
                for child in children {
                    collect(dom, child, out);
                }
            }
            DynamicNode::Placeholder(_) => {}
        }
    }
}

/// Mounts `app` and returns every text it shows.
pub(crate) fn texts_of(app: fn() -> Element) -> Vec<String> {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let mut out = Vec::new();
    collect(&dom, dom.base_scope().root_node(), &mut out);
    out
}

/// [`texts_of`] after the effects have run: mounted, rendered again, mounted
/// text read. Components that fill their state in `use_effect` need this.
pub(crate) fn texts_of_settled(app: fn() -> Element) -> Vec<String> {
    use dioxus::dioxus_core::NoOpMutations;
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    // Effects queued by the first render run, and what they set renders.
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    let mut out = Vec::new();
    collect(&dom, dom.base_scope().root_node(), &mut out);
    out
}

/// An element a mounted component renders: its tag and its attributes, static and
/// dynamic, as text. What a test that asks "does this landmark have a name" reads.
#[derive(Debug, Clone)]
pub(crate) struct Rendered {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
}

impl Rendered {
    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

fn attribute_text(value: &dioxus::dioxus_core::AttributeValue) -> Option<String> {
    use dioxus::dioxus_core::AttributeValue as V;
    match value {
        V::Text(s) => Some(s.clone()),
        V::Float(f) => Some(f.to_string()),
        V::Int(i) => Some(i.to_string()),
        V::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn walk_elements(dom: &VirtualDom, node: &VNode, t: &TemplateNode, out: &mut Vec<Rendered>) {
    use dioxus::dioxus_core::TemplateAttribute;
    match t {
        TemplateNode::Text { .. } => {}
        TemplateNode::Element {
            tag,
            attrs,
            children,
            ..
        } => {
            let mut found = Vec::new();
            for a in attrs.iter() {
                match a {
                    TemplateAttribute::Static { name, value, .. } => {
                        found.push((name.to_string(), value.to_string()));
                    }
                    TemplateAttribute::Dynamic { id } => {
                        for attr in node.dynamic_attrs[*id].iter() {
                            if let Some(v) = attribute_text(&attr.value) {
                                found.push((attr.name.to_string(), v));
                            }
                        }
                    }
                }
            }
            out.push(Rendered {
                tag: tag.to_string(),
                attrs: found,
            });
            for child in children.iter() {
                walk_elements(dom, node, child, out);
            }
        }
        TemplateNode::Dynamic { id } => match &node.dynamic_nodes[*id] {
            DynamicNode::Component(c) => {
                if let Some(scope) = c.mounted_scope(*id, node, dom) {
                    collect_elements(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Fragment(children) => {
                for child in children {
                    collect_elements(dom, child, out);
                }
            }
            DynamicNode::Text(_) | DynamicNode::Placeholder(_) => {}
        },
    }
}

fn collect_elements(dom: &VirtualDom, node: &VNode, out: &mut Vec<Rendered>) {
    for root in node.template.roots.iter() {
        walk_elements(dom, node, root, out);
    }
}

/// Mounts `app` after its effects have run and returns every element it renders,
/// in document order, with its attributes.
pub(crate) fn elements_of_settled(app: fn() -> Element) -> Vec<Rendered> {
    use dioxus::dioxus_core::NoOpMutations;
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    let mut out = Vec::new();
    collect_elements(&dom, dom.base_scope().root_node(), &mut out);
    out
}
