# Handoff: RFC-070 — `split_section` Input Validation, with RFC-066's P4

Related RFC: `../../proposed/070-split-section-input-validation.md`
(and `../../done/066-property-based-and-fuzz-testing.md` §3.1b for P4)

Lifecycle status: inherited from RFC-070.

**Item 11, the last core item of 0.18.0.** It closes the one `#[ignore]`d
property and adds the rename/join property that nothing has yet. The design is
RFC-070 §4.1, §8 and §9; where §8 and §9 differ, §9 wins.

**State:** PR 1 and PR 2 landed. **Next: PR 3** (RFC-070 §10), then the RFC
closes.

The report goes to `.git-exclude/review-request/070-split-and-p4-3/README.md`.

Files:

- `implementation-handoff.md` — what to build, and the constraints
- `task-breakdown-pr-plan.md` — two PRs
- `acceptance-qa-checklist.md` — gates and proofs
