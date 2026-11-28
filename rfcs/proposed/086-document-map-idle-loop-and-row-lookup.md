# RFC-086: The Document Map's Idle Render Loop and Row Lookup

**Project:** omriss — Omriss Editor
**Milestone:** M13 — defect (0.18.0, by the owner's decision of 2026-11-28: "if the fix is small it goes into 0.18.0")
**Status.** Proposed
**Document type:** Defect design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-049, RFC-085
**Related RFCs:** RFC-067

---

## 1. Summary

The Document Map **never stops re-rendering**, even when nobody touches
the app. With a large file open, omriss uses a whole CPU core while idle, and
every click waits behind the render already in progress. Separately, each
render finds every row's node by searching from the root, which is quadratic in
the number of rows.

Measured by Task 029 (`.git-exclude/review-request/029-document-map-large/`),
release build, WebKitGTK 2.54.1.

## 2. Findings

| # | Defect | Measured |
|---|---|---|
| L1 | **An effect that re-runs itself forever.** The map's effect reads `item_tree` (`is_expanded`) and then writes it, so each run queues the next. It is present since `321144d` (2026-08-02), so 0.17.0 shipped it. | idle, 10 s: 96 % of a core on an 8,000-item JSON file; 1.5 % on a 12-section Markdown file. Changing the two reads to `peek()` brings both to 0 % |
| L2 | **Every row's node is found by a search from the root** (`find_node`, per visible row) | 89 % of a 270–355 ms render at 8,000 rows; an index built once per render brings it to 16–20 ms |
| L3 | The whole item tree is rebuilt on a focus change, though only the selection changed | 12–20 ms per run |

With L1 and L2 fixed, a row click on 8,000 items goes from 0.74–1.5 s to
0.16–0.17 s, mouse-down to paint. On a focus change the DOM changes 2
attributes, before and after: rows are keyed correctly.

## 3. Design

1. **An effect never subscribes to a signal it writes.** In the map's effect,
   `item_tree` is read with `peek()`. The comment above the effect, which
   claims this already ("`item_tree` … not deps of this effect"), becomes true.
2. **A guard against the whole class, not this one instance: a settled app
   does no work.** A level-2 test mounts each pane, Document Map included, on
   Markdown and on JSON. Once it settles, it asserts that the `VirtualDom` has
   no further work: no render and no effect without input. This catches any
   effect in any component that re-triggers itself. Search every `use_effect`
   in the application crates for a read-then-write of the same signal, and
   report each one found and what was done.
3. **Rows are found through an index**, built once per render (or once per
   tree build) by one walk, keyed by row id. No per-row search from the root.
4. **The item tree is rebuilt only when the structure changes:** a new
   revision, a new format, or a view change that alters which tree is shown.
   A focus change only updates the selection (`on_selected`) and expands the
   ancestors.

## 4. Telling users

CHANGELOG, `Fixed`: omriss 0.17.0 and earlier kept the processor busy while
idle. It was a little with a small file and a whole core with a large one,
and on a large file clicks in the Document Map were slow. Laptop users will
notice the battery.

## 5. Non-goals: 0.19.0, with a design of its own

- **Two renders and a full 8,000-row diff per click.** About 100 ms of Rust
  remains for 2 changed rows. Making that cost what the changed rows cost needs
  rows that Dioxus can skip, which means restructuring the map's render.
- **The overview's 8,000 cards** (RFC-085 P5): 0.5–1.3 s, mostly WebKit
  building ~48,000 elements, and leaving the overview costs again.

Together these are **large structures in the Document Map and the overview**,
a 0.19.0 RFC to be written. This RFC only removes the defects.

## 6. Validation

- §3.2's settle test, with a deliberate break: put the `read()` back, and the
  test fails.
- The map's existing tests pass unmodified.
- Measured again on Task 029's fixtures and in the same way: idle CPU for
  10 s (Markdown and the 8,000-item JSON), and mouse-down to paint for a row
  click, before and after.

## 7. Acceptance criteria

1. Idle CPU near zero on every file.
2. No effect in the application re-triggers itself; the settle test pins it.
3. No per-row search from the root.
4. A focus change does not rebuild the item tree.
5. The CHANGELOG tells users.
