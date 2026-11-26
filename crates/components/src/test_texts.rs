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
