//! RFC-069 §2.1: committing a draft must resync the draft signal, or a stale
//! draft re-commits on every navigation and pollutes the undo history.
//!
//! These run inside a bare Dioxus `VirtualDom` — a runtime to own the signals,
//! with no renderer and no WebView — because `draft_sync` is written against
//! `Signal`s and the defect lives in how the signals and the session interact.

use std::cell::RefCell;

use dioxus::prelude::*;
use omriss_core::{DocumentFormat, NodeId};
use omriss_ui::{DocumentMapNode, EditorSession, FileTextProfile, node_id_from_raw};

use super::{commit_and_resync, is_pending, sync};

type Panic = Box<dyn std::any::Any + Send>;

thread_local! {
    static BODY: RefCell<Option<fn()>> = const { RefCell::new(None) };
    static FAILURE: RefCell<Option<Panic>> = const { RefCell::new(None) };
}

/// Runs `body` once, inside a Dioxus runtime, and re-raises its panic.
///
/// Dioxus catches a panic in a component, so without this an assertion failing
/// inside `body` would be swallowed and the test would pass. It is caught here,
/// stored, and resumed after the runtime has returned.
fn in_runtime(body: fn()) {
    fn root() -> Element {
        use_hook(|| {
            let body = BODY.with(|b| b.borrow().expect("body set"));
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
            FAILURE.with(|f| *f.borrow_mut() = outcome.err());
        });
        rsx! {}
    }
    BODY.with(|b| *b.borrow_mut() = Some(body));
    FAILURE.with(|f| *f.borrow_mut() = None);
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    if let Some(panic) = FAILURE.with(|f| f.borrow_mut().take()) {
        std::panic::resume_unwind(panic);
    }
}

fn find_in(node: &DocumentMapNode, title: &str) -> Option<NodeId> {
    if node.title == title {
        return Some(node_id_from_raw(node.id));
    }
    node.children.iter().find_map(|c| find_in(c, title))
}

const DOC: &str = "# One\nbody\n\n# Two\nmore\n";

/// A session focused on `One`, with its draft synced and then edited to `typed`
/// (no trailing newline — which is what makes the commit differ from the draft).
fn focused_on_one(typed: &str) -> (Signal<EditorSession>, Signal<String>, Signal<String>) {
    let mut session = Signal::new(EditorSession::open(DOC.to_string(), None).unwrap());
    let id = find_in(&session.read().document_map_nodes(), "One").unwrap();
    session.write().focus(id).unwrap();
    let mut draft = Signal::new(String::new());
    sync(&session, &mut draft);
    draft.set(typed.to_string());
    (session, draft, Signal::new(String::new()))
}

/// The same, with the draft left exactly as the section holds it.
fn focused_clean() -> (Signal<EditorSession>, Signal<String>, Signal<String>) {
    let (session, mut draft, status) = focused_on_one("");
    sync(&session, &mut draft);
    (session, draft, status)
}

#[test]
fn after_a_commit_the_draft_is_no_longer_pending() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_on_one("typed");
        assert!(
            is_pending(&session, &draft),
            "precondition: an edit is pending"
        );

        assert!(commit_and_resync(&mut session, &mut draft, &mut status));

        assert!(
            !is_pending(&session, &draft),
            "the dirty dot must clear: the draft is {:?}, the section is {:?}",
            draft.read(),
            session.read().current_snapshot().map(|s| s.body)
        );
        assert!(
            session.read().source().contains("typed\n"),
            "the session stored the body with its appended newline"
        );
    });
}

#[test]
fn navigating_after_a_commit_pushes_no_new_undo_record() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_on_one("typed");
        let original = session.read().source().to_string();

        commit_and_resync(&mut session, &mut draft, &mut status);
        // Three more navigations, each of which commits any pending draft first.
        for _ in 0..3 {
            assert!(commit_and_resync(&mut session, &mut draft, &mut status));
        }

        session.write().undo().unwrap();
        assert_eq!(
            session.read().source(),
            original,
            "one Ctrl+Z must undo the one edit, not a no-op re-commit of it"
        );
        assert!(
            !session.read().can_undo(),
            "exactly one undo record existed"
        );
    });
}

#[test]
fn a_save_straight_after_a_commit_leaves_nothing_to_warn_about() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_on_one("typed");
        assert!(commit_and_resync(&mut session, &mut draft, &mut status));
        // `handle_save` commits, writes, then marks the session saved.
        session.write().mark_saved(None);

        // `handle_open_guarded` warns when `is_dirty() || has_pending_draft()`.
        assert!(!session.read().is_dirty());
        assert!(
            !is_pending(&session, &draft),
            "Open right after a save must not warn about unsaved changes"
        );
    });
}

#[test]
fn with_nothing_pending_it_changes_nothing() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_on_one("");
        sync(&session, &mut draft); // discard the edit: draft == committed body
        assert!(!is_pending(&session, &draft));
        let before = session.read().source().to_string();
        assert!(commit_and_resync(&mut session, &mut draft, &mut status));
        assert_eq!(session.read().source(), before);
        assert!(!session.read().can_undo(), "no history entry for a no-op");
    });
}

