//! Desktop entry point: detects the startup locale, loads settings, configures
//! the window, and launches the root component (RFC-001, RFC-010, RFC-035,
//! RFC-036).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod file;
mod input;
mod locale;
mod shell;
mod storage;

use dioxus::desktop::{Config, WindowBuilder};
use omriss_ui::PRODUCT_NAME;

use crate::locale::detect_locale;
use crate::storage::settings::AppSettings;

fn main() {
    // RFC-036: load settings before launching; fall back to defaults silently.
    let settings = AppSettings::load();
    // RFC-063: the first argument (excluding the program name) is the file
    // to open at startup, if any.
    let startup_arg = cli::interpret(std::env::args().skip(1));

    let window = WindowBuilder::new()
        .with_title(PRODUCT_NAME)
        .with_inner_size(dioxus::desktop::LogicalSize::new(1080.0, 720.0));
    // RFC-064 §4.3's CSP is deliberately not wired here: as specified it
    // blocks the framework's UI events (`connect-src`) and `document::eval`
    // (`script-src` without `'unsafe-eval'`). See `shell::csp` for the
    // evidence and what wiring it would take.
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(window)
                .with_menu(None)
                // The page is named before any effect runs (RFC-060 §11 item 2).
                .with_custom_head(omriss_components::startup_head()),
        )
        .with_context(detect_locale())
        .with_context(settings)
        .with_context(startup_arg)
        .launch(shell::app::App);
}
