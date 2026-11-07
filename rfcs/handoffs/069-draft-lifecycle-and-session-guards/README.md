# Handoff: RFC-069 — Draft Lifecycle and Session Guards

Related RFC: `../../proposed/069-draft-lifecycle-and-session-guards.md`

Lifecycle status: inherited from RFC-069.

**Item 5 of 0.18.0's scope — three of its four remaining parts.** §2.1 already
landed with RFC-076. §2.2, the window-close guard, is **not** in this handoff:
the framework cannot veto a window close, and RFC-069 §2.2 records why and what
upstream change would fix it.

Files:

- `implementation-handoff.md` — the three fixes, and the evidence behind each
- `task-breakdown-pr-plan.md` — three PRs
- `acceptance-qa-checklist.md` — gates and proofs

Small items. If any of them grows, stop and say so.
