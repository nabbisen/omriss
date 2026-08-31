# Getting Started

## Install & run

Core logic builds with stable Rust 1.88+:

```sh
cargo build
cargo test
```

The desktop GUI uses the system WebView. On Linux, install the native
packages first:

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
                     libgtk-3-dev libxdo-dev libssl-dev
cargo run -p omriss
```

On Windows and macOS, `cargo run -p omriss` is enough.

Pass a file path as the first argument to open it immediately, the same way
"Open with…" from a file manager would:

```sh
cargo run -p omriss -- notes.md
```

## Your first session

1. **Open** a Markdown file from the toolbar, name one on the command line
   (see above), or just start typing in a new document and save it later.
2. The left Document Map shows the document's sections.
3. Click a section to focus it. Refine its text in the Writing Area, then move on.
4. **Save** writes your original file back with only your edits applied.
