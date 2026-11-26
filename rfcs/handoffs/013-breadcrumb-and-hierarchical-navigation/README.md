# Handoff: RFC-013 — Breadcrumb Layout Defects

Related RFC: `../../done/013-breadcrumb-and-hierarchical-navigation.md`

Lifecycle status: inherited from RFC-013 (done). A defect fix against its
goals — "display the path", "define overflow behavior" — not new design.

**Found by the owner, 2026-11-26:** the path is spread across the full width
in equal gaps instead of reading from the left; and long paths are a
question.

**State:** PR 1 landed (`c1fdf3f`). **Next: PR 2, a correction to the
cutting order** — the architect's spec was wrong (`task-breakdown-pr-plan.md`).
The report goes to `.git-exclude/review-request/013-breadcrumb-cut-order/README.md`.

Files:

- `implementation-handoff.md` — the cause and the decided fix
- `task-breakdown-pr-plan.md` — one PR
- `acceptance-qa-checklist.md` — gates and proofs
