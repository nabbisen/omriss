# Handoff: RFC-068 — Durable File Writes and Path Integrity

Related RFC: `../../proposed/068-durable-file-writes-and-path-integrity.md`

Lifecycle status: inherited from RFC-068.

**0.18.0's second stretch item.** A save could lose the file on power loss,
break a symlink, widen permissions, or, with a non-UTF-8 name, go to another
path. The design is RFC-068 §3 as §7 completes it; where they differ, §7 wins.

**State:** ready. **Next: PR 1.**

The report goes to `.git-exclude/review-request/068-durable-writes/README.md`.
One report per PR is fine; PR 4 stops for a decision by design.

Files:

- `task-breakdown-pr-plan.md` — four PRs, constraints, gates and proofs
