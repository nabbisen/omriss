# RFC-070 — Task Breakdown and PR Plan

Related RFC: `../../done/070-split-section-input-validation.md`

Lifecycle status: inherited from RFC-070.

## PR 1 — split validation (RFC-070 §4.1, §8.1–§8.3)

- `validate_title`, shared.
- The body bound, the character boundary, the CRLF rule.
- Tests per RFC-070 §6 and §8.3. Remove the `#[ignore]` on
  `split_increases_count_by_one_and_preserves_titles`; it passes.
- Show the property bites: restore the `full_range` bound, and the property
  fails with a shrunk case. Quote it.

**PR 1 part 1 landed** (`d131b6e`): `validate_title`, the CRLF rule, the
character-boundary test.

## PR 1 part 2 — the offset rule (RFC-070 §9)

- The heading-range rule, as prototyped; the parameter rename and the
  variant's documentation; the CHANGELOG.
- Offset tests: inside an ATX child heading, between a setext title and its
  underline, past the section's end, all refused; after the children, legal.
- Remove the `#[ignore]`; show the property fails without the rule (quote
  the case, as you did).

## PR 2 — P4 (RFC-066 §3.1b, RFC-070 §8)

- The property for rename and join.
- Show it bites: revert one of RFC-065 §2.8's guards around `atx_title_range`
  (RFC-066 §3.1b measured P2 failing 0 of 10 with them reverted). P4 must catch
  it. Quote the shrunk case.
- If join's seam cannot be stated exactly, stop and report before writing an
  exemption.

**PR 1 part 2 landed** (`4e65200`); **PR 2 landed** (`daebed0`).

## PR 3 — rename replaces the whole setext title (RFC-070 §10)

- All title lines of a setext heading become the requested line; the underline
  and the rest unchanged, line endings included.
- P4's rename clause covers every heading; the exemption, its doc and its
  comment go.
- Tests: two- and three-line setext titles, in LF and CRLF; undo exact.
- CHANGELOG `Fixed`: renaming a heading whose title spans lines used to keep
  all but its first line.
