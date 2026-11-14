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
their own and stay in the RFC.

**The design is RFC-059 §12.** It is the authority; where anything in this
directory disagrees with it, §12 wins.

How it got there: Phase A was a technical investigation by the dev team —
measurements on the real WebView, a throwaway prototype, and the owner's
Windows checklist — reported in `design-output.md`. The architect made the
design decisions from those findings and recorded them in §12. The report's
own recommendations are input to that, not decisions.

Files:

- `implementation-handoff.md` — the cause, and what the investigation was asked
- `design-output.md` — the investigation's findings (Phase A, done)
- `task-breakdown-pr-plan.md` — three PRs, the third only if Windows needs it
- `acceptance-qa-checklist.md` — gates and proofs, including the owner's
  Windows check
