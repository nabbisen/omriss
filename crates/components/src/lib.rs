//! Dioxus components for Omriss, built on `omriss-ui`'s renderer-independent
//! view state (RFC-075).
//!
//! This crate holds everything that draws, and nothing that touches the
//! platform: no file system, no dialogs, no settings, no windowing. Those
//! belong to the host application (`omriss`). That split is what lets a second
//! host be written without untangling this crate.

pub mod draft_sync;
mod focus;

#[cfg(test)]
mod commit_path_tests;
#[cfg(test)]
mod contrast_tests;
#[cfg(test)]
mod draft_set_tests;
#[cfg(test)]
mod eval_path_tests;
#[cfg(test)]
mod focus_path_tests;
#[cfg(test)]
mod landmark_tests;
#[cfg(test)]
mod source_scan;
#[cfg(test)]
mod test_texts;
#[cfg(test)]
mod text_area_sites_tests;
#[cfg(test)]
mod title_path_tests;

mod breadcrumb;
mod command_palette;
mod confirm_delete_dialog;
mod document_map_pane;
mod document_title;
mod error_dialog;
mod ext_modified_dialog;
mod focused_content_pane;
mod overview_pane;
mod preview_pane;
mod raw_source;
mod search_panel;
mod split_dialog;
mod status_bar;
mod structure_request;
mod toolbar;
mod unreadable;
mod unsaved_dialog;
mod welcome;

pub use breadcrumb::Breadcrumb;
pub use command_palette::{CommandId, CommandPalette};
pub use confirm_delete_dialog::{ConfirmDeleteChoice, ConfirmDeleteDialog};
pub use document_map_pane::DocumentMapPane;
pub use document_title::{DocumentTitle, STARTUP_HEAD};
pub use error_dialog::ErrorDialog;
pub use ext_modified_dialog::{ExtModifiedChoice, ExtModifiedDialog};
pub use focus::use_focus_script;
pub use focused_content_pane::FocusedContentPane;
pub use overview_pane::OverviewPane;
pub use preview_pane::PreviewPane;
pub use raw_source::RawSourceView;
pub use search_panel::SearchPanel;
pub use split_dialog::{SectionTitleAction, SectionTitleChoice, SectionTitleDialog};
pub use status_bar::StatusBar;
pub use structure_request::{
    StructureDialog, StructureRequest, dialog_for, use_structure_requests,
};
pub use toolbar::Toolbar;
pub use unreadable::UnreadablePane;
pub use unsaved_dialog::{UnsavedChoice, UnsavedDialog};
pub use welcome::WelcomeScreen;
