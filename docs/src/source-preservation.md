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
  whose lines end in a bare CR (classic Mac OS) stays that way;
- HTML comments and inline/block HTML;
- code fences (a `#` inside a fence is code, not a heading);
- YAML/TOML front matter;
- headings that share the same title (each keeps its own identity).

This is verified continuously by a golden test suite that edits **a section
in each fixture document** and asserts the surrounding bytes are untouched
and that undo restores the original file exactly.
