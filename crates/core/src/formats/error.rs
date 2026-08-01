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

/// A format adapter's structure-building failure (RFC-053 §11,
/// `DocumentFormatAdapter::build_structure`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureError {
    pub kind: StructureErrorKind,
}

/// A format adapter's focused-content failure (RFC-053 §11,
/// `DocumentFormatAdapter::focused_content`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusError {
    pub kind: StructureErrorKind,
}

/// A format adapter's edit-validation failure (RFC-053 §11,
/// `DocumentFormatAdapter::validate_focused_edit`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditValidationError {
    pub kind: StructureErrorKind,
}

/// A format adapter's apply-edit failure (RFC-053 §11,
/// `DocumentFormatAdapter::apply_validated_edit`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyEditError {
    pub kind: StructureErrorKind,
}

/// A format adapter's structure-command failure (RFC-053 §11,
/// `DocumentFormatAdapter::structure_command`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureCommandError {
    pub kind: StructureErrorKind,
}

// Every adapter error struct above has the same one-field shape, so a
// `StructureErrorKind` converts into whichever one a call site needs.
impl From<StructureErrorKind> for StructureError {
    fn from(kind: StructureErrorKind) -> Self {
        Self { kind }
    }
}
impl From<StructureErrorKind> for FocusError {
    fn from(kind: StructureErrorKind) -> Self {
        Self { kind }
    }
}
impl From<StructureErrorKind> for EditValidationError {
    fn from(kind: StructureErrorKind) -> Self {
        Self { kind }
    }
}
impl From<StructureErrorKind> for ApplyEditError {
    fn from(kind: StructureErrorKind) -> Self {
        Self { kind }
    }
}
impl From<StructureErrorKind> for StructureCommandError {
    fn from(kind: StructureErrorKind) -> Self {
        Self { kind }
    }
}
