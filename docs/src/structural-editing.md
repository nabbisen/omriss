# Structural Editing

omriss lets you reorganise your document's heading hierarchy without touching
the body text. Structure actions live in the **Document Map**, not in the
Writing Area. Use `+ Top` to create a new top-level section. Select a
section row to use the same creation strip for `+ Inside` and `+ After`, or
use its `...` actions to rename, move, join, or delete sections.

---

## Move Out and Move Inside

**Move out one level** raises the heading level by one step (for example H3
to H2), making the section a higher-level concept in the hierarchy. **Move
inside previous section** lowers it (H2 to H3). The selected section's
subtree moves with it, so child sections remain children of the moved section.

Buttons are disabled when the operation would be invalid:

- You cannot move an H1 heading farther out; it is already at the top level.
- You cannot move an H6 heading deeper.
- Setext-style headings (`Title\n======`) are not supported for these actions;
  use the plain file text view to inspect them before converting them to ATX
  headings in an external editor.

When moving a non-last child out one level, omriss places the moved section
after the parent section's remaining children. This keeps following siblings
under their original parent instead of accidentally making them children of
the moved section.

---

## Move Up / Move Down

These swap the focused section with the adjacent sibling in source order,
keeping the full subtree (all children and descendants) intact. The buttons
are disabled when the section is already at the top or bottom of its siblings.

---

## Merge Into Previous Section Content

**Merge into previous section content** removes the focused section's heading
marker and keeps its heading text and body text as part of the previous
sibling's content. The focused section is no longer a section after this
operation. When safe, its child sections become children of the previous
sibling.

This action is disabled when the previous sibling already has subsections. In
that shape, removing the focused heading could make the joined text belong to
the previous sibling's last child instead of the previous sibling itself.

This operation cannot be undone by merging again — use **Ctrl+Z**
(Undo) to restore the heading.

---

## Add section

The Document Map's `+ Top` button opens a dialog where you enter a title. It
always creates a new top-level section.

After you select a section row, `+ Inside` creates a child heading at the end
of the selected section at the next depth level. `+ After` creates a sibling
after the selected section and its child sections. You can then move or
redistribute body text by editing normally.

---

## Rename

Rename changes only the selected section's heading text. The body text, child
sections, sibling order, and surrounding file text are preserved.

---

## Delete Section

Opens a confirmation dialog showing the section title and the number of child
sections that will also be removed. Click **Delete** to proceed or **Cancel**
to abort. The deletion is reversible with **Ctrl+Z** (Undo).

---

## Undo

All structural operations are recorded in the same undo history as body
edits. **Ctrl+Z** reverses the most recent committed operation; **Ctrl+Y**
re-applies it. Structural edits are byte-exact: undo restores the source
character-for-character.
