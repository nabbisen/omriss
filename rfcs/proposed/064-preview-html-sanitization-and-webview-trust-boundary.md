# RFC-064: Preview HTML Sanitization and WebView Trust Boundary

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate
**Status.** Proposed
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

```text
default-src 'self'; script-src 'self'; img-src 'self' data:; connect-src 'none'
```

`connect-src 'none'` is the load-bearing clause: omriss makes no network
requests by design, so this costs nothing and turns exfiltration into a console
error even if layers 1 and 2 are one day bypassed.

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
3. A CSP is set on the window, and its absence fails a test.
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
