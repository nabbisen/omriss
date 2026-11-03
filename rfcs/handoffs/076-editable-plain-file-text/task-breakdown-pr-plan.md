# RFC-076 — Task Breakdown and PR Plan

Related RFC: `../../proposed/076-editable-plain-file-text.md`

Lifecycle status: inherited from RFC-076.

Four PRs. The first belongs to RFC-069 and this item cannot safely start
without it.

---

## PR 1 — RFC-069 §2.1: resync the draft after commit

**Credited to RFC-069**, recorded in its Status. Handoff §2.2 says why it comes
first: without it, the navigation this feature performs after every raw commit
re-commits an identical body and pushes an undo record, and undo is the whole
of this feature's safety.

One call — `draft_sync::sync` — at `handle_save`, `open_search_if_available`
and `AppCommand::TogglePreview`.

**Checks:** gates. Plus the three symptoms, each as a test or a recorded
observation: the dirty dot clears after save; a navigation after a commit
pushes **no** new undo record; Open right after a save does not warn.

That middle one is the one this item depends on. Prove it.

## PR 2 — The commit path, in `omriss-ui`

`replace_range` over the whole source, with the revision check, routed through
RFC-071's detector.

- Decide and state how `expected_change` expresses "a raw commit may change
  anything" (handoff §3.2).
- Focus returns to the outline (§3.1).
- Unit tests in `omriss-ui` — no WebView.

**Checks:** gates; RFC-071's coverage test passes **without modification**;
the new call site appears in its enumeration.

Cases at minimum:

| Case | Expect |
|---|---|
| raw commit that changes nothing | no history entry, no warning |
| raw commit that rewrites one body | one undo step, no structure warning |
| raw commit that removes every heading | one undo step, **warning**, count correct |
| raw commit that adds headings | one undo step, **warning** |
| undo after any of the above | source byte-identical to before |
| raw commit against a stale revision | refused, source unchanged |

## PR 3 — The view, in `omriss-components`

Editable, with §3.4's read-only path intact, and §3.3's gate on entering and
leaving.

**Checks:** gates, plus one GUI pass under the GUI rules:

- edit the raw text, commit, and confirm the file on disk matches byte for
  byte;
- Ctrl+Z once returns the whole document;
- the structure warning appears when headings vanish;
- "Show plain file text" on a malformed `.json` still works and is still
  read-only-reachable (handoff §5).

## PR 4 — Documentation

- `docs/src/known-limitations.md`: **remove** "Plain File Text Editing Is
  Read-Only" rather than rewording it (RFC-076 acceptance criterion 5).
- `CHANGELOG.md` under **Added**.
- `docs/src/editing-and-history.md` wherever it describes the view.
- RFC-076's Status.

---

## Ordering

Strictly sequential. PR 1 is a prerequisite, not a convenience.
