//! Document structure vocabulary (RFC-053 §6).
//!
//! These types are the format-neutral vocabulary a `DocumentFormatAdapter`
//! (introduced by later RFC-053 slices) projects source text into: a
//! [`DocumentStructure`] of [`StructureNode`]s, each carrying its
//! [`NodeCapabilities`]. Nothing here builds a `DocumentStructure` from a
//! real document yet — that projection is `MarkdownAdapter::build_structure`
//! (RFC-053 S4), which will reuse the shipped `Outline`'s `NodeId` values
//! rather than re-deriving them (RFC-053 §13.2).
//!
//! This slice (S2) additionally ports the Markdown capability-computation
//! logic that slice S4's adapter will call, so it can be tested at the core
//! level now. `omriss-ui`'s own copy (`crates/ui/src/session/document_map_bridge.rs`)
//! is untouched; the duplication is temporary and resolved when RFC-053 S3
//! reconciles `omriss-ui` onto these types.
//!
//! `CapabilityReason` here never names an i18n catalog key: `omriss-core`
//! must not depend on `omriss-ui` (RFC-001). `omriss-ui` maps a reason to
//! localized text at render time.

mod capability;
mod markdown_capability;
mod model;

pub use capability::{Capability, CapabilityReason, NodeCapabilities};
pub use markdown_capability::markdown_node_capabilities;
pub use model::{DocumentStructure, StructureNode, StructureNodeKind};
