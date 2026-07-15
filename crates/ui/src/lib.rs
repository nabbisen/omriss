//! # omriss-ui
//!
//! Renderer-independent GUI logic for Omriss: editor sessions,
//! focus navigation state, file text profile, search, command registry,
//! document statistics, and internationalized UI strings. Everything here
//! is plain Rust with no windowing or WebView dependency, so it builds and
//! tests on any host; the desktop shell in the `omriss` app package wires these
//! types to Dioxus.
//!
//! ```
//! use omriss_ui::{i18n::{t, Locale}, EditorSession};
//!
//! let mut session = EditorSession::open("# Idea\n\nDraft.\n".to_string(), None)?;
//! let top = session.outline_items();
//! let snapshot = session.focus(top[0].id)?;
//! assert_eq!(snapshot.title, "Idea");
//! assert_eq!(t(Locale::Ja, "toolbar.undo"), "元に戻す");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// ── Domain modules ───────────────────────────────────────────────────────────
//
// editor/    : in-editor state — navigation, search, view mode, statistics
// file/      : file-level concerns — encoding and line-ending profile
// interface/ : UI model types — command registry, Document Map view model
// i18n/      : internationalization catalogs and locale lookup
// session    : EditorSession facade (wires everything together)

pub mod editor;
pub mod file;
pub mod i18n;
pub mod interface;
mod session;

#[cfg(test)]
mod tests;

// ── Public API ──────────────────────────────────────────────────────────────
pub use editor::navigation::SiblingInfo;
pub use editor::search::SearchMatch;
pub use editor::stats::DocumentStats;
pub use editor::view_state::{ViewMode, ViewState};
pub use file::file_profile::{FileTextProfile, NewlinePolicy};
pub use interface::commands::{COMMANDS, CommandSpec, filter_commands};
pub use interface::document_map::{
    CapabilityReason, DocumentMapNode, DraftState, MapCapability, MapNodeCapabilities,
    node_id_from_raw,
};
pub use session::{EditorSession, OutlineNode};
// Structural editing types re-exported for the desktop crate.
pub use omriss_core::{MoveTarget, StructuralEditError};
