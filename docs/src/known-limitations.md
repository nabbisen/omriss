# Known Limitations

This page documents the intentional limitations and deferred features in the
current version of omriss.

---

## JSON Limitations

JSON is supported from 0.17.0. You can open a `.json` file, navigate its
structure in the Document Map, edit values, and save with the rest of the file
byte-for-byte unchanged. The following are deliberately not in this release.

### Structure Cannot Be Changed

You can change what a value *is*, but not what keys or elements *exist*. Adding,
deleting, renaming, and reordering keys and array elements are all unavailable.
On a JSON document those structure actions are not shown at all, rather than
shown greyed out — there is no committed plan for when they arrive, so
displaying them as blocked buttons would promise something undated.

**Why:** Those operations synthesize punctuation — commas, braces, brackets — at
positions the editor has to infer. That is where the real risk to your file
lives, and it is being designed separately rather than folded in alongside value
editing.

**Owner:** RFC-054 Phase 4 — designed, not yet scheduled.

**Workaround:** Use the "Show this part as text" action on a group or list to
replace that whole part with JSON you write yourself, or edit the file in a
standard text editor.

### Null Values Are Read-Only

A value that is `null` is displayed but cannot be edited.

**Why:** Changing `null` to a string, number, or object is a change of type,
which is a structure change rather than a value edit — the same reason as above.

**Owner:** RFC-054 Phase 4 — designed, not yet scheduled.

**Workaround:** Use the containing group's "Show this part as text" action.

### Strict JSON Only

Comments (`//`, `/* */`) and trailing commas are not accepted. A file using them
opens in the plain file text view, and the Document Map (and the editor area, if
you leave the text view) says the file could not be read as JSON — this includes most `tsconfig.json`-style and VS Code
configuration files, which are JSONC rather than JSON.

**Why:** JSONC is a different format with no single agreed specification.
Accepting it silently would mean guessing which dialect you meant, and guessing
wrong writes damage into your file.

**By design** (RFC-054) — no change planned.

**Workaround:** The file's text is open and nothing is lost. You can edit it
there (choose **Edit**); once it is valid JSON, the structure appears in the
Document Map.

### Duplicate Keys Are Kept, Not Merged

If an object contains the same key twice, both appear in the Document Map and
each is edited independently. omriss will not drop one or combine them.

**Why:** Silently discarding a key would be a data loss, and which duplicate
"wins" varies between JSON parsers. Preserving what you actually wrote is the
safer answer even though the file is unusual.

**By design** (RFC-054) — no change planned.

### Other Structured Formats

TOML and YAML are not available yet; a `.toml` or `.yaml` file opens as plain
text. See [File Formats](./file-formats.md) for the current support table.

---

## Preview Limitations

**Owner:** RFC-055 (TOML) and RFC-056 (YAML).

### Raw HTML in Markdown Is Shown as Text, Not Rendered

Markdown preview used to render any HTML written directly in the document —
`<table>`, `<details>`, `<b>`, and so on — as live markup. As of this release,
raw HTML is shown as visible, escaped text instead: you will see `<table>`
rather than an actual table. Link and image destinations using a scheme other
than `http`, `https`, `mailto`, or a relative path (`javascript:`, `data:`,
and similar) are also no longer rendered — the destination is dropped and the
link text or image alt text is kept.

**Why:** The preview renders directly into the app's own window, not a
sandboxed browser tab, so live HTML or an unusual link scheme in a document
could run with the same privileges as the app itself. This was a real
trust-boundary gap (RFC-064): a document containing `<img src=x
onerror="...">` or a `javascript:` link could execute code the moment its
preview was viewed. Escaping is the fix; there is no sandboxing layer in this
release to render such HTML safely instead.

**By design** (RFC-064) — no change planned.

