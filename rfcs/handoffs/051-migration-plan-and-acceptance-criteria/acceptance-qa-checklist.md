# RFC-051 Acceptance / QA Checklist

## Sequencing

- [ ] RFC-048 handoff exists.
- [ ] RFC-049 handoff exists.
- [ ] RFC-050 handoff exists.
- [ ] RFC-051 handoff exists.
- [ ] RFC-052 through RFC-056 handoffs exist or are explicitly stubbed with
      RFC-053 canonical names.
- [ ] RFC-053 type core is accepted, pulled forward, or mirrored consistently.
- [ ] Existing spike code is reconciled before implementation PRs.

## Migration Phases

- [ ] Layout split foundation complete.
- [ ] Structure controls moved to Document Map.
- [ ] Writing Area simplified.
- [ ] Accessibility and safety hardening complete.
- [ ] Format-neutral readiness reviewed.

## Release Gate

- [ ] All existing source-preservation tests pass.
- [ ] Migrated UI workflows pass manual QA.
- [ ] Non-technical-user role-split QA is recorded.
- [ ] No structural controls are visible in Writing Area.
- [ ] Document Map supports required Markdown structure operations.
- [ ] Keyboard navigation works for the primary workflow.
- [ ] Plain-language label audit passes.
- [ ] No critical/high accessibility issue is open.
- [ ] Rollback/release-gating decision is followed.

## Required Commands

```text
cargo fmt
cargo test -p omriss -p omriss-ui
cargo check -p omriss-app
mdbook build docs
bash scripts/check-rfcs.sh
git diff --check
```
