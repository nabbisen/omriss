# RFC-084: Line Endings and Separators in Section Edits

**Project:** omriss — Omriss Editor
**Milestone:** M13 — defect (0.18.0, pending the owner's scope decision)
**Status.** Proposed
**Document type:** Defect design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-065, RFC-076
**Related RFCs:** RFC-054, RFC-066, RFC-069, RFC-080

---

## 1. Summary

Editing a section's text **rewrites line endings the user did not touch**,
and inserts a separator in the wrong newline. Both break the promise
`docs/src/source-preservation.md` makes in its first list: "CRLF and LF line
endings, exactly as found".

Found from the owner's Windows report (2026-11-26: Ctrl+S at the end of a
section's text added a line break and moved the caret) and measured by the dev
team's Task 027 on Linux.

## 2. Findings

| # | Defect | Measured |
|---|---|---|
| D1 | **In a CRLF file, editing a section converts that section's whole text to LF** — lines the user never touched. The file ends with mixed line endings. Present since the section editor existed; shipped in 0.17.0 | `body two\r\n\r\n` became `body two\n\n` after typing one character at the end |
| D2 | **A separator in LF only.** Typing at the end of a non-last section, the next heading is kept on its own line by an appended `\n` — always LF, also in a CRLF file | one `\n` after the typed text, before `# Three` |
| D3 | **The caret moves past the separator** after the commit, so the next keystroke goes on a new line the user did not make | caret 11 → 12 |

Not defects, confirmed by the same measurement: saving without typing leaves
the file byte-identical, in LF and CRLF; repeated saves do not accumulate; the
last section and a file without a final newline gain nothing.

## 3. Causes

- **D1:** a browser text area always delivers `\n` (`\r\n` and a lone `\r` are
  normalised before the application sees them). The section editor stores
  that value as the draft and commits it as the new body
  (`focused_content_pane.rs` `oninput`, `session.rs` `commit_focused_body`),
  so the normalised endings replace the original ones. **The plain-file-text
  editor solved exactly this in RFC-076** (`crates/ui/src/editor/raw_text.rs`:
  compare in the text-area view, replace only the changed span, write inserted
  newlines in the document's style). The section editor never used it.
- **D2:** `session.rs` `body_for_focused_editor_commit` (lines 396–408) appends
  a literal `'\n'` to a non-last body that does not end with one — a second,
  LF-only copy of the rule core's `joining_separator` already implements with
  the document's newline (RFC-065, "one helper, not local patches").
- **D3:** `draft_sync::sync` replaces the editor's value with the committed
  body; replacing a text area's value puts the caret at the end.

## 4. Design

1. **One newline-preserving edit for every text area.** A commit from any text
   area — the section editor, the JSON group's raw-text editor, the
   plain-file-text editor — goes through one function that compares the draft
   with the original **as the text area shows it**, replaces only the changed
   span of the original bytes, and writes inserted newlines in the
   **document's** style. That is `raw_text.rs`'s mechanism, generalised from
   the whole document to a byte range; the plain-file-text editor keeps using
   it, now through the general form.
2. **"Pending" means "the user changed something".** Whether a draft is
   pending is decided in the same text-area view (`draft_sync::is_pending`),
   so a CRLF section the user has not changed is never pending and never
   committed.
3. **One separator rule.** The session's LF-only append is deleted; core's
   `joining_separator` — newline-aware, and already the rule RFC-065 chose —
   is the only place that keeps the next heading on its own line.
4. **The separator is real, so the editor shows it; the caret stays where the
   user left it.** After a commit, the editor's value is resynchronised and
   the selection is restored to the same place in the user's text (RFC-080
   §5.4 rule 9). The restoring belongs to `focus.rs`, the one place that moves
   focus or selection (RFC-059 §12.1).
5. **A guard against a fourth copy:** no code outside the newline-preserving
   function turns text-area text into source text — a source-scanning test in
   the style of the commit-path guard, and a property (RFC-066): for any
   document with LF, CRLF or mixed endings and any edit given as a text area
   would deliver it, every byte outside the changed span is unchanged, every
   inserted newline is the document's, and undo restores the original exactly.

## 5. Telling users

Users who edited CRLF files with 0.17.0 or earlier may have files with mixed
line endings. The CHANGELOG says so plainly under `Fixed`, with what it means
and that the file's text is otherwise intact — not hidden as "line ending
handling improved" (the owner's rule: tell the user when something surprising
happened).

## 6. Non-goals

- Converting existing mixed files back. omriss does not rewrite what the user
  did not edit — including endings an earlier version mixed.
- A line-ending setting.

## 7. Validation

- The property in §4.5, in the default `cargo test`.
- Session-level tests with LF-normalised drafts (as a text area delivers) on
  the CRLF fixture: Task 027's cases 1–5, each byte-exact.
- The JSON group raw-text editor on a CRLF JSON file: an edit keeps every
  untouched ending.
- GUI (`scripts/gui/`): typing at the end of a middle section, Ctrl+S — the
  file gains the document's newline once, and the caret stays after the typed
  text. Owner on Windows: the same with a CRLF file.

## 8. Acceptance criteria

- Editing a section in a CRLF file changes only the bytes of the edit; every
  newline it inserts is CRLF.
- No code adds a separator outside `joining_separator`.
- The caret does not move on commit.
- `docs/src/source-preservation.md`'s line-ending promise is true, and the
  CHANGELOG tells affected users.
