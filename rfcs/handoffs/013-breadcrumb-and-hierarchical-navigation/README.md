# Handoff: RFC-013 — Breadcrumb Layout Defects

Related RFC: `../../done/013-breadcrumb-and-hierarchical-navigation.md`

Lifecycle status: inherited from RFC-013 (done). A defect fix against its
goals — "display the path", "define overflow behavior" — not new design.

**Found by the owner, 2026-11-26:** the path is spread across the full width
in equal gaps instead of reading from the left; and long paths are a
question.

**State: closed.** PR 1 (`c1fdf3f`) and PR 2 (`1202748`) landed. The cutting
order's second phase (ancestors shrinking once the current crumb is spent) is
held by RFC-079's layout check when it exists.

Files:

- `implementation-handoff.md` — the cause and the decided fix
- `task-breakdown-pr-plan.md` — one PR
- `acceptance-qa-checklist.md` — gates and proofs
