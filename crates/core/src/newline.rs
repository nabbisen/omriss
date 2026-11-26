//! Line endings: LF-normalised text, and the newline a document writes (RFC-084).
//!
//! A document may end its lines with `\r\n`, `\n` or a lone `\r`. Text that
//! comes from a widget which cannot carry the difference (a browser text area
//! delivers `\n` only) is compared with the document in its LF-normalised form,
//! so that a line ending nobody touched is never mistaken for an edit; text
//! written into a document takes the document's own newline.

/// `text` with every `\r\n` and every lone `\r` turned into `\n`: the form in
/// which a line-ending-blind widget shows and delivers text, and the one to
/// compare its drafts against.
pub fn normalize_to_lf(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

/// The newline `source` uses for text written into it: the one rule for core's
/// structural edits and for every text the application writes into a document
/// (RFC-084 §10.3).
///
/// `"\r\n"` if the document contains any CRLF — a document with any CRLF at
/// all should get CRLF insertions rather than an LF that would itself create a
/// mixed-ending file. Otherwise `"\r"` if its line breaks are lone `\r` only
/// (a classic Mac OS file: there is no LF anywhere), else `"\n"`. A single
/// sequence to *write*, not the three-way label the UI tracks for display.
pub fn document_newline(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else if source.contains('\r') && !source.contains('\n') {
        "\r"
    } else {
        "\n"
    }
}

/// Splits `s` at its first line break into `(line, terminator, rest)`, where
/// `terminator` is `"\r\n"`, `"\n"`, `"\r"`, or `""` (no break found) and `line`
/// never includes it. The one three-ending rule for every place in core that
/// finds the end of a line (RFC-084 §11.3): a split on `'\n'` alone misreads a
/// bare-CR file and keeps a CRLF's `\r` in the line.
pub(crate) fn split_first_line(s: &str) -> (&str, &str, &str) {
    match s.find(['\r', '\n']) {
        None => (s, "", ""),
        Some(i) => {
            let width = if s[i..].starts_with("\r\n") { 2 } else { 1 };
            (&s[..i], &s[i..i + width], &s[i + width..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newline_rule() {
        assert_eq!(document_newline(""), "\n");
        assert_eq!(document_newline("a\nb\n"), "\n");
        assert_eq!(document_newline("a\r\nb\r\n"), "\r\n");
        assert_eq!(document_newline("a\r\nb\nc"), "\r\n");
        // Bare CRs and nothing else: CR.
        assert_eq!(document_newline("a\rb\r"), "\r");
        // Bare CRs beside LFs (no CRLF): LF, as before.
        assert_eq!(document_newline("a\rb\nc"), "\n");
        // Bare CRs beside CRLF: CRLF.
        assert_eq!(document_newline("a\rb\r\nc"), "\r\n");
    }

    #[test]
    fn a_line_ends_at_any_of_the_three_endings() {
        assert_eq!(split_first_line("ab\ncd"), ("ab", "\n", "cd"));
        assert_eq!(split_first_line("ab\r\ncd"), ("ab", "\r\n", "cd"));
        assert_eq!(split_first_line("ab\rcd"), ("ab", "\r", "cd"));
        assert_eq!(split_first_line("ab\r\rcd"), ("ab", "\r", "\rcd"));
        assert_eq!(split_first_line("ab"), ("ab", "", ""));
        assert_eq!(split_first_line("ab\r"), ("ab", "\r", ""));
        assert_eq!(split_first_line("\nx"), ("", "\n", "x"));
    }
}
