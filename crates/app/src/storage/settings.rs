//! Application settings: persistent local configuration (RFC-036).
//!
//! Settings are stored in the OS-appropriate user config directory:
//!   Linux   : ~/.config/omriss/settings.toml
//!   macOS   : ~/Library/Application Support/omriss/settings.toml
//!   Windows : %APPDATA%\omriss\settings.toml
//!
//! INVARIANT (RFC-036): settings never touch or reference the content of
//! Markdown documents. The settings file may hold file *paths* in the
//! recent-files list, but never document bodies or headings.
//!
//! Failure policy: a missing or corrupt settings file silently falls back to
//! defaults; the user is never blocked from opening the editor by a bad
//! config file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Maximum number of paths kept in the recent-files list.
const MAX_RECENT: usize = 10;

/// All persisted application preferences (RFC-036 §4 initial settings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Ordered list of recently opened file paths (most recent first).
    #[serde(default)]
    pub recent_files: Vec<String>,

    /// Editor font size in points.
    #[serde(default = "default_font_size")]
    pub font_size: u8,

    /// Whether long lines wrap in the body editor.
    #[serde(default = "default_true")]
    pub line_wrap: bool,
}

fn default_font_size() -> u8 {
    14
}
fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            recent_files: Vec::new(),
            font_size: default_font_size(),
            line_wrap: default_true(),
        }
    }
}

impl AppSettings {
    /// Returns the platform-specific path for the settings file, or `None`
    /// when the config directory cannot be determined.
    pub fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("omriss").join("settings.toml"))
    }

    /// Loads settings from disk, falling back to `Default` on any error.
    /// Errors are silently discarded per RFC-036 failure policy.
    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str::<Self>(&text).unwrap_or_default()
    }

    /// Saves settings to disk. Errors are silently discarded (RFC-036), but a save
    /// that fails part-way can no longer damage the file: it goes through the same
    /// durable write as a document (RFC-068 §7.1), so the old settings are intact
    /// until the new ones are complete and renamed over them.
    pub fn save(&self) {
        if let Some(path) = Self::settings_path() {
            let _ = self.save_to(&path);
        }
    }

    /// Saves to `path`, creating its folder, and says why it could not.
    pub(crate) fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        crate::file::durable_write::write_durably(path, text.as_bytes())
    }

    /// Adds `path` to the front of the recent-files list, deduplicating and
    /// capping at `MAX_RECENT` entries.
    pub fn push_recent(&mut self, path: &str) {
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_string());
        self.recent_files.truncate(MAX_RECENT);
    }

    /// Removes a specific path from the recent-files list.
    #[allow(dead_code)]
    pub(crate) fn remove_recent(&mut self, path: &str) {
        self.recent_files.retain(|p| p != path);
    }

    /// Returns only the recent paths that still exist on disk.
    pub fn valid_recent_files(&self) -> Vec<String> {
        self.recent_files
            .iter()
            .filter(|p| std::path::Path::new(p.as_str()).exists())
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests;
