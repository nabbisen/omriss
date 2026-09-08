//! RFC-064 §4.3: Content-Security-Policy for the app's WebView.
//!
//! **Not yet wired into `main.rs`.** As specified, this policy blocks
//! dioxus-desktop 0.7's own edit-streaming WebSocket (`connect-src 'none'`
//! vs. `edits.rs`'s `ws://127.0.0.1:{port}` channel that every UI update
//! streams through) and its inline interop `<script type="module">` bridge
//! (`protocol.rs`'s `module_loader`, whose content is templated per launch
//! with a fresh key/port, ruling out a hash-based allowlist) — applying it
//! leaves the window permanently blank. See the RFC-064 review request's
//! escalation for the evidence and the options under consideration. Kept
//! here, tested, so the literal RFC text and the eventual real fix have a
//! home; `#[allow(dead_code)]` until `main.rs` can apply it.

/// Verbatim policy from RFC-064 §4.3. `connect-src 'none'` is load-bearing:
/// omriss makes no network requests by design, so this costs nothing and turns
/// exfiltration into a console error even if the escaping and scheme checks in
/// `omriss_core::doc::preview` are ever bypassed.
#[allow(dead_code)]
pub const CONTENT_SECURITY_POLICY: &str =
    "default-src 'self'; script-src 'self'; img-src 'self' data:; connect-src 'none'";

/// The `<meta>` tag to pass to `Config::with_custom_head`.
#[allow(dead_code)]
pub fn content_security_policy_meta_tag() -> String {
    format!(r#"<meta http-equiv="Content-Security-Policy" content="{CONTENT_SECURITY_POLICY}">"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_tag_carries_every_required_directive() {
        let tag = content_security_policy_meta_tag();
        for directive in [
            "default-src 'self'",
            "script-src 'self'",
            "img-src 'self' data:",
            "connect-src 'none'",
        ] {
            assert!(tag.contains(directive), "missing {directive:?} in {tag:?}");
        }
    }

    #[test]
    fn never_loosens_to_unsafe_inline() {
        // RFC-064 §8: a styling conflict is escalated, never papered over
        // with `'unsafe-inline'`.
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-inline"));
    }

    #[test]
    fn is_a_well_formed_meta_tag() {
        let tag = content_security_policy_meta_tag();
        assert!(tag.starts_with(r#"<meta http-equiv="Content-Security-Policy" content=""#));
        assert!(tag.ends_with(r#"">"#));
    }
}
