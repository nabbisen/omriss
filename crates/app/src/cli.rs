//! Command-line argument interpretation (RFC-063).
//!
//! [`interpret`] is a pure `(args) -> StartupArg` mapping, unit-tested
//! without a GUI — the same shape as [`crate::input::keyboard::interpret`]'s
//! pure event-to-command mapping and
//! [`crate::storage::settings::AppSettings::push_recent`]. No I/O, no
//! Dioxus, no flag parsing beyond recognizing that an argument starting
//! with `-` is not a path (RFC-063 §4.1/§5.3).

/// What startup should do, derived from the command line (RFC-063 §5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupArg {
    /// No argument was given. Welcome screen, unchanged (§5.5).
    None,
    /// The first argument, to be opened as a file path (§5.1). Existence,
    /// readability, and directory-vs-file are **not** checked here —
    /// `file_dialog::open_markdown_path` already owns that failure path
    /// (§5.2), the same one the Recent Files list uses.
    Path(String),
    /// The first argument looks like an option (starts with `-`), which
    /// omriss does not parse (§4.1) and never treats as a path (§5.3).
    RejectedOption(String),
}

/// Pure interpretation of the command line, *excluding* the program name
/// (`argv[0]`), into a [`StartupArg`]. The first argument decides
/// everything; any further arguments are ignored, not an error (§5.1) —
/// a future RFC may give them meaning, and treating them as an error now
/// would make that a breaking change later.
pub fn interpret(mut args: impl Iterator<Item = String>) -> StartupArg {
    match args.next() {
        None => StartupArg::None,
        Some(first) if first.starts_with('-') => StartupArg::RejectedOption(first),
        Some(first) => StartupArg::Path(first),
    }
}

#[cfg(test)]
mod tests;
