# RFC-077 — Implementation Handoff

Related RFC: `../../proposed/077-search-covers-heading-text.md`

Lifecycle status: inherited from RFC-077.

---

## 1. Build what RFC-077 §4 and §8 say

Search already has one walk (`search_texts`) and a per-revision cache of each
node's own text (RFC-085 PR 6). Titles join that cache and that walk as one more
result kind. No second search.

## 2. Constraints

1. `omriss-core`: only the doc comment of RFC-077 §8.1 changes. Anything else
   that needs core: stop and report.
2. Markdown body results do not change: same matches, same order among
   themselves, same targets. Existing search tests pass unmodified.
3. The title rule (§8.1) is one function, not a check at each call site.
4. Both locales for the result-kind words, through the catalog.
5. `docs/src/known-limitations.md`: the entry "Search Does Not Match Heading
   Text" is deleted in this PR. Every docs page that describes search says
   what search now covers. CHANGELOG `Fixed`.
6. No new crate dependency; `omriss-components` still builds for
   `wasm32-unknown-unknown`.
