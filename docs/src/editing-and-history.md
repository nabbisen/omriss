# Editing and History

## Applying a section text edit

Refine the focused section's text in the Writing Area. omriss applies the
draft when you save, preview, change focus, or leave the editor. Each applied
edit carries the document revision your draft was based on. If the document
changed underneath (for example, an undo elsewhere), the stale edit is
rejected before anything mutates, and your text stays in the editor.

## Structural operations

Structure operations live in the Document Map. Use a section row's actions to
move up, move down, move inside the previous section, move out one level, join
with the section above, add a section, or delete. These reorganise the heading
hierarchy without touching unrelated section text. Every structural operation
is recorded in the same undo history as text edits — see
[Structural Editing](structural-editing.md) for details.

## Undo and redo

Undo restores the previous text **byte-exactly**, and redo re-applies the
edit byte-exactly. History is bounded (100 entries) and survives structural
changes: undoing an edit that introduced new headings also retracts those
sections from the Document Map.

## Unsaved changes

The status bar shows when the text differs from what is on disk. Undoing back
to the exact saved bytes clears the indicator — omriss compares content, not
edit counts.
