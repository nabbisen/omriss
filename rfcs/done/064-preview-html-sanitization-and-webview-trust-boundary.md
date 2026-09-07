# RFC-064: Preview HTML Sanitization and WebView Trust Boundary

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate
**Status.** Implemented (main, unreleased) — H1 (escaping), H2 (scheme
checking) and H4 (documentation) landed and close the finding. **H3 (the CSP) is
deliberately not wired**, deferred to 0.18.0 for cross-platform verification —
§4.3 records why the originally-specified policy was wrong and §9 why the
corrected one waits. `crates/app/src/shell/csp.rs` holds it, tested and
dead-coded, in the meantime.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, security reviewer, QA engineer
**Depends on:** RFC-045
**Related RFCs:** RFC-010, RFC-017, RFC-033, RFC-039, RFC-042

---

## 1. Summary

The Markdown preview renders author-supplied HTML verbatim into the
application's own WebView, with no Content-Security-Policy and no scheme
checking on links. Opening someone else's Markdown file and pressing Preview
executes their JavaScript inside omriss.

This RFC closes that, and — more importantly — writes down a trust boundary the
project has never named. Sixty-three RFCs discuss preserving the user's bytes;
none asks what those bytes are allowed to *do* when rendered.

**Source of this RFC:** AUDIT-0170-001, reproduced independently by the
architect before acceptance.

## 2. The defect

`render_html` passes `Event::Html` and `Event::InlineHtml` through unchanged
(`crates/core/src/doc/preview.rs:165-168`), and the result is handed to Dioxus
`dangerous_inner_html` (`crates/app/src/components/preview_pane.rs:51`). The
window is built with no CSP (`crates/app/src/shell/app.rs:39-44`). Link
destinations are HTML-escaped but never scheme-checked.

Executed against `omriss_core::section_html`, verbatim:

```text
in   <img src=x onerror="alert(1)">
out  <img src=x onerror="alert(1)">

in   [c](javascript:alert(1))
out  <p><a href="javascript:alert(1)">c</a></p>
```

Both round-trip unescaped.

### Why this matters more than a normal XSS

A browser tab's XSS is bounded by an origin. This is not a browser tab. The
WebView is the application, so script running in it shares the process with the
document the user has open — it can read the open document out of the DOM and
exfiltrate it, overlay a convincing dialog on the app's own chrome, or navigate
the view. This is the only defect in the 0.17.0 audit whose blast radius extends
beyond the file being edited.

The delivery path is ordinary: a shared note, a README, a downloaded template.
Markdown is a format people exchange, and omriss's entire premise is opening
files other people wrote.

## 3. Non-goals

1. **Not a sandbox.** The WebView keeps full-trust access to the app; this RFC
   does not attempt process isolation.
2. **Not a change to what is stored.** Raw HTML stays in the user's source text
   byte-for-byte. This RFC changes only what the *preview* renders. Source
   preservation is untouched and must remain so.
3. **No HTML sanitizer dependency.** Escaping is a smaller, more auditable
   answer than allow-listing tags, and matches what the preview is for.

## 4. Design

Three independent layers. Layer 1 alone closes the finding; 2 and 3 are defence
in depth and cost nothing.

### 4.1 Escape authored HTML in the preview

```rust
Event::Html(raw) | Event::InlineHtml(raw) => {
    // Show authored HTML as text. Rendering it would execute it: the
    // preview is injected into the app's own WebView.
    out.push_str(&html_escape(&raw));
}
```

This is a deliberate product decision, not only a security one: the preview is a
**reading aid for structure**, not a browser. A user who wants their HTML
rendered as HTML has the source view and an external tool. Showing the markup as
text is also more honest about what omriss is.

### 4.2 Scheme-check link and image destinations

Allow `http`, `https`, `mailto`, and relative paths in the `Tag::Link` and
`Tag::Image` arms. Drop the attribute otherwise, keeping the link text. `data:`
is excluded deliberately — `data:text/html` is a script vector.

### 4.3 Content-Security-Policy on the WebView

**The policy below is corrected. The original was wrong and would have shipped a
blank window.** It read:

```text
default-src 'self'; script-src 'self'; img-src 'self' data:; connect-src 'none'
```

