# RFC-073 — Task Breakdown and PR Plan

Related RFC: `../../done/073-complete-localization-and-locale-persistence.md`

Lifecycle status: inherited from RFC-073.

In order.

## PR 1 — typed open and save failures (§8.1–§8.3)

- `OpenFailure` per §8.1, carried by `OpenOutcome::Failed` and
  `Modal::OpenError`; the dialog renders it from the catalog.
- `SaveOutcome::Failed` with its reason; the status message says why.
- The command-line message (`app.rs`) as a variant.
- Tests: the exhaustive maps; level 2 per open variant in both locales; the
  "no English cause" command.
- GUI, `en_US` and `ja_JP`: a `chmod 000` file. The dialog's text, and its
  accessible description via AT-SPI, contain no OS text.

## PR 2 — dialog filter labels, `from_tag` (§8.4–§8.5)

Small; can travel with PR 1 if that is cleaner.

## PR 3 — JSON list labels (§8.6)

- Core's positional title is the number; one display-title function in the UI,
  used by every place that shows a title.
- Tests: Japanese labels in the map, cards, breadcrumb, heading and search path;
  "Item" and the numbers still not searchable (RFC-077's tests unmodified).

**PR 1–3 landed** (`ac4f75b`, `7ec5c93`, `420f4bb`).

## PR 4 — the open-failure dialog (RFC-073 §9)

The cause as body text; the title without advice; "Close" from its own key;
both locales. Level 2 for the three; one GUI screenshot per locale of the
`chmod 000` case.
