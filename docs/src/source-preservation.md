# How omriss Treats Your Text

The raw Markdown text **is** the document. The outline you navigate is a
derived index, rebuilt after each committed edit — it is never saved, and
the document is never regenerated from it.

When you commit a body edit, omriss splices exactly that byte range of the
file. Everything else is preserved bit-for-bit:

- spacing quirks, blank-line conventions, missing trailing newlines;
- CRLF and LF line endings, exactly as found — including the lines of the
  section you are editing that you did not change; a line break you type or that
  omriss must add (such as the one that keeps the next heading on its own line)
  is written in the file's own style, so a CRLF file stays CRLF — and a file
  whose lines end in a bare CR (classic Mac OS) stays that way. The one
  exception: in a file that mixes bare CRs with LFs, a bare CR right next to the
  text you edit can be rewritten as the file's newline, because a bare CR followed
  by an LF would otherwise read as one CRLF and a line break would be lost;
- HTML comments and inline/block HTML;
- code fences (a `#` inside a fence is code, not a heading);
- YAML/TOML front matter;
- headings that share the same title (each keeps its own identity).

This is verified continuously by a golden test suite that edits **a section
in each fixture document** and asserts the surrounding bytes are untouched
and that undo restores the original file exactly.

## How a save reaches your disk

Saving writes the complete new file beside the old one, makes sure it has reached
the storage, and only then replaces the old file with it in a single step. At every
moment your file is therefore either exactly as it was or exactly as you saved it,
never half of each. If a save fails for any reason — a full disk, a folder you may
not write to, a crash — the old file is untouched and no leftover file stays behind.
The new file is written under a hidden temporary name in the same folder
(`.notes.md.omriss-….tmp`), which exists only while the save is running.

If the final step, making the replacement itself durable, is the one that fails (on
Linux, the flush of the folder), the file **is** saved, and omriss says so: *"Saved,
but the disk did not confirm it. If this keeps happening, check the disk."* It never
reports a save as failed when the new text is on disk.

The same holds for omriss's own settings file, so a failed write can no longer
leave it truncated and reset your preferences and recent files.

What you can rely on, by system:

- **Linux, and other Unix systems.** (macOS runs the same code, but the project has
  not run it there.) The folder is flushed as well, so the replacement itself
  survives a power cut: afterwards you have the old file or the new one, whole.
  The file keeps its permissions, and while it is being saved its text is never
  readable by anyone who could not read the file before. **A symbolic link is
  followed**: the file it points to is saved and the link stays a link, so a vault,
  a dotfiles repository or a synced folder keeps receiving your edits. Saving to a
  link whose target does not exist yet creates the target.
- **Windows.** The file is flushed before the replacement, and the replacement is a
  single step, so a file is never half-written. Windows has no call to flush the
  folder entry, so after a power cut in the moments just after a save, you may find
  the previous version (whole, not damaged) instead of the new one. Permissions are
  those the folder gives a new file; omriss does not copy them. Symbolic links are
  followed as on Linux.

What a save does **not** carry over, on any system: a file with several hard links
becomes a separate file (an atomic replacement makes a new file; keeping the link
would mean writing over the old bytes in place, which is the risk this method
removes), extended attributes (macOS Finder tags, for example) are not copied, and
a file that belongs to another user, if you are allowed to write it, belongs to you
after the save. Whatever the drive itself reports as flushed is what is relied on;
a drive that misreports that cannot be corrected from here.
