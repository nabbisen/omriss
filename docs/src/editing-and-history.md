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

## When an edit changes the document's structure

Section text is Markdown, so text you type can change which *other* sections are
headings. Writing `<!--` (or an unclosed `<?`, `<script>`, `<!DOCTYPE` or
`<![CDATA[`) turns every heading after it into comment text until the comment is
closed; writing `# Foo` creates a new section. Moving a section can do the same
when its text carries one of those openers.

Nothing is lost — your bytes are exactly as you wrote them — but a section you
did not touch may stop appearing in the Document Map. When that happens omriss
shows a notice above the status bar saying how many sections appeared or
disappeared, what usually causes it, and that **Ctrl+Z** undoes it. The notice
is informational: the edit has been applied and may be what you meant. Ordinary
edits, and operations whose whole purpose is to add or remove sections (adding,
deleting, renaming, joining), never show it. It clears on the next edit and on
undo or redo, and is only shown for Markdown files.

## Undo and redo

Undo restores the previous text **byte-exactly**, and redo re-applies the
edit byte-exactly. History is bounded (100 entries) and survives structural
changes: undoing an edit that introduced new headings also retracts those
sections from the Document Map.

## Unsaved changes

The status bar shows when the text differs from what is on disk. Undoing back
to the exact saved bytes clears the indicator — omriss compares content, not
edit counts.

## Files changed on disk

When you save an existing file, omriss checks whether the file on disk has a
newer modification time than the version it opened or last saved. If another
process changed the file, omriss shows a **File Changed on Disk** dialog before
writing. Choose **Overwrite** to replace the disk file with the current omriss
document, **Save As** to write to a different path, or **Cancel** to leave the
disk file untouched.

This check happens before saving, not continuously while you edit or navigate.
It is a practical protection against accidental overwrite, not a hard security
boundary against every filesystem race.
