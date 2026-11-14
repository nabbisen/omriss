//! RFC-069 §2.5, level 2: a bare Dioxus `VirtualDom` owns the signals and runs
//! the effect, with no renderer and no WebView.
//!
//! Each of the five Document Map actions posts its request; the consumer must
//! hand the host exactly the dialog that action meant, reset the request to
//! `None`, and leave `status` as a plain message ("status.ready").

use std::cell::RefCell;

use dioxus::dioxus_core::NoOpMutations;
use omriss_core::NodeId;
use omriss_ui::{DocumentMapNode, EditorSession, node_id_from_raw};

use super::*;

const DOC: &str = "# One\nbody\n\n## Child\nc\n\n# Two\nmore\n";

type Handles = (
    Signal<EditorSession>,
    Signal<Option<StructureRequest>>,
    Signal<String>,
);

thread_local! {
    static FOCUS: RefCell<Option<&'static str>> = const { RefCell::new(None) };
    static HANDLES: RefCell<Option<Handles>> = const { RefCell::new(None) };
    static DIALOGS: RefCell<Vec<StructureDialog>> = const { RefCell::new(Vec::new()) };
}

fn find_in(node: &DocumentMapNode, title: &str) -> Option<NodeId> {
    if node.title == title {
        return Some(node_id_from_raw(node.id));
    }
    node.children.iter().find_map(|c| find_in(c, title))
}

fn host() -> Element {
    let session = use_signal(|| {
        let mut s = EditorSession::open(DOC.to_string(), None).unwrap();
        if let Some(title) = FOCUS.with(|f| *f.borrow()) {
            let id = find_in(&s.document_map_nodes(), title).unwrap();
            s.focus(id).unwrap();
        }
        s
    });
    let request = use_signal(|| None);
    // A message that is not "status.ready", so the reset is observable.
    let status = use_signal(|| "status.unsaved".to_string());
    let on_dialog = use_callback(|d: StructureDialog| DIALOGS.with(|v| v.borrow_mut().push(d)));
    use_structure_requests(session, request, status, on_dialog);
    HANDLES.with(|h| *h.borrow_mut() = Some((session, request, status)));
    rsx! {}
}

/// Posts `request` against a session focused on `focus` (or on nothing), lets
/// the effect run, and returns the dialogs the host received, with the final
/// request and status.
fn post(
    request: StructureRequest,
    focus: Option<&'static str>,
) -> (Vec<StructureDialog>, Option<StructureRequest>, String) {
    FOCUS.with(|f| *f.borrow_mut() = focus);
    DIALOGS.with(|v| v.borrow_mut().clear());
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);

    let (_, mut req, _) = HANDLES.with(|h| h.borrow().unwrap());
    dom.in_scope(ScopeId::ROOT, || req.set(Some(request)));
    dom.render_immediate(&mut NoOpMutations);
    // An effect that writes the request back re-runs once more; let it settle.
    dom.render_immediate(&mut NoOpMutations);

    let (_, req, status) = HANDLES.with(|h| h.borrow().unwrap());
    let (req, status) = dom.in_scope(ScopeId::ROOT, || (*req.read(), status.read().clone()));
    (DIALOGS.with(|v| v.borrow().clone()), req, status)
}

fn title_dialog(action: SectionTitleAction, initial: &str) -> StructureDialog {
    StructureDialog::SectionTitle {
        action,
        initial_title: initial.to_string(),
    }
}

#[test]
fn each_request_opens_the_dialog_that_action_meant() {
    let cases = [
        (
            StructureRequest::AddTopLevel,
            title_dialog(SectionTitleAction::AddTopLevel, ""),
        ),
        (
            StructureRequest::AddInside,
            title_dialog(SectionTitleAction::AddInside, ""),
        ),
        (
            StructureRequest::AddAfter,
            title_dialog(SectionTitleAction::AddAfter, ""),
        ),
        (
            StructureRequest::Rename,
            title_dialog(SectionTitleAction::Rename, "One"),
        ),
        (
            StructureRequest::Delete,
            StructureDialog::ConfirmDelete {
                title: "One".into(),
                child_count: 1,
            },
        ),
    ];
    for (request, expected) in cases {
        let (dialogs, left, status) = post(request, Some("One"));
        assert_eq!(dialogs, vec![expected], "{request:?}: exactly one dialog");
        assert_eq!(left, None, "{request:?}: the request is cleared");
        assert_eq!(status, "status.ready", "{request:?}: status is a message");
    }
}

#[test]
fn a_request_is_handled_once_not_on_every_pass() {
    // The effect writes the request it reads; it must not loop or re-fire.
    let (dialogs, left, _) = post(StructureRequest::Rename, Some("Two"));
    assert_eq!(dialogs.len(), 1);
    assert_eq!(left, None);
}

#[test]
fn delete_and_rename_with_nothing_focused_open_no_dialog_but_clear_the_request() {
    let (dialogs, left, status) = post(StructureRequest::Delete, None);
    assert!(dialogs.is_empty(), "nothing to confirm: {dialogs:?}");
    assert_eq!(left, None);
    assert_eq!(status, "status.ready");

    let (dialogs, left, _) = post(StructureRequest::Rename, None);
    assert_eq!(
        dialogs,
        vec![title_dialog(SectionTitleAction::Rename, "")],
        "rename opens with an empty title when nothing is focused (as before)"
    );
    assert_eq!(left, None);
}

#[test]
fn no_request_means_no_dialog_and_status_untouched() {
    FOCUS.with(|f| *f.borrow_mut() = Some("One"));
    DIALOGS.with(|v| v.borrow_mut().clear());
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let (_, _, status) = HANDLES.with(|h| h.borrow().unwrap());
    assert!(DIALOGS.with(|v| v.borrow().is_empty()));
    assert_eq!(
        dom.in_scope(ScopeId::ROOT, || status.read().clone()),
        "status.unsaved"
    );
}
