# RFC-066 Handoff — Property-Based and Fuzz Testing

**Governing RFC:** [RFC-066](../../proposed/066-property-based-and-fuzz-testing.md)
**Milestone:** M12 hardening — 0.17.0 ship gate
**State:** inherits RFC-066 (Proposed)

Contents:

- `implementation-handoff.md` — scope, slices, constraints, required evidence
- `acceptance-qa-checklist.md` — what must be demonstrated before review

**Do this before RFC-065.** The properties must be shown *failing* against
current `main`. A property written after the fix proves nothing about whether
the fix is complete.
