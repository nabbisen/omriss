//! Structural editing: promote/demote, move, split, delete, merge (RFC-023..026).
//!
//! All operations go through `apply_replacement`, so they:
//! - roll back automatically if re-indexing fails (RFC-008 transaction shape);
//! - record an `EditRecord` for byte-exact undo/redo (RFC-044);
//! - never partially mutate the document.
//!
//! Operations on Setext headings (level detected from underline style) are
//! rejected for promote/demote in M5; convert them to ATX first via raw view.

mod delete_split_merge;
mod error;
mod level;
mod move_ops;
mod preflight;
mod rename;

pub use error::StructuralEditError;
pub use move_ops::MoveTarget;

pub(crate) use delete_split_merge::{delete_section, merge_with_prev_sibling, split_section};
pub(crate) use level::{demote_section, promote_section};
pub(crate) use move_ops::move_section;
pub(crate) use rename::rename_section;
