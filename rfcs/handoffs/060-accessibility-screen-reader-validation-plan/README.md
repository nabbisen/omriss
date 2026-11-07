# Handoff: RFC-060 — Accessibility and Screen-Reader Validation Plan

Related RFC: `../../proposed/060-accessibility-screen-reader-validation-plan.md`

Lifecycle status: inherited from RFC-060.

Files:

- `implementation-handoff.md` — why this can start now, what the first probe
  already established, the instrument, and the constraints on a run
- `acceptance-qa-checklist.md` — the measurement scenarios, the required record,
  and the severity rules
- the instruments — `atspi_probe.py`, `atspi_act.py`, `atspi_events.py` and
  `atspi_text.py` — moved on 2026-11-07 to **`scripts/gui/`**, because every GUI
  task uses them, not only this one. See `scripts/gui/README.md`
- `baseline-accessible-tree.txt` — the architect's first probe, 2026-10-22.
  Reference, **not** a validation run

There is no `task-breakdown-pr-plan.md`. RFC-060 is a validation RFC: a run
produces a record and findings, not a pull request. A PR plan appears only when
findings are accepted for fixing, and it belongs to whichever RFC owns the fix.
