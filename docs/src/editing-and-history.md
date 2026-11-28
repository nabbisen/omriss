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

## Editing the plain file text

**Ctrl+\`** (or the **Show plain file text** button) opens the whole file as
text. It opens **read-only**: you can look at the bytes without being one
keystroke from changing them. Choose **Edit** to type in it; **Stop editing**
returns to the read-only view.

Your changes apply as **one edit** — one undo step — when you save, leave the
view, choose **Stop editing**, or click away from the text. Until then the
status bar shows the file as having unsaved changes, and nothing in the document
has changed. **Ctrl+Z** takes the whole edit back byte-exactly. Only what you
actually changed is replaced, and a file you open, view and save without typing
is written byte-identical (line endings included).

Afterwards focus returns to the **whole-document outline**. omriss does not
guess which section you were in: a rewrite can remove or rename the heading you
were editing, and being dropped into *a* section without being able to tell
whether it is the same one would mislead you. If the edit changed which
sections are headings, the structure notice above appears, with the counts.

A pending section text edit is committed before the view opens, and a pending
file-text edit is committed before it closes. If the document changed under an
open editor, the edit is refused and the text is reloaded rather than merged.

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

When you save an existing file, omriss reads the file again and compares what is in it
with what omriss last read from it or wrote to it. If another program changed it — or
removed it, or it can no longer be read — omriss shows a **File Changed on Disk**
dialog before writing anything. It compares the file's *contents*, not its
modification time, so a change is found even when a tool leaves the old time on new
bytes (a `git checkout`, `rsync --times`, `touch -d`), and a program that rewrites the
file with the same bytes is not a change at all.

The dialog offers four choices, and focus starts on **Cancel**, which changes nothing:

- **Overwrite** replaces the file with your document, as the save you asked for.
- **Reload from disk** replaces your document with the file as it is now. It is one
  undo step: **Ctrl+Z** brings your unsaved text back exactly. And because that text was
  made from the *old* file, saving it after the undo asks again, so the other
  program's changes are never overwritten without you being asked. Redo returns to the
  reloaded text.
- **Save As** writes your document to a different path.
- **Cancel** leaves the file and your document untouched.

If the file was removed, or is no longer UTF-8 text, Reload cannot read it: omriss says
why, as when opening it, and your document is left as it was.

This check happens before saving, not continuously while you edit or navigate. Between
the check and the write another program can still change the file; omriss narrows that
to the moment, it cannot close it.
