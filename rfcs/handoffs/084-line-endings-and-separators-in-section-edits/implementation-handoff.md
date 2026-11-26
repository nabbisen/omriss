# RFC-084 — Implementation Handoff

Related RFC: `../../done/084-line-endings-and-separators-in-section-edits.md`

Lifecycle status: inherited from RFC-084.

---

## 1. Build what RFC-084 §4 says

The design is decided there; the choices left to you are local: the general
function's name and signature, and where its tests live. Two constraints:

- the **document's** newline decides inserted newlines — not the edited
  range's, which may contain none;
- `raw_text.rs`'s lone-`\r` rule (an inserted `\n` right after a lone `\r`)
  carries over unchanged.

Find every text area that commits by construction — grep for every
`event.value()` that reaches a draft — not from this list: section editor,
JSON group raw text, plain file text. If there is a fourth, say so.

## 2. Two harness notes, from your Task 027 and RFC-059 PR 3 reports

Record both in `scripts/gui/README.md`, where the next person looks:

- AT-SPI's text of a `textarea` **omits one trailing newline**. Read the
  page's value when a trailing newline matters.
- `xdotool type` did not register in `ja_JP` (`gui_type_into` timed out) while
  key-by-key typing worked. Find out why if it is quick; otherwise record the
  symptom and the workaround.

## 3. Constraints

- `omriss-core` changes only if `joining_separator` itself needs to — say so
  before changing it.
- No new dependency.
- Existing golden tests stay green unchanged; if one must change, that is a
  finding to report, not a fix to make.
