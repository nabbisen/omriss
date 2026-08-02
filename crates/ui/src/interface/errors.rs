//! `StructureErrorKind` -> friendly message mapping (RFC-053 §11, RFC-054 J4).
//!
//! Closes RFC-053 acceptance criterion 10. `omriss-core` produces a typed
//! `StructureErrorKind` and never names a catalog key (RFC-001) — the same
//! split already established for `CapabilityReason`
//! (`crate::interface::document_map::CapabilityReasonCatalogKey`).
//! `omriss-ui` owns the mapping to a localized string, at render time.
//!
//! The `match` below has no wildcard arm: adding a `StructureErrorKind`
//! variant without updating it is a compile error, not a silent fallback to
//! the raw key (RFC-054 §7 J4: "make the mapping exhaustive so a later
//! variant without a message fails the build rather than falling back
//! silently").

use omriss_core::StructureErrorKind;

/// Maps a core-owned [`StructureErrorKind`] to its i18n catalog key.
pub trait StructureErrorKindCatalogKey {
    /// Returns the i18n catalog key for this error kind.
    fn catalog_key(self) -> &'static str;
}

impl StructureErrorKindCatalogKey for StructureErrorKind {
    fn catalog_key(self) -> &'static str {
        match self {
            Self::InvalidSyntax => "structure_error.invalid_syntax",
            Self::UnsupportedFeature => "structure_error.unsupported_feature",
            Self::UnsafeRange => "structure_error.unsafe_range",
            Self::TooLarge => "structure_error.too_large",
            Self::InternalInvariantFailed => "structure_error.internal_invariant_failed",
        }
    }
}