#[test]
fn a_refused_commit_blocks_and_leaves_the_users_draft_alone() {
    in_runtime(|| {
        let mut session = Signal::new(
            EditorSession::open_detected(
                "{\"scale\": 1}\n".to_string(),
                None,
                FileTextProfile::detect("", false),
                DocumentFormat::Json,
            )
            .unwrap(),
        );
        let id = find_in(&session.read().document_map_nodes(), "scale").unwrap();
        session.write().focus(id).unwrap();
        let mut draft = Signal::new(String::new());
        sync(&session, &mut draft);
        draft.set("not a number".to_string());
        let mut status = Signal::new(String::new());

        assert!(!commit_and_resync(&mut session, &mut draft, &mut status));

        assert_eq!(
            *draft.read(),
            "not a number",
            "an invalid draft must survive the refusal, not be overwritten by a resync"
        );
        assert!(!status.read().is_empty(), "the refusal is reported");
    });
}

#[test]
#[should_panic(expected = "harness sanity")]
fn the_harness_propagates_a_panic_from_inside_the_runtime() {
    in_runtime(|| panic!("harness sanity"));
}

// ── RFC-076 §3.3: two drafts are never live at once ─────────────────────────

/// Enters the raw view and its edit mode the way the app does: commit what is
/// pending (and resync), open the view, choose Edit.
fn enter_raw_edit(
    session: &mut Signal<EditorSession>,
    draft: &mut Signal<String>,
    status: &mut Signal<String>,
) {
    assert!(
        commit_and_resync(session, draft, status),
        "entering commits first"
    );
    session.write().show_raw();
    let text = session.write().begin_raw_edit();
    draft.set(text);
}

#[test]
fn entering_the_raw_view_commits_the_section_draft_instead_of_carrying_it() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_on_one("typed");
        enter_raw_edit(&mut session, &mut draft, &mut status);

        assert!(
            session.read().source().contains("typed\n"),
            "the section draft was committed, not dropped"
        );
        assert_eq!(
            *draft.read(),
            session.read().raw_text(),
            "the one draft is the raw text"
        );
        assert!(!is_pending(&session, &draft), "and it starts clean");
    });
}

#[test]
fn a_raw_edit_is_pending_until_it_is_committed_and_then_focus_is_the_outline() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_clean();
        enter_raw_edit(&mut session, &mut draft, &mut status);
        let original = session.read().source().to_string();

        let edited = draft.read().replace("more", "rewritten");
        draft.set(edited);
        assert!(is_pending(&session, &draft));

        assert!(commit_and_resync(&mut session, &mut draft, &mut status));

        assert!(!is_pending(&session, &draft));
        assert_eq!(
            *draft.read(),
            session.read().raw_text(),
            "resynced to the document"
        );
        assert!(session.read().source().contains("rewritten"));
        session.write().leave_raw();
        assert!(
            session.read().current_snapshot().is_none(),
            "the outline, not a guess"
        );

        session.write().undo().unwrap();
        assert_eq!(session.read().source(), original, "one Ctrl+Z returns it");
    });
}

#[test]
fn leaving_the_raw_view_with_an_uncommitted_raw_draft_commits_it() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_clean();
        enter_raw_edit(&mut session, &mut draft, &mut status);
        let edited = draft.read().replace("body", "BODY");
        draft.set(edited);

        // `leave_raw_guarded` in the app: commit_and_resync, then leave, then sync.
        assert!(commit_and_resync(&mut session, &mut draft, &mut status));
        session.write().leave_raw();
        sync(&session, &mut draft);

        assert!(
            session.read().source().contains("BODY"),
            "nothing was discarded"
        );
        assert!(!session.read().raw_editing());
        assert!(
            !is_pending(&session, &draft),
            "no stale raw text left in the draft"
        );
    });
}

#[test]
fn a_refused_raw_commit_blocks_leaving_and_keeps_the_draft() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_clean();
        enter_raw_edit(&mut session, &mut draft, &mut status);
        let typed = draft.read().replace("body", "MINE");
        draft.set(typed.clone());
        // The document moves on under the draft.
        let other = session.read().raw_text().replace("more", "theirs");
        let rev = session.read().revision();
        session.write().commit_raw_text(&other, rev).unwrap();

        assert!(!commit_and_resync(&mut session, &mut draft, &mut status));
        assert_eq!(*draft.read(), typed, "the user's text is not overwritten");
        assert!(
            session.read().raw_editing(),
            "still editing: the leave was blocked"
        );
        assert_eq!(*status.read(), "error.stale_edit");
    });
}

#[test]
fn read_only_raw_view_has_no_pending_draft() {
    in_runtime(|| {
        let (mut session, mut draft, mut status) = focused_on_one("typed");
        assert!(commit_and_resync(&mut session, &mut draft, &mut status));
        session.write().show_raw(); // read-only: Edit not chosen
        assert!(!session.read().raw_editing());
        assert!(!is_pending(&session, &draft));
    });
}
