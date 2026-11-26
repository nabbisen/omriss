# RFC-084: Line Endings and Separators in Section Edits

**Project:** omriss — Omriss Editor
**Milestone:** M13 — defect (0.18.0, taken into the frozen scope by the owner on 2026-11-26 as a file-integrity exception)
**Status.** Implemented (0.18.0) — §4 (`5fa86b7`, `83fe573`, `1a89eb4`), §10 (`8a1cf23`, `485c331`, `bc36acb`), §11 (`f4bf739`); defects D1–D7. Accepted by the owner on 2026-11-26, which closes the Windows check of §12.
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

## 9. Found during implementation (2026-11-26)

1. **The JSON value field is a one-line `input`** showing the decoded string.
   A browser strips line breaks from such a field's value, so a JSON string
   that decodes to text containing a line break would lose it as soon as the
   user edits that field — the same class as D1. Not yet measured.
2. **A lone `\r` at the edge of a section body** (a heading ended by a bare CR)
   is not covered by the property; inside a body it is.

Both are measured by Task 028 before anything is decided.

## 10. Task 028's findings, and the design (2026-11-26)

Measured by Task 028 (`.git-exclude/review-request/028-json-value-newlines-and-lone-cr/`).

### Findings

| # | Defect | Measured |
|---|---|---|
| D4 | **A JSON string value loses its line breaks when edited.** The field is a one-line `input` showing the decoded text; a browser strips line breaks from it, so the first keystroke drops them, silently. A line break can be neither shown, kept nor typed | `"line1\nline2"` + `x` → `"line1line2x"`; `"a\r\nb"` → `"abx"` |
| D5 | **An edit re-spells escapes the user did not touch.** A commit re-encodes the whole string (`core/src/formats/json.rs:144`, `scalar::encode_string`), so any escape the encoder would not itself write changes form. `docs/src/file-formats.md` promises "every edit changes only the bytes you touched" | `"café"` + `x` → `"caféx"`; `"a\/b"` → `"a/bx"` |
| D6 | **A file whose lines end in a bare CR** (classic Mac OS): (1) a line break typed at the start of a section body fuses with the heading's CR into a CRLF — the heading's ending changes and the typed blank line vanishes; (2) every newline omriss writes into it is LF, because the newline rule knows only CRLF and LF (`raw_text::newline_of`, core's `document_newline`) | `"# A\rb\r"`, draft `"\nb\n"` → `"# A\r\nb\r"` |

Not defects: leaving any value untouched changes nothing (all six cases byte-
identical); tabs and literal non-ASCII survive edits; files mixing CR with LF or
CRLF pass the property (over 1,400 edits each).

### Design

*Taken into 0.18.0 in full — D4, D5 and D6 — by the owner, 2026-11-26.*

1. **One principle, one more layer: replace only what changed — also inside
   an encoded string.** A JSON string edit compares the draft with the
   decoded original, finds the changed span **in the decoded text**, maps that
   span back to the original literal's bytes, and replaces only those bytes
   with the encoded new text. Untouched escapes keep their spelling (D5). This
   is §4.1's rule — the same compare-and-splice — applied through the JSON
   string codec; the decoder gains offsets for that, in `omriss-core`.
2. **A string value is edited in a text area** (D4), so a line break can be
   shown, kept and typed. Its line breaks go through §4.1 like any text area:
   untouched ones keep their encoded form (`\n` or `\r\n`); a typed one is
   written in the style the string already uses, or `\n` if it has none.
   Numbers stay in a one-line field. The text area starts one line high and
   grows with its content, so a one-line value looks as it does today.
3. **Bare-CR files** (D6): the document's newline becomes **CR** when the file
   has bare CRs and neither CRLF nor LF — in core's `document_newline` and the
   UI's `newline_of`, which must stay one rule (one function, used by both). The
   section commit computes its edit against the document, not the body alone,
   so the lone-CR rule can see the heading's CR before the body.
4. **The property covers all of it:** bare-CR documents join the line-ending
   property; a JSON string property — every value × every span × the snippets —
   asserts that bytes outside the changed span of the literal are unchanged,
   escapes included, and undo restores the original.

`docs/src/file-formats.md`'s promise becomes true; the CHANGELOG entry gains a
sentence for JSON string values (edits in 0.17.0 dropped line breaks inside a
value and re-spelled escapes).

## 11. Decisions on PR 4–6 (2026-11-26)

1. **The lone-CR rule as built is accepted.** Where an edit would put a lone
   CR next to an LF, the two would read as one CRLF — one line break where the
   file had two. No byte sequence keeps both breaks without changing one of
   them, so the narrowest rewrite is the right one: only a lone CR at the edge
   of an edit, only in a file that mixes bare CRs with LFs. Files that are all
   CR, all LF, or contain CRLF never have a byte beside the edit rewritten —
   the property pins it. **The documentation says so**, so the promise stays
   true: `source-preservation.md` names this one exception.
2. **`omriss-core`'s public names must not name a browser widget.** Core is a
   published library; "text area" is the UI's concept. The function becomes a
   name for what it does — LF-normalised text — and its module a newline
   module; `omriss-ui` uses core's name, with no second alias.
3. **Line splitting in core, by construction.** The dev team found by reading
   that `split_first_line` splits on `\n` only. Three sites in core find line
   ends by `\n` alone (`delete_split_merge.rs`, `rename.rs`); every one on an
   edit path uses the same three-ending rule, each with a bare-CR test. If the
   search finds more than a handful, stop and report.

## 12. PR 7 landed (2026-11-26)

- Core's names: `normalize_to_lf` and `document_newline`, in the `newline`
  module; `omriss-ui` uses them with no alias. Neither name was ever released,
  so this is an addition to core's API, not a rename for anyone.
- `source-preservation.md` names §11.1's exception.
- The search over core found two line-splitting sites, both now
  `newline::split_first_line`. One was **a further defect, D7: renaming a
  section in a CRLF file turned the heading's ending into LF** — the same harm
  as D1, shipped in 0.17.0, now in the CHANGELOG entry. The other refused to
  join a setext heading in a bare-CR file. Rename, join, delete and split are
  tested in LF, CRLF and CR, with undo; the rename and join tests fail on the
  old code.

**The owner's Windows check, with a CRLF file:** (1) type at the end of a
middle section, Ctrl+S — one CRLF added, caret after the typed text; (2) rename
a heading; (3) edit a JSON text value containing `\n`. After each, the file has
no LF without a CR before it. A build of `main` from `f4bf739` or later.
