//! Markdown preview rendering (RFC-045).
//!
//! Converts source ranges to HTML for read-only display. Implemented with a
//! manual event walker over the pulldown-cmark parser so no additional
//! feature flags or C build-script dependencies are required.
//!
//! Supported elements: headings H1-H6, paragraphs, bold, italic, inline code,
//! fenced code blocks, unordered and ordered lists, block quotes, links,
//! horizontal rules, hard/soft breaks, strikethrough, tables.
//!
//! The source-preservation invariant is unaffected: this module is read-only.

use pulldown_cmark::{Alignment, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Escapes HTML special characters in text content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// RFC-064 §4.2: whether `url` is safe to render as a link/image
/// destination in the preview's WebView. Allows `http`, `https`,
/// `mailto`, and scheme-less destinations (relative paths, and in-page
/// fragments like `#section`) — deliberately excludes every other
/// scheme, `data:` most of all: `data:text/html` is itself a script
/// vector, not merely an unusual one.
///
/// A destination with no scheme at all, or with a prefix that isn't
/// shaped like a URI scheme (RFC 3986: a letter, then letters/digits/`+`/
/// `-`/`.`, then `:`), is treated as scheme-less and allowed — the
/// allow-list below is what actually excludes anything dangerous, so
/// erring permissive here only affects whether an ordinary relative path
/// is recognised as one, never whether `javascript:`/`data:` gets through.
fn is_allowed_destination(url: &str) -> bool {
    let Some(colon) = url.find(':') else {
        return true; // no scheme: relative path or bare fragment
    };
    let scheme = &url[..colon];
    let looks_like_a_scheme = scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !looks_like_a_scheme {
        return true;
    }
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "mailto"
    )
}

