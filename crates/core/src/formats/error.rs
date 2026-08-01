//! Adapter error model (RFC-053 §11).
//!
//! Adapters return structured, internal error kinds. The presentation layer
//! maps each kind to a friendly message (RFC-053 §11's table); this crate
//! never names that text, matching how `omriss-ui` — not `omriss-core` —
//! owns the `CapabilityReason` catalog-key mapping (RFC-001).

/// Why a format adapter could not build or apply structure (RFC-053 §11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureErrorKind {
    /// The source does not parse as valid input for this format.
    InvalidSyntax,
    /// A construct exists that this format's adapter cannot yet represent.
    UnsupportedFeature,
    /// The requested range or edit could not be applied safely.
    UnsafeRange,
    /// The input is too large to process this way.
    TooLarge,
    /// An internal invariant failed; never caused by user input.
    InternalInvariantFailed,
}

/// A format adapter's structure-building failure (RFC-053 §11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureError {
    pub kind: StructureErrorKind,
}
