//! Desktop entry point: detects the startup locale, loads settings, configures
//! the window, and launches the root component (RFC-001, RFC-010, RFC-035,
//! RFC-036).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod components;
mod file;
mod input;
mod shell;
mod storage;

use dioxus::desktop::{Config, WindowBuilder};
use omriss_ui::i18n::Locale;

use crate::storage::settings::AppSettings;

/// Detects the OS locale at startup from environment variables.
/// Explicit user preference from saved settings will take precedence.
fn detect_locale() -> Locale {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|var| std::env::var(var).ok())
        .find_map(|tag| Locale::from_tag(&tag))
        .unwrap_or_default()
}

fn main() {
    // RFC-036: load settings before launching; fall back to defaults silently.
    let settings = AppSettings::load();
    // RFC-063: the first argument (excluding the program name) is the file
    // to open at startup, if any.
    let startup_arg = cli::interpret(std::env::args().skip(1));

    let window = WindowBuilder::new()
        .with_title("Omriss")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1080.0, 720.0));
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window).with_menu(None))
        .with_context(detect_locale())
        .with_context(settings)
        .with_context(startup_arg)
        .launch(shell::app::App);
}