fn heading_tag(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

// ── rendering ─────────────────────────────────────────────────────────────────

fn render_html(markdown: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(markdown, opts);
    let mut out = String::with_capacity(markdown.len() * 2);
    // Track table column alignment for rendering <th>/<td> cells.
    let mut table_alignments: Vec<Alignment> = Vec::new();
    let mut table_col: usize = 0;
    let mut in_table_head = false;

    for event in parser {
        match event {
            // ── block elements ────────────────────────────────────────────────
            Event::Start(Tag::Paragraph) => out.push_str("<p>"),
            Event::End(TagEnd::Paragraph) => out.push_str("</p>\n"),

            Event::Start(Tag::Heading { level, .. }) => {
                let n = heading_tag(level);
                out.push_str(&format!("<h{n}>"));
            }
            Event::End(TagEnd::Heading(level)) => {
                let n = heading_tag(level);
                out.push_str(&format!("</h{n}>\n"));
            }

            Event::Start(Tag::BlockQuote(_kind)) => out.push_str("<blockquote>\n"),
            Event::End(TagEnd::BlockQuote(_)) => out.push_str("</blockquote>\n"),

            Event::Start(Tag::CodeBlock(_)) => out.push_str("<pre><code>"),
            Event::End(TagEnd::CodeBlock) => out.push_str("</code></pre>\n"),

            Event::Start(Tag::List(None)) => out.push_str("<ul>\n"),
            Event::End(TagEnd::List(false)) => out.push_str("</ul>\n"),
            Event::Start(Tag::List(Some(_start))) => out.push_str("<ol>\n"),
            Event::End(TagEnd::List(true)) => out.push_str("</ol>\n"),

            Event::Start(Tag::Item) => out.push_str("<li>"),
            Event::End(TagEnd::Item) => out.push_str("</li>\n"),

            Event::Rule => out.push_str("<hr>\n"),

            // ── tables ────────────────────────────────────────────────────────
            Event::Start(Tag::Table(alignments)) => {
                table_alignments = alignments.clone();
                out.push_str("<table>\n");
            }
            Event::End(TagEnd::Table) => out.push_str("</table>\n"),

            Event::Start(Tag::TableHead) => {
                in_table_head = true;
                out.push_str("<thead><tr>\n");
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
                out.push_str("</tr></thead>\n<tbody>\n");
                table_col = 0;
            }
            Event::Start(Tag::TableRow) => {
                out.push_str("<tr>\n");
                table_col = 0;
            }
            Event::End(TagEnd::TableRow) => out.push_str("</tr>\n"),

            Event::Start(Tag::TableCell) => {
                let tag = if in_table_head { "th" } else { "td" };
                let align = match table_alignments.get(table_col) {
                    Some(Alignment::Left) => " style=\"text-align:left\"",
                    Some(Alignment::Right) => " style=\"text-align:right\"",
                    Some(Alignment::Center) => " style=\"text-align:center\"",
                    _ => "",
                };
                out.push_str(&format!("<{tag}{align}>"));
            }
            Event::End(TagEnd::TableCell) => {
                let tag = if in_table_head { "th" } else { "td" };
                out.push_str(&format!("</{tag}>\n"));
                table_col += 1;
            }

            // ── inline elements ───────────────────────────────────────────────
            Event::Start(Tag::Strong) => out.push_str("<strong>"),
            Event::End(TagEnd::Strong) => out.push_str("</strong>"),

            Event::Start(Tag::Emphasis) => out.push_str("<em>"),
            Event::End(TagEnd::Emphasis) => out.push_str("</em>"),

            Event::Start(Tag::Strikethrough) => out.push_str("<del>"),
            Event::End(TagEnd::Strikethrough) => out.push_str("</del>"),

            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            }) => {
                // RFC-064 §4.2: a disallowed scheme (javascript:, data:,
                // vbscript:, ...) drops only the destination attribute —
                // the link text (and title, if any) still render, just
                // as unlinked text wrapped in an otherwise-inert <a>.
                let href_attr = if is_allowed_destination(&dest_url) {
                    format!(" href=\"{}\"", html_escape(&dest_url))
                } else {
                    String::new()
                };
                let is_plain = link_type == LinkType::Autolink
                    || link_type == LinkType::Email
                    || title.is_empty();
                if is_plain {
                    out.push_str(&format!("<a{href_attr}>"));
                } else {
                    let t = html_escape(&title);
                    out.push_str(&format!("<a{href_attr} title=\"{t}\">"));
                }
            }
            Event::End(TagEnd::Link) => out.push_str("</a>"),

            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                // RFC-064 §4.2: same scheme check as links; a disallowed
                // source drops only the `src` attribute.
                let src_attr = if is_allowed_destination(&dest_url) {
                    format!(" src=\"{}\"", html_escape(&dest_url))
                } else {
                    String::new()
                };
                let alt = html_escape(&title);
                out.push_str(&format!("<img{src_attr} alt=\"{alt}\">"));
            }
            Event::End(TagEnd::Image) => {}

            // ── text ──────────────────────────────────────────────────────────
            Event::Text(text) => out.push_str(&html_escape(&text)),
            Event::Code(code) => {
                out.push_str("<code>");
                out.push_str(&html_escape(&code));
                out.push_str("</code>");
            }
            Event::Html(raw) | Event::InlineHtml(raw) => {
                // RFC-064 §4.1: show authored HTML as text, never render it.
                // The preview is injected into the app's own WebView via
                // `dangerous_inner_html` (`crates/app/src/components/preview_pane.rs`) —
                // passing raw markup through unchanged would execute
                // whatever script tags or event-handler attributes a
                // document's author (not necessarily this user) wrote,
                // inside the same process as the open document. The
                // preview is a reading aid for structure, not a browser;
                // a user who wants their HTML rendered as HTML has the
                // raw source view and an external tool.
                out.push_str(&html_escape(&raw));
            }
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("<br>\n"),

            Event::TaskListMarker(checked) => {
                if checked {
                    out.push_str("<input type=\"checkbox\" checked disabled> ");
                } else {
                    out.push_str("<input type=\"checkbox\" disabled> ");
                }
            }

            // Ignore footnote definitions in preview for simplicity.
            _ => {}
        }
    }
    out
}

// ── public API ────────────────────────────────────────────────────────────────

