//! Desktop GUI components (RFC-010..013, RFC-017, RFC-019..026, RFC-027..030, RFC-039..041, RFC-048..050).

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
pub(crate) use command_palette::focus_active_palette;
pub use command_palette::{CommandId, CommandPalette};
pub use confirm_delete_dialog::{ConfirmDeleteChoice, ConfirmDeleteDialog};
pub use document_map_pane::DocumentMapPane;
pub use error_dialog::ErrorDialog;
pub use ext_modified_dialog::{ExtModifiedChoice, ExtModifiedDialog};
pub use focused_content_pane::FocusedContentPane;
pub(crate) use modal_focus::{focus_active_modal, trap_modal_tab};
pub use overview_pane::OverviewPane;
pub use preview_pane::PreviewPane;
pub use raw_source::RawSourceView;
pub use search_panel::SearchPanel;
pub use split_dialog::{SectionTitleAction, SectionTitleChoice, SectionTitleDialog};
pub use status_bar::StatusBar;
pub use toolbar::Toolbar;
pub use unsaved_dialog::{UnsavedChoice, UnsavedDialog};
pub use welcome::WelcomeScreen;
