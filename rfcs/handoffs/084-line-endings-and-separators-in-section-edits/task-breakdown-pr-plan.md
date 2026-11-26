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
