# Handoff: RFC-071 — Structure-Change Warning

Related RFC: `../../proposed/071-structure-change-warning-on-body-commit.md`

Lifecycle status: inherited from RFC-071.

**Item 3 of 0.18.0's scope**, and the gate on item 4: RFC-076 lets a user
replace a whole document in one commit, and must not land before the warning
that tells them what that did.

Files:

- `implementation-handoff.md` — the two things the RFC does not say, the
  detection rule, and where each piece lands after RFC-075
- `task-breakdown-pr-plan.md` — three PRs
- `acceptance-qa-checklist.md` — gates and the cases that must be shown

**Read `implementation-handoff.md` §2 before planning.** One of RFC-071's
dependencies is scheduled *after* it in 0.18.0, which is my ordering error, and
§2.1 says how it is resolved.
