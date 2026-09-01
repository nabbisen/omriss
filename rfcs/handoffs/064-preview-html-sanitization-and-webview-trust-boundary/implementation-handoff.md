# RFC-064 Implementation Handoff

**Governing RFC:** [RFC-064](../../proposed/064-preview-html-sanitization-and-webview-trust-boundary.md)
**Milestone:** M12 hardening — 0.17.0 ship gate
**Sequence:** independent; may run in parallel with RFC-065/066.

---

## 1. Purpose

Stop author-supplied HTML in a document from executing inside omriss's own
WebView.

This is the only defect in the 0.17.0 audit whose blast radius extends beyond
the file being edited. Everything else damages the open document; this reads it
and can send it somewhere.

## 2. What makes this different from ordinary XSS

A browser tab's XSS is bounded by an origin. This is not a browser tab — the
WebView *is* the application. Script running in it shares the process with the
open document: it can read the document out of the DOM and exfiltrate it,
overlay a convincing dialog on the app's own chrome, or navigate the view.

The delivery path is ordinary. omriss's whole premise is opening Markdown other
people wrote.

## 3. Slices

| Slice | Content | Layer |
|---|---|---|
| **H1** | Escape `Event::Html` / `Event::InlineHtml` in `render_html` | closes the finding on its own |
| **H2** | Scheme-check `Tag::Link` / `Tag::Image` destinations | defence in depth |
| **H3** | CSP on the WebView | defence in depth |
| **H4** | `known-limitations.md` entry for the behaviour change | required, see §5 |

H1 alone closes it. H2 and H3 cost almost nothing and are in scope.

## 4. Constraints

1. **Do not change what is stored.** Raw HTML stays in the user's source text
   byte-for-byte. This changes only what the *preview renders*. Assert it.
2. **No HTML sanitizer dependency.** Escaping is smaller and more auditable
   than tag allow-listing, and it is what the preview is for. Do not add
   `ammonia` or similar.
3. `data:` is **not** an allowed scheme in H2 — `data:text/html` is a script
   vector. `http`, `https`, `mailto`, and relative paths only.
4. In H2, drop the *attribute*, keep the link text. Do not drop the text.
5. `connect-src 'none'` is the load-bearing CSP clause. omriss makes no network
   requests by design, so it costs nothing and turns exfiltration into a console
   error even if H1 and H2 are one day bypassed.

## 5. This is a user-visible behaviour change, and must be documented

A document that today renders an embedded `<table>` or `<details>` in preview
will show the markup as text instead. That is correct for a local-first editor
and it is a regression for anyone relying on it. H4 is not optional.

If any existing preview test asserts raw HTML passthrough, it is **scaffolding**
by the RFC-054 handoff's definition — its premise is exactly what this RFC
invalidates. Changing it is expected; **flag it in the review request** with the
reasoning rather than slipping it in.

## 6. Required tests

- Table-driven: `section_html` and `document_html` never emit `<script`,
  `onerror=`, `onmouseover=`, `<iframe`, or `href="javascript:`. Include the
  four vectors from RFC-064 §2 plus `data:text/html` and `srcdoc`.
- Source text byte-identical across a preview render.
- The CSP is present — its absence must fail a test, not just be set.

## 7. Required review-request content

1. The table-driven test's inputs and outputs, pasted.
2. Explicit confirmation that source bytes are unchanged by rendering.
3. Any preview test changed, with the scaffolding reasoning.
4. A screenshot or description of what a document containing `<table>` now
   looks like in preview — the owner should see the regression, not read about
   it.

## 8. Escalation

Stop and report if:

- escaping cannot be done without altering stored source;
- a CSP that permits the app's own styles cannot be found — report the exact
  console error rather than loosening the policy to `unsafe-inline` and moving
  on.
