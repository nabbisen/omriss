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

RFC-060 is a validation RFC: a run produces a record and findings. A PR plan
exists only for findings accepted for fixing that RFC-060 itself owns, which
are §9.4's P-1 to P-4, designed in §10.

**State:** P-1 to P-4 are 0.18.0's item 10. **Next: PR 3**
(`task-breakdown-pr-plan.md`). The report goes to
`.git-exclude/review-request/060-names-3/README.md`.

- `task-breakdown-pr-plan.md` — P-1 to P-4