justified as costing nothing "since the app makes no network requests by
design". That claim is true of *omriss* and false of the framework it runs on,
which is what matters to a CSP. Verified in `dioxus-desktop` 0.7.9's own source:

- **`connect-src 'none'` blocks every UI update.** `edits.rs` streams all
  mutations to the page over a loopback websocket —
  `TcpListener::bind((IpAddr::from([127, 0, 0, 1]), 0))`, an ephemeral port that
  can change mid-session. Blocked, the page loads and then never receives a
  single edit. The crate's own doc comment says it plainly: *"Using websockets
  does mean we need to handle security and content security policies
  ourselves."*
- **`script-src 'self'` blocks the interop bridge.** `protocol.rs`'s
  `module_loader` injects an inline `<script type="module">` whose content
  varies per launch (it embeds the edits path and connection key), which rules
  out a hash allowlist. `dioxus-desktop` 0.7 exposes no nonce for it.

Reproduced empirically as well as traced: the literal policy renders a blank
window; loosening `script-src`/`style-src` alone still renders blank, isolating
`connect-src` as sufficient on its own to break the app.

The corrected policy:

```text
default-src 'self'; script-src 'self' 'unsafe-inline';
img-src 'self' data:; connect-src ws://127.0.0.1:*
```

`connect-src` remains the load-bearing clause. Narrowed to the loopback
websocket scheme and host, it still turns an injected `fetch('https://…')` into
a console error — which is the exfiltration path that matters — while permitting
the framework's own channel. `'unsafe-inline'` on `script-src` is a real loss of
the backstop against event-handler attributes, and is accepted because §4.1's
escaping, not the CSP, is the actual fix; the CSP is defence in depth on top of
it.

## 5. Validation and test plan

- `section_html` and `document_html` never emit `<script`, `onerror=`,
  `onmouseover=`, `<iframe`, or `href="javascript:` for any input. Table-driven,
  with the four vectors from §2 plus `data:text/html` and `srcdoc`.
- Escaped HTML still round-trips: the source text is unchanged after a preview
  render, asserted byte-for-byte.
- The existing preview tests continue to pass, and any that assert raw HTML
  passthrough are **scaffolding** by the RFC-054 handoff's definition — their
  premise is what this RFC invalidates. Flag them in the review request.
- A fuzz target over `render_html` once RFC-066 lands.

## 6. Acceptance criteria

1. No authored HTML from document source reaches the WebView unescaped.
2. No `javascript:` or `data:` destination survives into rendered output.
3. A CSP is set on the window, and its absence fails a test. **Deferred to
   0.18.0** — see §9.
4. Source text is byte-identical across preview rendering.
5. `SECURITY.md` exists before this RFC is discussed anywhere public.

## 7. Open question for the owner

Escaping HTML is a **user-visible behaviour change**: a document that today
renders an embedded `<table>` or `<details>` in preview will show the markup as
text instead. That is the correct default for a local-first editor, and it is a
regression for anyone relying on it.

If preserving rendered HTML matters, the follow-up is an allow-list of inert
tags (`table`, `details`, `summary`, `sup`, `sub`, `br`) with all attributes
stripped — strictly more work and strictly more risk. **The recommendation is
escaping.** Ship it, note it in `known-limitations.md`, and revisit only with a
concrete user need.

## 9. H3 deferred to 0.18.0

The corrected §4.3 policy has been verified on **Linux only**. WebKitGTK,
WKWebView, and WebView2 differ in how they honour a `<meta>` CSP, and the
failure mode when one of them disagrees is a blank window — not a degraded
style, the whole application.

0.17.0 ships to the Microsoft Store, and its Windows smoke run has not yet been
performed. Wiring an unverifiable CSP into that release trades a catastrophic,
platform-specific regression risk against defence in depth that §4 already says
is not the fix: **H1 alone closes this finding**, and H1 and H2 are verified.

So `crates/app/src/shell/csp.rs` holds the corrected policy, documented and
unit-tested, marked `#[allow(dead_code)]`, and `main.rs` does not apply it. It
gets wired in 0.18.0, when it can be exercised on all three platforms in the
same smoke run that would catch it breaking.
