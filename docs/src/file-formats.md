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
| JSON | Planned — not available yet (RFC-054) |
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

## Your file stays your file

Whatever the format, the plain-text file you open is always the source of
truth. omriss never rewrites parts of the file you did not touch, and it
never requires a hidden companion file to reopen your document correctly.
