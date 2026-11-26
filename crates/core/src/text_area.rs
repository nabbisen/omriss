//! What a browser text area shows, and the newline a document writes (RFC-084).
//!
//! A text area's value always uses `\n`: the browser turns `\r\n` and a lone
//! `\r` into `\n` before the application sees it. Every place that compares a
//! text-area draft with source text compares in this view, so that a line
//! ending the user did not touch is never mistaken for an edit.

/// What a browser text area shows for `text`: every `\r\n` and lone `\r`
/// becomes `\n`.
pub fn as_shown_in_a_text_area(text: &str) -> String {
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
/// structural edits and for the application's text areas alike (RFC-084 §10.3).
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
}
