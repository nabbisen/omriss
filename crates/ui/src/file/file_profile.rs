//! File-level text profile for encoding and line-ending integrity (RFC-018).
//!
//! Detection happens at load time; the profile is stored in the session so
//! the status bar and save logic can report and preserve file characteristics.
//! `omriss-core` stays UTF-8 plain text; profile data never affects source
//! bytes outside the edited range.

/// Dominant newline style detected in the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlinePolicy {
    /// All line endings are LF (`\n`).
    #[default]
    Lf,
    /// All line endings are CRLF (`\r\n`).
    Crlf,
    /// The file uses both LF and CRLF; unrelated bytes are preserved exactly.
    Mixed,
}

impl NewlinePolicy {
    /// Short display label for the status bar.
    pub fn label(self) -> &'static str {
        match self {
            NewlinePolicy::Lf => "LF",
            NewlinePolicy::Crlf => "CRLF",
            NewlinePolicy::Mixed => "Mixed",
        }
    }
}

/// Summary of file-level text characteristics detected at open time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileTextProfile {
    pub newline: NewlinePolicy,
    /// The source bytes started with a UTF-8 BOM (`EF BB BF`); it was stripped
    /// before passing text to core. On save the BOM is re-prepended.
    pub had_utf8_bom: bool,
    /// The source text ended with a newline character.
    pub had_trailing_newline: bool,
}

impl FileTextProfile {
    /// Analyses `source` (after any BOM stripping) and returns its profile.
    pub fn detect(source: &str, had_bom: bool) -> Self {
        let crlf = source.matches("\r\n").count();
        let lf_only = source
            .char_indices()
            .filter(|&(i, ch)| ch == '\n' && (i == 0 || source.as_bytes()[i - 1] != b'\r'))
            .count();

        let newline = match (crlf > 0, lf_only > 0) {
            (true, false) => NewlinePolicy::Crlf,
            (false, true) => NewlinePolicy::Lf,
            (true, true) => NewlinePolicy::Mixed,
            (false, false) => NewlinePolicy::Lf, // no newlines at all → default LF
        };

        let had_trailing_newline = source.ends_with('\n');

        Self {
            newline,
            had_utf8_bom: had_bom,
            had_trailing_newline,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lf_detected() {
        let p = FileTextProfile::detect("# A\nbody\n", false);
        assert_eq!(p.newline, NewlinePolicy::Lf);
        assert!(p.had_trailing_newline);
    }

    #[test]
    fn crlf_detected() {
        let p = FileTextProfile::detect("# A\r\nbody\r\n", false);
        assert_eq!(p.newline, NewlinePolicy::Crlf);
        assert!(p.had_trailing_newline);
    }

    #[test]
    fn mixed_detected() {
        let p = FileTextProfile::detect("# A\r\nbody\nmore\r\n", false);
        assert_eq!(p.newline, NewlinePolicy::Mixed);
    }

    #[test]
    fn no_trailing_newline() {
        let p = FileTextProfile::detect("# A\nbody", false);
        assert!(!p.had_trailing_newline);
    }

    #[test]
    fn bom_recorded() {
        let p = FileTextProfile::detect("# A\n", true);
        assert!(p.had_utf8_bom);
    }
}
