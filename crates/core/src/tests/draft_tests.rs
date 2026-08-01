//! RFC-053 §9.3: focused-draft validity state.

use crate::DraftState;

#[test]
fn clean_does_not_block_navigation() {
    assert!(!DraftState::Clean.blocks_navigation());
}

#[test]
fn valid_uncommitted_does_not_block_navigation() {
    assert!(!DraftState::ValidUncommitted.blocks_navigation());
}

#[test]
fn invalid_uncommitted_blocks_navigation() {
    assert!(DraftState::InvalidUncommitted.blocks_navigation());
}

#[test]
fn default_is_clean() {
    assert_eq!(DraftState::default(), DraftState::Clean);
}
