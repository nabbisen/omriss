# File Formats

omriss works best with Markdown documents. It is also growing into a
**structured plain-text editor**: a tool for viewing and, where supported,
safely editing other plain-text formats using the same layer-by-layer
Document Map and focused editing model.

Support is introduced one format at a time, and each format only becomes
editable once it has passed its own preservation and safety checks. The
table below is the status that is true **today** — not a promise of what a
format will eventually do.

| Format | Status |
|---|---|
| Markdown | Fully supported |
| JSON | Supported (RFC-054) |
| TOML | Planned — not available yet (RFC-055) |
| YAML | Under investigation (RFC-056) |

Formats are being added in this order: Markdown first, then JSON, then
TOML, with YAML treated separately as a feasibility investigation because of
its complexity. A later format's arrival never changes how an earlier one
behaves.

## What opens as Markdown today

omriss's file picker accepts `.md`, `.markdown`, `.mdown`, and `.txt`, and
opens all four as fully editable Markdown documents. This is existing,
reliable behavior — not part of the JSON/TOML/YAML work above — and you can
rely on it continuing to work.

## What "Supported" means for JSON

Open a `.json` file and its keys, objects, arrays, and values appear in the
Document Map exactly like Markdown's headings do. Select a value to edit it
in place:

- text, numbers, and on/off (`true`/`false`) values, validated as you type;
- an object or array's raw text, via "Show this part as text" — the
  replacement must itself be valid JSON and keep the same shape (an object
  cannot become an array, or the reverse);
- `null` values are read-only in this version — changing what *kind* of
  value something is remains out of scope, not just changing its content.

Every edit changes only the bytes you touched: indentation, key order, and
line endings elsewhere in the file are untouched, and Undo restores the
exact original bytes. Only strict JSON (RFC 8259) opens as structured; a
file with comments or trailing commas opens with its exact source text
preserved and a plain-text fallback, not silently reinterpreted.

**Not yet supported:** adding, deleting, renaming, or reordering
keys/items. Editing changes what a value *is*, not the document's shape —
that is a larger, separate piece of work with its own preservation risks.

## Your file stays your file

Whatever the format, the plain-text file you open is always the source of
truth. omriss never rewrites parts of the file you did not touch, and it
never requires a hidden companion file to reopen your document correctly.
