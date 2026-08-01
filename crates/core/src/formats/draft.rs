//! Focused-draft validity state (RFC-053 §9.3).
//!
//! Editor-local; not a second user-visible saved-document dirty state. The
//! user-facing file status remains Saved / Unsaved changes; `DraftState`
//! drives the block-on-invalid gate.
//!
//! RFC-053 §14 assigns this type to `omriss-core` alongside the capability
//! vocabulary (§6), reconciling it out of `omriss-ui`, where it shipped
//! early in M10. Ported verbatim; meaning unchanged.

/// Tracks the validity of an in-progress focused-content edit (RFC-053 §9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DraftState {
    /// No local edit in progress; matches the committed content.
    #[default]
    Clean,
    /// The draft differs from committed content and is format-valid.
    ValidUncommitted,
    /// The draft differs from committed content and fails format validation.
    /// Navigation, save, and preview are blocked until resolved.
    InvalidUncommitted,
}

impl DraftState {
    /// Returns `true` when the draft blocks navigation/save/preview.
    pub fn blocks_navigation(self) -> bool {
        self == Self::InvalidUncommitted
    }
}
