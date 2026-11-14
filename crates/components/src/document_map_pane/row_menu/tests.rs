//! RFC-059 §12.1, level 2: the Document Map row menu declares itself a
//! containing scope that takes the arrow keys. Mounted in a bare `VirtualDom`;
//! see `focus/tests.rs` for how the template is read.

use dioxus::dioxus_core::{TemplateAttribute, TemplateNode};
use dioxus::prelude::*;
use omriss_ui::EditorSession;
use omriss_ui::i18n::Locale;

use super::NodeRowMenu;
use crate::focus::tests::first_element_root;

#[test]
fn the_row_menu_contains_focus_and_takes_arrows() {
    fn host() -> Element {
        let session = use_signal(|| EditorSession::open("# One\nbody\n".into(), None).unwrap());
        let node = session.read().document_map_nodes().children[0].clone();
        rsx! {
            NodeRowMenu {
                node,
                session,
                locale: use_signal(Locale::default),
                draft: use_signal(String::new),
                status: use_signal(String::new),
                request: use_signal(|| None),
                menu_open_for: use_signal(|| None),
            }
        }
    }
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    let menu = first_element_root(&dom, dom.base_scope().root_node())
        .expect("the menu renders an element");
    let Some(TemplateNode::Element { attrs, .. }) = menu.template.roots.first() else {
        unreachable!()
    };
    let has = |wanted: &str, value: &str| {
        attrs.iter().any(|a| {
            matches!(a, TemplateAttribute::Static { name, value: v, .. }
                if *name == wanted && *v == value)
        })
    };
    assert!(has("data-focus-scope", "contain"));
    assert!(has("data-focus-arrows", "true"));
    assert!(has("role", "menu"));
}
