//! RFC-066: property-based tests over the document-generator's output.
//!
//! Run before RFC-065's fixes, deliberately: a property that has only ever
//! been green is indistinguishable from one that is wrong. See
//! `rfcs/handoffs/066-property-based-and-fuzz-testing/implementation-handoff.md`.

mod generator;
mod p1_replace_undo;
mod p2_structural_invariants;
mod p3_body_addressing;
