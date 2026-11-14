# RFC-059 — Task Breakdown and PR Plan

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Lifecycle status: inherited from RFC-059.

Phase A first, then four PRs in order: each builds on `focus.rs`.

---

## Phase A — measure and design (no product code)

The measurements in the implementation handoff §4, and `design-output.md` in
this directory. The only committed code allowed is the `gui_focused` helper
and its self-test — Phase B needs it, and it is not product code.

Probes are throwaway: keep them under `.git-exclude/tmp/`, and put what they
found, not the probes, in `design-output.md`.

**Stop here.** The architect reviews `design-output.md`; the design accepted
there is recorded in RFC-059 before PR 1 starts.

## PR 1 — `focus.rs`, with no behaviour change but one

Scopes (Rule 1): the module, the scope marker on dialogs and Quick Actions,
the old scripts and the palette sentinels deleted, the section-title dialog's
extra `eval` and `menu_focus.rs` folded in. The source-scanning guard.

The one behaviour change: Shift+Tab no longer escapes Quick Actions
(`RFC048-QA-019`). The limitations section on reverse Tab traversal goes in
this PR.

Show that the guard bites: put a `.focus()` back into `app.rs`, watch the test
fail, restore.

## PR 2 — restoration (F-4)

Rule 2: the remembered opener, the fallback function and its level-1 table,
for every dialog and for Quick Actions. GUI check: after each close, exactly
one node is focused, and it is the expected one.

## PR 3 — the search panel (§6.2b)

The scope marker on `.search-overlay`; initial focus and restoration, and
containment only if `design-output.md` concluded it. The "search panel is the
exception" paragraph of the limitations page goes in this PR.

## PR 4 — re-render (§6.2a)

Rule 3 as `design-output.md` settled it. GUI check: Ctrl+S, then Ctrl+F at
once, opens the search panel with its field focused; the same for Esc out of a
section, zoom in and out, and Ctrl+\` there and back. The limitations section
on shortcuts after a view change or save is deleted or reduced to what still
holds.

Then the owner's Windows checklist, re-run against this build.
