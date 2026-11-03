//! Dioxus components for Omriss, built on `omriss-ui`'s renderer-independent
//! view state (RFC-075).
//!
//! This crate holds everything that draws, and nothing that touches the
//! platform: no file system, no dialogs, no settings, no windowing. Those
//! belong to the host application (`omriss`). That split is what lets a second
//! host be written without untangling this crate.

pub mod draft_sync;

mod breadcrumb;
mod command_palette;
mod confirm_delete_dialog;
mod document_map_pane;
mod error_dialog;
mod ext_modified_dialog;
mod focused_content_pane;
mod modal_focus;
mod overview_pane;
mod preview_pane;
mod raw_source;
mod search_panel;
mod split_dialog;
mod status_bar;
mod toolbar;
mod unsaved_dialog;
mod welcome;

pub use breadcrumb::Breadcrumb;
pub use command_palette::focus_active_palette;
pub use command_palette::{CommandId, CommandPalette};
pub use confirm_delete_dialog::{ConfirmDeleteChoice, ConfirmDeleteDialog};
pub use document_map_pane::DocumentMapPane;
pub use error_dialog::ErrorDialog;
pub use ext_modified_dialog::{ExtModifiedChoice, ExtModifiedDialog};
pub use focused_content_pane::FocusedContentPane;
pub use modal_focus::{focus_active_modal, trap_modal_tab};
pub use overview_pane::OverviewPane;
pub use preview_pane::PreviewPane;
pub use raw_source::RawSourceView;
pub use search_panel::SearchPanel;
pub use split_dialog::{SectionTitleAction, SectionTitleChoice, SectionTitleDialog};
pub use status_bar::StatusBar;
pub use toolbar::Toolbar;
pub use unsaved_dialog::{UnsavedChoice, UnsavedDialog};
pub use welcome::WelcomeScreen;
