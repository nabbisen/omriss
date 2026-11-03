//! RFC-064 §4.3: Content-Security-Policy for the app's WebView.
//!
//! **Not wired into `main.rs`, deliberately, and not yet wireable.** The
//! policy below is RFC-064 §4.3's current text. Wiring it was attempted for H3
//! (0.18.0) and cannot be done as specified; the evidence is in the H3 review
//! request (`.git-exclude/review-request/064-h3-wire-csp/`). Two independent
//! blockers, found by running the application, not by reading the policy:
//!
//! 1. **`connect-src` blocks the framework's UI events.** dioxus-desktop sends
//!    every click and input to Rust by a synchronous `XMLHttpRequest` to
//!    `<base>/__events` — `dioxus://index.html/` on Linux and macOS,
//!    `http://dioxus.index.html/` on Windows (`protocol.rs`, `native.ts`). With
//!    `connect-src ws://127.0.0.1:*` that request is a violation and is dropped:
//!    the window renders (edits stream over the loopback websocket, which is
//!    allowed) and then ignores all input. Observed violation:
//!    `connect-src dioxus://index.html//__events`. Adding `'self'` to
//!    `connect-src` fixes this on Linux (verified); on Windows it is unverified.
//! 2. **`script-src` blocks `document::eval`.** dioxus-desktop runs it through
//!    `new AsyncFunction(...)`, which CSP treats as `eval` and permits only with
//!    `'unsafe-eval'`. omriss calls `document::eval` at eleven sites (modal and
//!    row-menu focus, the palette, the preview, the shell's key handling). With
//!    the policy applied, a dialog opens with nothing focused — Task 019's
//!    "Cancel is focused" is lost — and the other eval-driven behaviour fails
//!    the same way. Observed violation: `script-src eval`.
//!
//! `'unsafe-eval'` is never conceded (RFC-064 §4.3, H3 handoff §5.1), so (2)
//! cannot be fixed by editing this policy: it needs either an owner/architect
//! decision to concede it, or the eleven call sites to stop using `eval`.
//!
//! The constant and its tests stay so the policy has a tested home. The tests
//! pin which directives may carry `'unsafe-inline'` and refuse every other
//! `'unsafe-*'`, so a loosening cannot arrive unannounced when it is wired.

/// RFC-064 §4.3's policy as last corrected (2026-11-03). Narrowing any
/// directive risks a blank or unstyled window, which no build log shows — a
/// violation appears only in the WebView console. See the module doc: as
/// written, it also leaves the application unable to receive input.
#[allow(dead_code)]
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; \
     script-src 'self' 'unsafe-inline'; \
     style-src 'self' 'unsafe-inline'; \
     img-src 'self' data:; \
     connect-src ws://127.0.0.1:*";

/// The `<meta>` tag to pass to `Config::with_custom_head`.
#[allow(dead_code)]
pub fn content_security_policy_meta_tag() -> String {
    format!(r#"<meta http-equiv="Content-Security-Policy" content="{CONTENT_SECURITY_POLICY}">"#)
}

#[cfg(test)]
mod tests;
