//! RFC-086: the Document Map's render does not search per row, the tree is rebuilt
//! only when the structure changed, and a focus change costs the DOM what it
//! should. (That the pane goes quiet at all is `settle_tests`.)

use std::cell::Cell;

use dioxus::dioxus_core::{Mutation, NoOpMutations};
use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::Locale;
use omriss_ui::{EditorSession, FileTextProfile};

use super::DocumentMapPane;
use super::node_tree::{find_node, index_nodes, same_structure};

fn session(source: &str, format: DocumentFormat) -> EditorSession {
    EditorSession::open_detected(
        source.to_string(),
        None,
        FileTextProfile::detect(source, false),
        format,
    )
    .unwrap()
}

const MD: &str = "# One\nbody\n\n## Child\nx\n\n# Two\ny\n";

// ── The index and the structure comparison ─────────────────────────────────

#[test]
fn the_index_holds_every_node_and_agrees_with_the_search_it_replaces() {
    for (src, fmt) in [
        (MD, DocumentFormat::Markdown),
        ("{\"a\":[1,{\"b\":2}],\"c\":3}", DocumentFormat::Json),
    ] {
        let root = session(src, fmt).document_map_nodes();
        let index = index_nodes(&root);
        assert_eq!(index.len(), root.descendant_count() + 1, "{src:?}");
        for (id, node) in &index {
            assert_eq!(node.id, *id);
            assert_eq!(find_node(&root, *id), Some(*node), "{src:?}: {id}");
        }
    }
}

#[test]
fn a_focus_change_is_not_a_change_of_structure() {
    for (src, fmt) in [
        (MD, DocumentFormat::Markdown),
        ("{\"a\":[1,2],\"c\":3}", DocumentFormat::Json),
    ] {
        let mut s = session(src, fmt);
        let before = s.document_map_nodes();
        let ids: Vec<_> = s.outline_items().iter().map(|i| i.id).collect();
        s.focus(ids[0]).unwrap();
        let first = s.document_map_nodes();
        s.focus(*ids.last().unwrap()).unwrap();
        let last = s.document_map_nodes();
        // The trees do differ in the one field a focus change alters ...
        assert_ne!(first, last, "{src:?}");
        // ... and that is not a structure change.
        assert!(same_structure(&before, &first), "{src:?}");
        assert!(same_structure(&first, &last), "{src:?}");
    }
}

#[test]
fn an_edit_a_rename_and_another_document_are_changes_of_structure() {
    let mut s = session(MD, DocumentFormat::Markdown);
    let before = s.document_map_nodes();
    let id = s.outline_items()[0].id;
    s.focus(id).unwrap();
    s.rename_focused("Uno").unwrap();
    assert!(
        !same_structure(&before, &s.document_map_nodes()),
        "a rename"
    );
    let mut s = session(MD, DocumentFormat::Markdown);
    s.focus(s.outline_items()[0].id).unwrap();
    s.add_top_level_section("New").unwrap();
    assert!(
        !same_structure(&before, &s.document_map_nodes()),
        "an added section"
    );
    // A different file with the same shape but other titles, and another format.
    let other = session(
        "# Uno\nbody\n\n## Child\nx\n\n# Two\ny\n",
        DocumentFormat::Markdown,
    );
    assert!(!same_structure(&before, &other.document_map_nodes()));
    let json = session("[1,2]", DocumentFormat::Json);
    assert!(!same_structure(&before, &json.document_map_nodes()));
}

// ── Source guards ─────────────────────────────────────────────────────────

fn pane_source() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/document_map_pane.rs"
    ))
    .unwrap()
}

/// The text of the pane's `use_effect` body.
fn effect_body(src: &str) -> &str {
    let start = src.find("use_effect(move || {").expect("the effect");
    let rest = &src[start..];
    let end = rest.find("\n    });").expect("the effect's end");
    &rest[..end]
}

#[test]
fn the_effect_peeks_every_signal_it_also_writes() {
    let src = pane_source();
    let body = effect_body(&src);
    assert!(
        body.contains("item_tree.write()"),
        "the guard is looking at the right text"
    );
    for signal in ["item_tree", "map_root_sig", "is_raw_sig", "format_sig"] {
        assert!(
            !body.contains(&format!("{signal}.read()")),
            "the effect reads `{signal}` with read(): that subscribes it to its own writes and it \
             never stops (RFC-086 L1). Use peek()."
        );
    }
}

#[test]
fn no_row_searches_from_the_root() {
    // `find_node` is a search from the root. A click handler may use it once; a
    // render must not use it per row (RFC-086 L2).
    let src = pane_source();
    let calls: Vec<&str> = src
        .lines()
        .take_while(|l| l.trim() != "#[cfg(test)]")
        .filter(|l| !l.trim_start().starts_with("//"))
        .filter(|l| l.contains("find_node(") && !l.contains("use node_tree"))
        .collect();
    assert!(
        !calls.is_empty(),
        "the guard no longer sees the click handlers' calls"
    );
    for line in calls {
        assert!(
            line.contains("fresh_root"),
            "a find_node call that is not in a click handler: {line:?}; look the node up in \
             `node_index` instead (RFC-086 L2)"
        );
    }
}

#[test]
fn the_tree_is_rebuilt_only_when_the_structure_changed() {
    let src = pane_source();
    let body = effect_body(&src);
    assert_eq!(body.matches("set_tree(").count(), 1, "one rebuild site");
    let at = body.find("set_tree(").unwrap();
    assert!(
        body[..at].trim_end().ends_with("if structure_changed {")
            || body[..at]
                .rsplit('\n')
                .nth(1)
                .is_some_and(|l| l.contains("if structure_changed")),
        "set_tree must be inside `if structure_changed` (RFC-086 L3)"
    );
}

// ── What a focus change costs the DOM ──────────────────────────────────────

thread_local! {
    static SESSION: Cell<Option<Signal<EditorSession>>> = const { Cell::new(None) };
}

fn host() -> Element {
    let mut source = String::from("[");
    for i in 0..300 {
        if i > 0 {
            source.push(',');
        }
        source.push_str(&format!("{{\"id\":{i}}}"));
    }
    source.push(']');
    let session = use_signal(|| {
        let mut s = session(&source, DocumentFormat::Json);
        s.settle_on_open();
        s
    });
    SESSION.with(|c| c.set(Some(session)));
    rsx! {
        DocumentMapPane {
            session,
            locale: use_signal(|| Locale::En),
            draft: use_signal(String::new),
            status: use_signal(String::new),
            request: use_signal(|| None),
        }
    }
}

#[test]
fn a_focus_change_patches_two_attributes_and_creates_nothing() {
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.render_immediate(&mut NoOpMutations);
    }
    dom.in_scope(ScopeId::APP, || {
        let mut s = SESSION.with(|c| c.get()).unwrap();
        let id = s.read().outline_items()[5].id;
        s.write().focus(id).unwrap();
    });
    let mut edits = Vec::new();
    for _ in 0..8 {
        edits.extend(dom.render_immediate_to_vec().edits);
    }
    let attributes = edits
        .iter()
        .filter(|m| matches!(m, Mutation::SetAttribute { name: "class", .. }))
        .count();
    assert_eq!(
        attributes, 2,
        "the row that lost the highlight and the row that gained it: {edits:?}"
    );
    assert_eq!(edits.len(), 2, "nothing else changes in the DOM: {edits:?}");
}
