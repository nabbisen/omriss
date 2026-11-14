# Handoff: RFC-030 — Contrast Defects

Related RFC: `../../done/030-visual-clarity-contrast-and-motion-policy.md`

Lifecycle status: inherited from RFC-030 (done). This is a defect fix against
its acceptance — "text contrast meets target tokens", target WCAG AA — not new
design.

**Found by the owner, 2026-11-15:** the welcome screen's *Open…* button,
black text on blue, hard to read. Measured from `crates/app/assets/style.css`:
the light theme's primary button is **3.71:1**, and it is not the only pair
below 4.5:1.

**State:** ready. One PR.

Files:

- `implementation-handoff.md` — the failing pairs, the fix, the guard test
- `task-breakdown-pr-plan.md` — one PR
- `acceptance-qa-checklist.md` — gates and proofs

The report goes to
`.git-exclude/review-request/030-contrast-defects/README.md`.
