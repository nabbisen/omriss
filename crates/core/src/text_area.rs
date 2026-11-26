//! What a browser text area shows (RFC-084).
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