**Workaround:** The plain file text view (Ctrl+\`) always shows your source
exactly as written, HTML included. If you need to see how a document's HTML
actually renders, open it in a web browser or a dedicated Markdown renderer
outside omriss.

---

## Heading Style Limitations

### Setext Headings Cannot Be Moved In or Out

Setext headings (underlined with `===` or `---`) are displayed correctly in
the Document Map but cannot be moved in or out using structure actions.

**Why:** ATX headings use a simple prefix change (`##` → `#`); Setext
headings require replacing the underline character on a different line, which
creates a more complex source range replacement.

**Owner:** RFC-078 — H1 and H2 become changeable in 0.18.0; beyond two levels stays refused.

**Workaround:** Use the plain file text view to inspect the heading, then
convert it to an ATX heading in an external editor before using the structure
actions normally.

---

## Navigation Limitations

### Closing the Window Does Not Warn About Unsaved Changes

Closing the window while an edit is unsaved discards that edit without asking.
Opening another file or starting a new one *does* warn; only window close does
not.

**Why:** The window-close guard was never built, though it has been part of the
design since v0.3.0. Recorded against RFC-016.

**Owner:** RFC-069 for 0.18.0 (RFC-016 is where it was first specified).

**Workaround:** Save with Ctrl+S before closing. The saved file on disk is never
affected — only edits you have not yet committed are lost.

---

## Feature Limitations

These features are intentionally absent in this release:

| Feature | Status |
|---------|--------|
| Plugin / extension system | Deferred — see Future RFC-C |
| AI writing assistance | Deferred — see Future RFC-D |
| Real-time collaboration | Deferred — see Future RFC (not started) |
| Cloud synchronization | Out of scope by design |
| Mobile application | Out of scope by design |
| Web / browser version | Out of scope by design |
| Multi-document workspace | Deferred — RFC-057 |
| TUI / CLI companion | Deferred — see Future RFC-F |

---

## Platform Limitations

See `PLATFORMS.md` for the full platform support matrix. Key notes:

- **Unsigned macOS builds**: Gatekeeper will warn. Right-click → Open to bypass.
- **Linux Wayland**: Runs via XWayland; native Wayland support follows upstream.
- **File dialogs on headless Linux**: May silently return no file if no portal is available.
- **Linux: omriss may open your Markdown files without your having chosen it.**
  Installing omriss registers it as *an* application for Markdown (`.md`) files, in
  the "Open With" list. It never sets itself as the default, and it does not
  register for JSON at all. But a desktop where you have never chosen a Markdown
  editor uses the first registered one in alphabetical order, and omriss's name can
  sort before GNOME Text Editor or Kate, so `.md` files may open in omriss. A
  default you chose before installing is never changed. To choose another, use
  your desktop's Default Applications setting, or run
  `xdg-mime default <application>.desktop text/markdown` (for example
  `xdg-mime default org.kde.kate.desktop text/markdown`). `gio mime text/markdown`
  shows what is chosen now.
- **A save makes a new file, so some file properties are not carried over.** A file
  with several hard links becomes a separate file after you save it; extended
  attributes (macOS Finder tags, for example) are dropped; a file owned by someone
  else that you are allowed to write is yours afterwards. Permissions, and symbolic
  links, are kept on Linux and other Unix systems (the link stays a link and the file it points
  to is saved). On Windows permissions are the folder's, not copied. Why, and what
  each system guarantees after a power cut, is in *How omriss Treats Your Text*.
- **Linux screen readers: the section-actions button is announced as a "combo
  box".** The "…" button on a Document Map row opens a menu — Move up, Rename,
  Delete and the rest — and omriss says so to assistive technology
  (`aria-haspopup="menu"`). WebKitGTK, the web view omriss runs in on Linux,
  exposes *any* button that says it opens a popup as a combo box, so Orca reads
  "combo box" where it should say "menu button". Checked on WebKitGTK 2.54.1 with
  a plain page that has none of omriss's markup. The button still does what it
  says: it opens the menu, and the menu's items carry the reason when they are
  unavailable. omriss keeps the correct
  markup rather than hide that a menu opens; this is a mapping on the platform's
  side. Windows and macOS were not checked.

---

## Performance Limitations

- Documents larger than ~50 000 words may cause noticeable pause after
  committing an edit (re-indexing is synchronous and full).
- Very deep heading trees (>50 levels of nesting, which is unusual) are
  supported but not performance-optimised.
- JSON documents are re-parsed in full after each **committed** edit and each
  undo — never incrementally, and never while you are still typing (RFC-067
  §3.1: a draft is cached alongside the document's revision and does not
  trigger a re-parse). Measured on a release build, 374 KB JSON / 8000 array
  items: committing keeps costing a real parse (`focus()` ~2.6 ms), but
  reading structure repeatedly at an unchanged revision — what typing in the
  focused-content editor now does — dropped from ~21 ms to ~4.7 ms per
  keystroke, and the Document Map's own rebuild from ~18 ms to ~9 ms per
  render. At 4 MB (roughly 10x this fixture) those numbers scale
  proportionally with parse cost, so the committed-edit and undo/redo cases
  remain full re-parses and are not yet incremental.

---

## What Is Not Limited

- Your source file is **never silently modified** by omriss, in any supported
  format. If you open a file, navigate around, and close without saving, the
  file is unchanged.
- Committed edits rewrite **only the part you edited**. Formatting, ordering,
  indentation, and line endings everywhere else survive a save untouched — this
  holds for JSON exactly as it does for Markdown.
- All structural operations (move in/out, move up/down, add, delete, join)
  are **undoable** via Ctrl+Z.
- Saved files are **standard UTF-8** in their original format. They open
  correctly in any text editor, with no omriss-specific metadata.
