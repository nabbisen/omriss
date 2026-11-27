# Handoff: RFC-073 — Complete Localization (the 0.18.0 slice)

Related RFC: `../../proposed/073-complete-localization-and-locale-persistence.md`

Lifecycle status: inherited from RFC-073.

**Item 9 of 0.18.0's scope.** A Japanese user meets English, and the raw text
`Permission denied (os error 13)`, at the moment a file fails to open. The
slice is RFC-073 §8: typed open and save failures, the dialog filter labels,
`from_tag`, and JSON list labels.

**State:** PR 1–3 landed. **Next: PR 4** (RFC-073 §9), then RFC-073 closes.

The report goes to `.git-exclude/review-request/073-localization-2/README.md`.

Files:

- `implementation-handoff.md` — what to build, and the constraints
- `task-breakdown-pr-plan.md` — three PRs
- `acceptance-qa-checklist.md` — gates and proofs
