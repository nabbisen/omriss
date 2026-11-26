# RFC-085 — Implementation Handoff

Related RFC: `../../proposed/085-format-aware-panes.md`

Lifecycle status: inherited from RFC-085.

---

## 1. Build what RFC-085 §3 says

The design is decided there. Local choices are yours: names, where the
structure-reading helpers live in the session, how the guard's list of allowed
places is written.

The findings table (RFC-085 §2) came from a read of the code, not a run. If a
row is wrong when you test it, say so in the report. Do not build around it.

## 2. Constraints

1. `omriss-core` does not change. If something needs it (a node field the
   structure lacks), stop and report.
2. No new crate dependency; `omriss-components` still builds for
   `wasm32-unknown-unknown`.
3. **Markdown behaviour is unchanged.** Existing Markdown tests pass
   unmodified; a changed expectation means the change is wrong, or must be
   reported.
4. Every new or changed message in both locales, through the catalog.
   English wording is in RFC-085 §3.4; write Japanese with the same meaning,
   not word for word.
5. The guard (RFC-085 §3.1) scans `crates/ui/src`, `crates/components/src`
   and `crates/app/src`, non-test code. Each allowed place carries a one-line
   reason in the list. Show it bites.
6. `docs/src/`: every page that describes JSON opening, the overview, search or
   the empty Document Map changes in the PR that changes the behaviour. So does
   any `known-limitations.md` entry the work closes.
7. Dead code the survey found (`navigate_parent`, `navigate_first_child`,
   `navigate_prev_sibling`, `navigate_next_sibling`, `outline_nodes`: no
   non-test callers) is deleted, not migrated. Confirm there are no callers first.

## 3. Coordination

RFC-077 (search covers heading text) is next and builds on PR 2's search. Do
not add title matches here. Do leave search shaped so a title match is one more
result kind (RFC-077 §4.2), not a second search.
