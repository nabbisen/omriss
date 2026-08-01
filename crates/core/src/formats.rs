//! Document format identification (RFC-053).
//!
//! This module names the format a file's content should be interpreted and
//! edited as. It classifies files; it does not parse them. Format adapters
//! that understand each format's structure are introduced in later RFC-053
//! slices (S4 `MarkdownAdapter`, S5 `PlainTextAdapter`) and in RFC-054/055/056
//! (JSON, TOML, YAML). Nothing in this module is wired into the document
//! session or the file dialog yet.

pub mod adapter;
pub mod detection;
pub mod draft;
pub mod edit;
pub mod error;
pub mod focused_content;
pub mod markdown;
pub mod structure;
pub mod unsupported;

/// The document format a file's content should be interpreted and edited as
/// (RFC-053 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    /// Markdown source: headings form the navigable outline (RFC-002..047).
    Markdown,
    /// Strict JSON (RFC 8259). Not yet implemented; see RFC-054.
    Json,
    /// TOML. Not yet implemented; see RFC-055.
    Toml,
    /// YAML feasibility candidate; read-only until RFC-056 accepts editing.
    YamlExperimental,
    /// Shown as plain file text with no synthetic structure (RFC-052 §5.2).
    PlainText,
    /// Content omriss should not attempt, such as non-UTF-8 or oversized
    /// input (RFC-052 §5.2). Not produced by extension-based detection.
    Unsupported,
}
