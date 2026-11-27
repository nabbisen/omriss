# RFC-077 — Task Breakdown and PR Plan

Related RFC: `../../proposed/077-search-covers-heading-text.md`

Lifecycle status: inherited from RFC-077.

## PR 1 — titles in search

- Titles in the per-revision cache, by RFC-077 §8.1's rule.
- A title match as a result kind: it opens its node; the result list shows
  "Heading" / "Key" in words.
- Order and scope per §8.3–§8.4.
- Tests per RFC-077 §6 and §8's validation; the measurement.
- The docs, the known-limitations entry and the CHANGELOG.
- GUI, `en_US`, once: search a Markdown heading's title, then a JSON key;
  each result shows its word and opens its node.
