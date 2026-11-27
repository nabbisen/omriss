# RFC-070 — Task Breakdown and PR Plan

Related RFC: `../../proposed/070-split-section-input-validation.md`

Lifecycle status: inherited from RFC-070.

## PR 1 — split validation (RFC-070 §4.1, §8.1–§8.3)

- `validate_title`, shared.
- The body bound, the character boundary, the CRLF rule.
- Tests per RFC-070 §6 and §8.3. Remove the `#[ignore]` on
  `split_increases_count_by_one_and_preserves_titles`; it passes.
- Show the property bites: restore the `full_range` bound, and the property
  fails with a shrunk case. Quote it.

## PR 2 — P4 (RFC-066 §3.1b, RFC-070 §8)

- The property for rename and join.
- Show it bites: revert one of RFC-065 §2.8's guards around `atx_title_range`
  (RFC-066 §3.1b measured P2 failing 0 of 10 with them reverted). P4 must catch
  it. Quote the shrunk case.
- If join's seam cannot be stated exactly, stop and report before writing an
  exemption.
