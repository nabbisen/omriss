# omriss

[![crates.io](https://img.shields.io/crates/v/omriss?label=rust)](https://crates.io/crates/omriss)
[![Rust Documentation](https://docs.rs/omriss/badge.svg?version=latest)](https://docs.rs/omriss)
[![Dependency Status](https://deps.rs/crate/omriss/latest/status.svg)](https://deps.rs/crate/omriss)
[![License](https://img.shields.io/github/license/nabbisen/omriss)](LICENSE)

**Omriss Editor** — a next-generation text editor that helps you clarify ideas
and refine them, consideration by consideration, **layer by layer**.

omriss treats a Markdown document as a stack of layers: the Document Map is
the map, and each section is a layer you can focus on in isolation. You
zoom into one section, refine just that thought, and zoom back out — without
the rest of the document getting in the way, and without the editor ever
rewriting a byte you didn't touch.

![screenshot-01](docs/assets/screenshot-01.png)

## Design principles

1. **The raw Markdown text is the canonical document.** The outline tree is a
   derived index, rebuilt after every committed edit. Saving always writes the
   canonical text back verbatim — never a serialization of an AST.
2. **Editing one section never rewrites unrelated source bytes.** Replacing a
   section body splices exactly that byte range; every byte outside it is
   preserved bit-for-bit (whitespace quirks, HTML comments, code fences,
   CRLF line endings, missing trailing newlines — all of it).
3. **Conflicts are impossible by construction.** Every edit carries the
   document revision it was composed against; stale edits are rejected before
   any mutation (optimistic concurrency, RFC-002/RFC-008).
4. **Undo/redo are first-class, byte-exact mutations** with bounded history
   (RFC-044).
5. **The GUI speaks your language.** UI strings are catalog-based with
   English fallback; English and Japanese ship in the MVP (RFC-043).

## Workspace layout

| Crate | Role |
| --- | --- |
| `crates/omriss` | Document engine: canonical text, outline index over `pulldown-cmark`, section-body edits, undo/redo. No GUI dependencies. |
| `crates/omriss-ui` | Renderer-independent GUI logic: editor session, focus navigation with back/forward history, i18n catalogs. |
| `crates/omriss-app` | Desktop shell: Dioxus components on the system WebView, file dialogs via `rfd`. |

Design documents live in [`rfcs/`](rfcs/) (see the
[RFC index](rfcs/README.md) and the lifecycle policy in
[`rfcs/done/000-rfc-lifecycle-policy.md`](rfcs/done/000-rfc-lifecycle-policy.md)).
The user guide sources live in [`docs/`](docs/) as an mdBook.

## Building

Requires Rust 1.88+ (edition 2024). Core and UI logic build everywhere:

```sh
cargo build            # builds omriss and omriss-ui (default members)
cargo test             # runs the full unit + golden integration suite
```

### Desktop GUI

`omriss-app` links the platform WebView, so it is excluded from the
default members. On Linux (Debian/Ubuntu) install the native packages first:

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev
cargo run -p omriss-app
```

Windows (WebView2) and macOS (WKWebView) need no extra packages:

```sh
cargo run -p omriss-app
```

## Using omriss

* The left Document Map lists and organizes sections of the open document.
* Click a section to **focus** it: you see its breadcrumb path, its text in
  the Writing Area, and its direct subsections as navigation links.
* Edit the body and commit; only that section's bytes change.
* **Back / Forward** retrace your focus history like a browser.
* **Undo / Redo** restore the document text byte-exactly.
* Switch the GUI language (English / 日本語) from the status bar at any time.

## Platform Support

omriss runs on Linux, macOS, and Windows. See [PLATFORMS.md](PLATFORMS.md)
for the support matrix, required system packages, and known platform
constraints.

> **Unsigned builds:** Early releases are not code-signed. On macOS, use
> Right-click → Open to bypass Gatekeeper. Verify the SHA-256 checksum
> published with each release.

## Documentation

```sh
cargo install mdbook
mdbook serve docs
```

Full docs cover working in layers, structural editing, keyboard reference,
known limitations, and the architecture.

## Release Policy

Public releases require explicit sign-off from the product owner. See
[RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) for the full checklist
including data-integrity tests, platform smoke tests, and the required
sign-off form.

## License

Apache-2.0 — see [LICENSE](LICENSE). Copyright (c) nabbisen.
