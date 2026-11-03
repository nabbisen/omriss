# Handoff: RFC-076 — Editable Plain File Text

Related RFC: `../../proposed/076-editable-plain-file-text.md`

Lifecycle status: inherited from RFC-076.

**Item 4 of 0.18.0's scope**, and the most dangerous thing in the backlog: it
lets a user replace an entire document in one commit. It is scheduled fourth
rather than ninth so that it gets the *most* exercise before the release ends,
not the least.

Files:

- `implementation-handoff.md` — the two things reading the tree changed, the
  design that must not be softened, and what the danger actually is
- `task-breakdown-pr-plan.md` — four PRs
- `acceptance-qa-checklist.md` — gates, and the proofs that matter

**Read `implementation-handoff.md` §2 first.** One of its findings removes work
from the RFC and the other adds a prerequisite that is not in the scope table.
