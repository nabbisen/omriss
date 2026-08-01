//! Format detection: deciding a file's [`DocumentFormat`] from its path and
//! content (RFC-053 §10).
//!
//! Extension mapping is owned by RFC-052 §5.1 and is binding here:
//!
//! ```text
//! md, markdown, mdown, txt -> Markdown   (shipped behavior; must not regress)
//! json                     -> Json
//! toml                     -> Toml
//! yaml, yml                -> YamlExperimental
//! (anything else)          -> PlainText
//! ```
//!
//! Detection here is extension-only, matched case-insensitively. Content-based
//! disambiguation (RFC-053 §10 step 2) has no case to resolve yet: every
//! extension in the mapping above is unambiguous, and an absent or unmapped
//! extension conservatively falls back to `PlainText` rather than guessing a
//! structured format from content omriss cannot yet parse (RFC-052
//! FR-052-001: "never a silent guess").
//!
//! Nothing calls this module yet; it is not wired into the document session
//! or the file dialog (that is later RFC-053/054 work).

use std::path::Path;

use super::DocumentFormat;

/// How confident a [`DocumentFormat`] classification is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DetectionConfidence {
    No,
    Maybe,
    Likely,
    Certain,
}

/// Classifies `path`/`text` into a [`DocumentFormat`] per the mapping above.
///
/// `path` is `Option` because an unsaved buffer has no path; in that case
/// detection falls back to `PlainText` without inspecting `text`, since no
/// format-specific parser exists yet to validate a content-based guess.
pub fn detect_format(path: Option<&Path>, _text: &str) -> DocumentFormat {
    path.and_then(extension_of)
        .and_then(|ext| format_for_extension(&ext))
        .unwrap_or(DocumentFormat::PlainText)
}

/// Reports how confident [`detect_format`] is that `path`/`text` classifies
/// as `format`.
///
/// There are two ways to reach the `PlainText` fallback in [`detect_format`],
/// and they are graded differently: an extension present but not in the
/// RFC-052 §5.1 mapping (e.g. `a.xyz`) is positive evidence the file is not a
/// format omriss knows, so it is `Certain`, same as any other extension-backed
/// match. Only the absence of a path at all — an unsaved buffer, where there
/// is no extension evidence to consult — is `Maybe`, since that case has not
/// been confirmed by any real content inspection. Any other format is `No`.
pub fn confidence_for(
    format: DocumentFormat,
    path: Option<&Path>,
    text: &str,
) -> DetectionConfidence {
    if detect_format(path, text) != format {
        return DetectionConfidence::No;
    }
    match path.and_then(extension_of) {
        Some(_) => DetectionConfidence::Certain,
        None => DetectionConfidence::Maybe,
    }
}

fn extension_of(path: &Path) -> Option<String> {
    Some(path.extension()?.to_str()?.to_ascii_lowercase())
}

fn format_for_extension(ext: &str) -> Option<DocumentFormat> {
    match ext {
        "md" | "markdown" | "mdown" | "txt" => Some(DocumentFormat::Markdown),
        "json" => Some(DocumentFormat::Json),
        "toml" => Some(DocumentFormat::Toml),
        "yaml" | "yml" => Some(DocumentFormat::YamlExperimental),
        _ => None,
    }
}
