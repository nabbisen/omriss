# Known Limitations

This page documents the intentional limitations and deferred features in the
current version of omriss. Limitations that affect data safety are listed
first.

---

## Data Safety Limitations

### Plain File Text Editing Is Read-Only

The plain file text view (Ctrl+\`) shows the full document source but
does not currently support editing. Changes must be made through the Writing
Area or an external text editor.

**Why:** Editing raw source requires re-indexing the entire document after
every commit, and handling the case where heading titles are changed during
a raw edit requires additional focus-remapping logic. This is deferred to a
future release.

**Workaround:** Open the file in a standard text editor (Vim, VS Code, etc.),
make your changes, then re-open the file in omriss.

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

**Workaround:** Use the "Show this part as text" action on a group or list to
replace that whole part with JSON you write yourself, or edit the file in a
standard text editor.

### Null Values Are Read-Only

A value that is `null` is displayed but cannot be edited.

**Why:** Changing `null` to a string, number, or object is a change of type,
which is a structure change rather than a value edit — the same reason as above.

**Workaround:** Use the containing group's "Show this part as text" action.

### Strict JSON Only

Comments (`//`, `/* */`) and trailing commas are not accepted. A file using them
is reported as invalid — this includes most `tsconfig.json`-style and VS Code
configuration files, which are JSONC rather than JSON.

**Why:** JSONC is a different format with no single agreed specification.
Accepting it silently would mean guessing which dialect you meant, and guessing
wrong writes damage into your file.

**Workaround:** The plain file text view still opens, so the source is readable
and nothing is lost; edit the file in a standard text editor.

### Duplicate Keys Are Kept, Not Merged

If an object contains the same key twice, both appear in the Document Map and
each is edited independently. omriss will not drop one or combine them.

**Why:** Silently discarding a key would be a data loss, and which duplicate
"wins" varies between JSON parsers. Preserving what you actually wrote is the
safer answer even though the file is unusual.

### Other Structured Formats

TOML and YAML are not available yet; a `.toml` or `.yaml` file opens as plain
text. See [File Formats](./file-formats.md) for the current support table.

---

## Preview Limitations

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

**Workaround:** Use the plain file text view to inspect the heading, then
convert it to an ATX heading in an external editor before using the structure
actions normally.

---

## Navigation Limitations

### Quick Actions Reverse Tab Traversal Is Deferred

Quick Actions opens with Ctrl+P and supports search, arrow-key command
selection, Enter execution, and Escape/Ctrl+P dismissal. Reverse Tab traversal
inside the Quick Actions panel is a known keyboard focus limitation in this
release.

**Why:** Repeated fixes around native Shift+Tab traversal and focus trapping
were brittle in the Dioxus WebView event stack. The remaining focus model work
is tracked as an RFC-059 keyboard follow-up.

**Workaround:** Use the search field and arrow keys to choose a command, then
press Enter. Use Escape or Ctrl+P to dismiss Quick Actions.

### Keyboard Shortcuts Stop Working After a View Change (WebView constraint)

Keyboard shortcuts are delivered to the application window. When an action
changes the view — zooming into a section, for example — the element that had
keyboard focus is replaced, and focus falls back to the document body, outside
the window's shortcut handler. Shortcuts pressed from that state do nothing
until focus returns.

The most visible case is **Esc to zoom out**: after zooming in with Enter, Esc
often does not zoom back out, because focus was lost during the zoom itself.
Ctrl+` for plain file text, and typing straight into the Search panel after
Ctrl+F, can be affected the same way.

**Why:** Restoring focus after a view change needs programmatic
`element.focus()` calls in the Dioxus WebView environment, which are not
implemented in this release. The shortcuts themselves are correctly bound; they
are simply not reached.

**Workaround:** Click once anywhere in the window, or press Tab, to put focus
back inside it — shortcuts then work normally. Every affected action also has an
on-screen control: use the "Document" breadcrumb to zoom out, the toolbar button
for plain file text, and click into the search field before typing.

### Closing the Window Does Not Warn About Unsaved Changes

Closing the window while an edit is unsaved discards that edit without asking.
Opening another file or starting a new one *does* warn; only window close does
not.

**Why:** The window-close guard was never built, though it has been part of the
design since v0.3.0. Recorded against RFC-016.

**Workaround:** Save with Ctrl+S before closing. The saved file on disk is never
affected — only edits you have not yet committed are lost.

### Search Does Not Match Heading Text

Searching finds text inside section bodies, but not the text of the headings
themselves. Searching for a heading title you can see in the Document Map may
return "No results".

**Why:** The search index covers section body ranges and excludes the heading
line.

**Workaround:** Use the Document Map to navigate to a heading by name, and use
search for body content.

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
| Multi-document workspace | Deferred — see Future RFC-B |
| TUI / CLI companion | Deferred — see Future RFC-F |

---

## Platform Limitations

See `PLATFORMS.md` for the full platform support matrix. Key notes:

- **Unsigned macOS builds**: Gatekeeper will warn. Right-click → Open to bypass.
- **Linux Wayland**: Runs via XWayland; native Wayland support follows upstream.
- **File dialogs on headless Linux**: May silently return no file if no portal is available.

---

## Performance Limitations

- Documents larger than ~50 000 words may cause noticeable pause after
  committing an edit (re-indexing is synchronous and full).
- Very deep heading trees (>50 levels of nesting, which is unusual) are
  supported but not performance-optimised.
- JSON documents are re-parsed in full after each committed edit and each undo,
  rather than incrementally. This has not been measured against a large file;
  correctness was prioritised over parse cost in this release.

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