/// Returns the focused section body rendered as HTML, or `None` when the
/// node does not exist in the outline.
pub fn section_html(doc: &crate::Document, id: crate::NodeId) -> Option<String> {
    let node = doc.outline().node(id)?;
    let body = doc.source().get(node.body_range.as_range())?;
    Some(render_html(body))
}

/// Returns the full document rendered as a single HTML string, headings
/// included. Useful for whole-document export or reference.
pub fn document_html(doc: &crate::Document) -> String {
    render_html(doc.source())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    fn doc(md: &str) -> Document {
        Document::parse(md.to_string()).unwrap()
    }

    #[test]
    fn section_html_renders_bold() {
        let d = doc("# A\nHello **world**\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(html.contains("<strong>world</strong>"), "{html}");
    }

    #[test]
    fn document_html_includes_heading() {
        let d = doc("# Title\nbody\n");
        let html = document_html(&d);
        assert!(html.contains("<h1>Title</h1>"), "{html}");
    }

    #[test]
    fn empty_body_yields_empty_or_minimal_html() {
        let d = doc("# A\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(html.trim().is_empty() || html.contains("<p>"), "{html}");
    }

    #[test]
    fn unknown_node_returns_none() {
        let d = doc("# A\n");
        assert!(section_html(&d, crate::NodeId(999_999_999)).is_none());
    }

    #[test]
    fn japanese_body_does_not_panic() {
        let d = doc("# 日本語\n東京は日本の首都です。\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(html.contains("東京"), "{html}");
    }

    #[test]
    fn code_fence_renders_pre_code() {
        let d = doc("# A\n```rust\nfn main() {}\n```\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(html.contains("<pre><code>"), "{html}");
    }

    #[test]
    fn html_escapes_angle_brackets() {
        let d = doc("# A\n1 < 2 > 0\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(html.contains("&lt;"), "{html}");
        assert!(html.contains("&gt;"), "{html}");
        assert!(!html.contains("<2"), "{html}");
    }

    // ── RFC-064: preview HTML sanitization ──────────────────────────────

    /// Markers that must never appear as *live* (unescaped) markup in
    /// rendered output — RFC-064 §5/handoff §6's own list, restricted to
    /// the forms that actually require an unescaped `<` or `"` to matter.
    /// An attribute name like `onerror=` is expected, and safe, to survive
    /// as plain escaped text once its enclosing tag has been turned to
    /// text by H1 — that is the whole point of "renders as text, not as
    /// an element" (acceptance checklist). What must never survive is the
    /// literal, unescaped `<script`/`<iframe` open, or one of our own
    /// generated `href=`/`src=` attributes pointing at a disallowed
    /// scheme — both of which require a real `<` or `"`, not just the
    /// word appearing inside inert text.
    const DANGEROUS_MARKERS: &[&str] = &[
        "<script",
        "<iframe",
        "href=\"javascript:",
        "src=\"javascript:",
        "href=\"data:",
        "src=\"data:",
    ];

    fn assert_no_dangerous_markers(html: &str, source: &str) {
        for marker in DANGEROUS_MARKERS {
            assert!(
                !html.contains(marker),
                "rendered output contains {marker:?} for source {source:?}: {html}"
            );
        }
    }

    /// Table-driven per RFC-064 §5/handoff §6: `section_html` (and, via
    /// `document_html`, the whole-document path) must escape every one of
    /// these vectors rather than render them as live markup, for both the
    /// body of a section and the top-level document text.
    #[test]
    fn dangerous_html_never_reaches_the_rendered_output() {
        const VECTORS: &[&str] = &[
            // RFC-064 §2's own two reproductions, verbatim.
            r#"<img src=x onerror="alert(1)">"#,
            r#"[c](javascript:alert(1))"#,
            // Acceptance checklist's remaining escaping cases.
            r#"<iframe src="https://evil.example"></iframe>"#,
            r#"<script>alert(document.cookie)</script>"#,
            r#"<a href="x" onmouseover="alert(1)">click</a>"#,
            r#"<iframe srcdoc="&lt;script&gt;alert(1)&lt;/script&gt;"></iframe>"#,
            // Acceptance checklist's remaining link/image-scheme cases.
            r#"![i](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)"#,
            r#"[c](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)"#,
        ];

        for vector in VECTORS {
            let source = format!("# A\n{vector}\n");
            let d = doc(&source);
            let id = d.outline().root().children[0];
            let section = section_html(&d, id).unwrap();
            assert_no_dangerous_markers(&section, vector);

            let whole = document_html(&d);
            assert_no_dangerous_markers(&whole, vector);
        }
    }

    /// Complements the blanket marker check above: the attribute names
    /// (`onerror=`, `onmouseover=`, `srcdoc=`) are expected to survive as
    /// plain text once escaped — this pins that they do so *only* inside
    /// an escaped, inert tag, never a live one.
    #[test]
    fn event_handler_attributes_survive_only_as_escaped_inert_text() {
        let d = doc(
            "# A\n<img src=x onerror=\"alert(1)\">\n\n<a href=\"x\" onmouseover=\"alert(1)\">click</a>\n",
        );
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(
            html.contains("&lt;img src=x onerror=&quot;alert(1)&quot;&gt;"),
            "{html}"
        );
        assert!(
            html.contains("&lt;a href=&quot;x&quot; onmouseover=&quot;alert(1)&quot;&gt;"),
            "{html}"
        );
        assert_no_dangerous_markers(&html, "onerror=/onmouseover= vectors");
    }

    #[test]
    fn raw_html_block_and_inline_html_render_as_visible_text() {
        let d = doc("# A\n<script>alert(1)</script>\n\nSome <b>bold</b> text.\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        // The tags themselves are visible, escaped text, not live markup.
        assert!(html.contains("&lt;script&gt;"), "{html}");
        assert!(html.contains("&lt;b&gt;"), "{html}");
        assert_no_dangerous_markers(&html, "<script>/<b>");
    }

    #[test]
    fn javascript_scheme_link_drops_the_href_but_keeps_the_text() {
        let d = doc("# A\n[click me](javascript:alert(1))\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(!html.contains("javascript:"), "{html}");
        assert!(html.contains("click me"), "{html}");
    }

    #[test]
    fn data_scheme_image_drops_the_src() {
        // Alt-text fidelity for images is a pre-existing, unrelated concern
        // (`Tag::Image::title` is the optional `"title"` in `![alt](url
        // "title")`, not the alt text itself — this renderer has never
        // captured the accumulated inline text as `alt`) — out of RFC-064's
        // scope, flagged separately rather than fixed here.
        let d = doc("# A\n![evil](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(!html.contains("data:"), "{html}");
    }

    #[test]
    fn allowed_link_schemes_still_render_their_destination() {
        for (source, expected_href) in [
            ("[a](http://example.com)", "href=\"http://example.com\""),
            ("[a](https://example.com)", "href=\"https://example.com\""),
            (
                "[a](mailto:person@example.com)",
                "href=\"mailto:person@example.com\"",
            ),
            ("[a](relative/path.md)", "href=\"relative/path.md\""),
            ("[a](#fragment)", "href=\"#fragment\""),
        ] {
            let d = doc(&format!("# A\n{source}\n"));
            let id = d.outline().root().children[0];
            let html = section_html(&d, id).unwrap();
            assert!(
                html.contains(expected_href),
                "expected {expected_href:?} in rendered output for {source:?}: {html}"
            );
        }
    }

    #[test]
    fn allowed_image_scheme_still_renders_its_source() {
        let d = doc("# A\n![alt text](https://example.com/pic.png)\n");
        let id = d.outline().root().children[0];
        let html = section_html(&d, id).unwrap();
        assert!(
            html.contains("src=\"https://example.com/pic.png\""),
            "{html}"
        );
    }

    #[test]
    fn source_text_is_byte_identical_across_a_preview_render() {
        let source = "# A\n<img src=x onerror=\"alert(1)\">\n\n[c](javascript:alert(1))\n";
        let d = doc(source);
        let _ = document_html(&d);
        let _ = section_html(&d, d.outline().root().children[0]);
        assert_eq!(
            d.source(),
            source,
            "rendering the preview must never mutate the stored source"
        );
    }
}
