# Handoff: RFC-068 — Durable File Writes and Path Integrity

Related RFC: `../../proposed/068-durable-file-writes-and-path-integrity.md`

Lifecycle status: inherited from RFC-068.

**0.18.0's second stretch item.** A save could lose the file on power loss,
break a symlink, widen permissions, or, with a non-UTF-8 name, go to another
path. The design is RFC-068 §3 as §7–§11 complete it; the later section wins.

**State:** PR 1–3 and PR 4's first part landed. **Next: PR 4's ceiling**
(RFC-068 §11), starting with the plain-file-text measurement.

The report goes to `.git-exclude/review-request/068-durable-writes-5/README.md`.
One report per PR is fine; PR 4 stops for a decision by design.

Files:

- `task-breakdown-pr-plan.md` — four PRs, constraints, gates and proofs
