# RFC-045: Markdown Preview Pane

**Project:** omriss — Omriss Editor
**Milestone:** Post-MVP Expansion
**Status.** Implemented (v0.12.0) — one presentation defect recorded below
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer


> **Defect recorded 2026-09-08, not yet fixed.** `Tag::Image`'s handler reads
> the `alt` attribute from the link *title* (`![alt](url "title")`'s optional
> third component), not from the image's own accumulated inline text. For an
> ordinary `![alt](url)` the title is empty, so alt text has never rendered —
> the output is `<img alt="">` with the alt text leaking out as a sibling text
> node instead.
>
> Found while writing RFC-064's sanitization tests; pre-existing, unrelated to
> that work, and with no security implication. No test caught it because none
> previously exercised an image with alt text. Presentation only, so it is not
> release-blocking, but it does affect screen-reader users and belongs with
> RFC-060's accessibility validation if not fixed sooner.

---

## 1. Summary

Add an optional rendered-HTML preview of the focused section body, toggled
inline without leaving Focus Edit mode.

## 2. Goals

- Render the focused section body as HTML using pulldown-cmark.
- Toggle preview on/off within the focus editor.
- Preserve the source-preservation invariant: preview is read-only.
- Keep the implementation additive; existing view modes are unchanged.

## 3. Non-Goals

- No WYSIWYG editing in preview.
- No full-document preview (focus-scoped only for now).
- No custom theme for rendered HTML in this RFC.
- No preview printing or export (separate future RFC).

## 4. Design

### Rendering

`omriss` gains a `preview` module exposing:

```rust
pub fn section_html(doc: &Document, id: NodeId) -> Option<String>
pub fn document_html(doc: &Document) -> String
```

Both use `pulldown-cmark::html::push_html` over the relevant source range.
The `html` feature of pulldown-cmark must be enabled.

### UI Integration

A `preview_open: Signal<bool>` is added to the App shell alongside
`search_open` and `palette_open`. When `preview_open` is true and the view
is in Focus mode, the `FocusEditor` renders a `PreviewPane` instead of the
textarea. The editor textarea is unmounted (not merely hidden) so the
section body is committed before switching to preview.

Toggle: a **Preview** button in the editor-actions bar; keyboard shortcut
`Ctrl+Shift+P`. Pressing the shortcut or the button a second time returns
to edit mode.

### PreviewPane component

Renders the HTML string via `dangerous_inner_html` inside a scoped div.
The rendered output is sandboxed by CSS to avoid layout pollution.

## 5. Accessibility

- The preview region has `role="region"` and an accessible label.
- The toggle button announces the current mode ("Edit" or "Preview").
- Screen readers can read the rendered paragraph text.

## 6. Validation and Test Plan

- `section_html` produces valid HTML for all existing fixture bodies.
- Toggling preview commits any pending draft before rendering.
- Toggling back to edit restores the textarea with the correct content.
- Keyboard shortcut works from the focused section view.

## 7. Acceptance Criteria

- User can see rendered Markdown without leaving the section.
- Preview is read-only; no editing occurs in preview mode.
- Source bytes are not modified by viewing the preview.

## 8. Dependencies

- RFC-012 (Focus Editor UI)
- RFC-014 (Keyboard Interaction)
