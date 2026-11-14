# Handoff: RFC-059 — Keyboard Focus Follow-Up Controls

Related RFC: `../../proposed/059-keyboard-focus-follow-up-controls.md`

Lifecycle status: inherited from RFC-059.

**Item 6 of 0.18.0's scope.** Three symptoms with one cause, as `ROADMAP.md`
lists them:

- §6.2a — after Ctrl+S (or Esc out of a section) the next shortcut does
  nothing, on Windows always, on Linux sometimes;
- §6.2b — Ctrl+F opens the search panel, but its field never takes focus;
- RFC-060 F-4 — after a dialog or Quick Actions closes, **no** node is focused.

Plus §6.2's Shift+Tab escape from Quick Actions (`RFC048-QA-019`), because it
is the same mechanism and the fix below absorbs it.

**Out of scope:** §6.1 header reachability, §6.3 pending-draft undo, §6.4
dirty indicators, §6.5 direct focus commands. They are design questions of
their own and stay in the RFC. §6.5 will be cheap once this lands; that is
deliberate, not an invitation to do it now.

**This is a two-phase handoff.** RFC-059 §7 says the design is settled before
implementation, and the two WebViews are known to disagree about focus, so
the first deliverable is a short design note backed by measurements — no
product code. The architect reviews it; implementation starts after.

Files:

- `implementation-handoff.md` — the cause, the recommended design, what
  Phase A must measure
- `task-breakdown-pr-plan.md` — Phase A, then four PRs
- `acceptance-qa-checklist.md` — gates and proofs, including the owner's
  Windows check

Phase A hands back **one** file: `design-output.md` in this directory.
