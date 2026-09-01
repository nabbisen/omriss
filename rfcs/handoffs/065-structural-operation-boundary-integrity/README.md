# RFC-065 Handoff — Structural Operation Boundary Integrity

**Governing RFC:** [RFC-065](../../proposed/065-structural-operation-boundary-integrity.md)
**Milestone:** M12 hardening — 0.17.0 ship gate
**State:** inherits RFC-065 (Proposed)
**Prerequisite:** the RFC-066 handoff, landed with its properties failing.

Contents:

- `implementation-handoff.md` — scope, slices, constraints, required evidence
- `acceptance-qa-checklist.md` — what must be demonstrated before review

This is the highest-risk slice in the hardening sequence: it changes shipped
byte-preservation code that the golden suites already cover.
