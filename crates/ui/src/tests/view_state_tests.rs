use omriss_core::NodeId;

use crate::editor::view_state::{ViewMode, ViewState};

fn node(seed: u64) -> NodeId {
    NodeId(seed)
}

#[test]
fn starts_at_the_outline_with_no_history() {
    let view = ViewState::new();
    assert_eq!(view.mode(), ViewMode::Outline);
    assert!(!view.can_go_back());
    assert!(!view.can_go_forward());
    assert_eq!(view.focused(), None);
}

#[test]
fn focusing_pushes_history_and_back_returns() {
    let mut view = ViewState::new();
    view.focus(node(1));
    view.focus(node(2));
    assert_eq!(view.focused(), Some(node(2)));

    assert_eq!(view.back(), Some(ViewMode::Focus(node(1))));
    assert_eq!(view.back(), Some(ViewMode::Outline));
    assert_eq!(view.back(), None, "history is exhausted");
}

#[test]
fn forward_retraces_after_back() {
    let mut view = ViewState::new();
    view.focus(node(1));
    view.focus(node(2));
    view.back();
    assert!(view.can_go_forward());
    assert_eq!(view.forward(), Some(ViewMode::Focus(node(2))));
    assert_eq!(view.forward(), None);
}

#[test]
fn new_focus_after_back_clears_forward_branch() {
    let mut view = ViewState::new();
    view.focus(node(1));
    view.focus(node(2));
    view.back();
    view.focus(node(3));
    assert!(!view.can_go_forward(), "branching discards forward history");
    assert_eq!(view.back(), Some(ViewMode::Focus(node(1))));
}

#[test]
fn refocusing_the_current_node_does_not_grow_history() {
    let mut view = ViewState::new();
    view.focus(node(1));
    view.focus(node(1));
    assert_eq!(view.back(), Some(ViewMode::Outline));
    assert_eq!(view.back(), None);
}

#[test]
fn show_outline_is_a_history_entry_too() {
    let mut view = ViewState::new();
    view.focus(node(1));
    view.show_outline();
    assert_eq!(view.mode(), ViewMode::Outline);
    assert_eq!(view.back(), Some(ViewMode::Focus(node(1))));
}

#[test]
fn retain_alive_prunes_dead_nodes_everywhere() {
    let mut view = ViewState::new();
    view.focus(node(1));
    view.focus(node(2));
    view.focus(node(3));
    view.back(); // forward holds node 3, back holds [outline, 1], current 2

    view.retain_alive(|id| id != node(2) && id != node(3));

    assert_eq!(view.mode(), ViewMode::Outline, "dead focus falls back");
    assert!(!view.can_go_forward(), "dead forward entries pruned");
    assert_eq!(view.back(), Some(ViewMode::Focus(node(1))));
    assert_eq!(view.back(), Some(ViewMode::Outline));
}
