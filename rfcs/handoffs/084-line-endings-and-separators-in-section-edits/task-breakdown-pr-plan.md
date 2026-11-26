# RFC-084 — Task Breakdown and PR Plan

Related RFC: `../../proposed/084-line-endings-and-separators-in-section-edits.md`

Lifecycle status: inherited from RFC-084.

---

## PR 1 — line endings (D1), the property, the guard

The general newline-preserving function; every text area through it;
`is_pending` in the text-area view. The property and the source guard (RFC-084
§4.5). Show the guard bites, and show the property fails on the code before
this PR (run it against `HEAD~1`'s session path, or describe exactly why it
cannot be).

## PR 2 — one separator (D2)

Delete the session's append; core's `joining_separator` decides. Task 027's
cases 1–5 as session tests, byte-exact, LF and CRLF.

## PR 3 — the caret (D3), docs

Selection restored after a commit's resync, through `focus.rs`. CHANGELOG
`Fixed` per RFC-084 §5. GUI check on Linux; then the owner's Windows check.

**PR 1–3 landed** (`5fa86b7`, `83fe573`, `1a89eb4`).

## PR 4 — JSON strings keep their escapes (§10.1, D5)

The decoder with offsets in `omriss-core`; the span-only splice through the
codec; the JSON string property. Show the property fails on the old path.
`file-formats.md`'s promise is then true — say so in the report with the
property as evidence.

## PR 5 — a string value is a text area (§10.2, D4)

The field for string values becomes a growing text area through §4.1's path;
numbers keep the one-line field. Level-2 test for which field each kind gets;
GUI: `"line1\nline2"` shows two lines, keeps them on an edit, and a typed line
break lands as `\n`. Task 028's six values, byte-exact.

## PR 6 — bare-CR files (§10.3, D6)

One newline rule for core and UI, CR included; the section commit computed
against the document. Bare-CR documents in the line-ending property (they are
reverted from Task 028 — bring them back for good). Task 028's minimal
example becomes a test.

CHANGELOG: the RFC-084 entry gains the JSON sentence (§10, last paragraph).

**PR 4–6 landed** (`8a1cf23`, `485c331`, `bc36acb`).

## PR 7 — names, the documented exception, line splitting in core (§11)

1. Rename core's `as_shown_in_a_text_area` and the `text_area` module to names
   that describe LF-normalised text and newlines; `omriss-ui` uses core's
   name directly (no alias). Public API of `omriss-core` changes — say so in
   the CHANGELOG's crate notes if it has them.
2. `docs/src/source-preservation.md`: one plain sentence for §11.1's exception.
3. Every site in `omriss-core` that finds a line end — search by construction
   (`find('\n')`, `split('\n')`, `.lines()`, byte scans for `b'\n'`), not from
   the three already known. Each on an edit path: the three-ending rule and a
   bare-CR test (join, delete, split, rename at least). More than a handful:
   stop and report.

**Landed** (`f4bf739`). The search found two sites; one was D7 (RFC-084 §12).
