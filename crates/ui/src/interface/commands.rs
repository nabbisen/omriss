//! Command registry: a static catalogue of application commands (RFC-022).
//!
//! Every command has an id, an i18n title key, and an optional default
//! shortcut string. Quick Actions and the keyboard-help page both draw from
//! this single source so they stay consistent.
//!
//! Commands are checked for availability by the desktop shell; the registry
//! itself carries no runtime state.

/// One entry in the command registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    /// Stable dot-namespaced identifier (never localized).
    pub id: &'static str,
    /// i18n key whose value is the human-readable command title.
    pub title_key: &'static str,
    /// Human-readable shortcut label (platform-neutral; displayed as-is).
    pub shortcut: Option<&'static str>,
}

/// All registered application commands, in display order.
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "file.open",
        title_key: "menu.file.open",
        shortcut: Some("Ctrl+O"),
    },
    CommandSpec {
        id: "file.new",
        title_key: "menu.file.new",
        shortcut: None,
    },
    CommandSpec {
        id: "file.save",
        title_key: "menu.file.save",
        shortcut: Some("Ctrl+S"),
    },
    CommandSpec {
        id: "file.save_as",
        title_key: "menu.file.save_as",
        shortcut: Some("Ctrl+Shift+S"),
    },
    CommandSpec {
        id: "edit.undo",
        title_key: "toolbar.undo",
        shortcut: Some("Ctrl+Z"),
    },
    CommandSpec {
        id: "edit.redo",
        title_key: "toolbar.redo",
        shortcut: Some("Ctrl+Y"),
    },
    CommandSpec {
        id: "nav.back",
        title_key: "nav.back",
        shortcut: Some("Alt+\u{2190}"),
    },
    CommandSpec {
        id: "nav.forward",
        title_key: "nav.forward",
        shortcut: Some("Alt+\u{2192}"),
    },
    CommandSpec {
        id: "nav.parent",
        title_key: "nav.parent",
        shortcut: Some("Esc"),
    },
    CommandSpec {
        id: "view.raw",
        title_key: "raw.title",
        shortcut: Some("Ctrl+`"),
    },
    CommandSpec {
        id: "view.preview",
        title_key: "editor.preview",
        shortcut: Some("Ctrl+Shift+P"),
    },
    CommandSpec {
        id: "search.open",
        title_key: "search.title",
        shortcut: Some("Ctrl+F"),
    },
    CommandSpec {
        id: "palette.open",
        title_key: "palette.title",
        shortcut: Some("Ctrl+P"),
    },
    CommandSpec {
        id: "view.stats",
        title_key: "view.stats",
        shortcut: None,
    },
];

/// Returns all commands whose title (looked up through `t_fn`) contains
/// `query` (case-insensitive). Used by the palette filter.
pub fn filter_commands<'a>(
    commands: &'a [CommandSpec],
    query: &str,
    t_fn: &impl Fn(&'static str) -> String,
) -> Vec<&'a CommandSpec> {
    if query.trim().is_empty() {
        return commands.iter().collect();
    }
    let q = query.to_lowercase();
    commands
        .iter()
        .filter(|cmd| {
            let title = t_fn(cmd.title_key).to_lowercase();
            let id = cmd.id.to_lowercase();
            title.contains(&q) || id.contains(&q)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_t(key: &'static str) -> String {
        key.to_string()
    }

    #[test]
    fn filter_returns_all_on_empty() {
        let all = filter_commands(COMMANDS, "", &mock_t);
        assert_eq!(all.len(), COMMANDS.len());
    }

    #[test]
    fn filter_by_id_prefix() {
        let file_cmds = filter_commands(COMMANDS, "file", &mock_t);
        assert!(file_cmds.iter().all(|c| c.id.starts_with("file.")));
    }

    #[test]
    fn all_commands_have_nonempty_id_and_title() {
        for cmd in COMMANDS {
            assert!(!cmd.id.is_empty());
            assert!(!cmd.title_key.is_empty());
        }
    }
}
