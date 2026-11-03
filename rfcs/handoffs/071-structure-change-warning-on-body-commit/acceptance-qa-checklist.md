# RFC-071 — Acceptance and QA Checklist

Related RFC: `../../proposed/071-structure-change-warning-on-body-commit.md`

Lifecycle status: inherited from RFC-071.

---

## 1. Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace` — state the count and name every added test
- [ ] `cargo test` (default members) — `omriss-ui` and `omriss-components` are
      both in it, so the detection and catalog tests run here
- [ ] `bash scripts/check-rfcs.sh`
- [ ] `mdbook build docs`
- [ ] `omriss-ui` still has exactly one dependency

## 2. The rule, demonstrated

- [ ] Every case in the PR plan's table, each asserting whether it warns and
      which shape
- [ ] **The silent cases are tested as carefully as the loud ones.** A warning
      that fires on an ordinary delete is the failure that makes users stop
      reading warnings
- [ ] One function expresses the expected-delta rule, and the review says which
- [ ] RFC-066's P2 and P3 pass **unchanged** — not adjusted to agree with the
      detector

## 3. The enumeration (handoff §6)

- [ ] Every source-rewriting entry point in `crates/core/src/doc/structural/`'s
      public surface listed, each marked covered or not, with a reason for any
      that is not. By construction, not by sampling

## 4. Interpolation (PR 1)

- [ ] `t()` is unchanged and the ~100 static keys still use it
- [ ] The placeholder-set parity test exists and bites — add `{count}` to an
      `en` key without its `ja` counterpart and watch it fail, then restore
- [ ] RFC-073's Status records that §4.1 landed here

## 5. The message

- [ ] All three shapes render, in `en` and `ja`
- [ ] The disappearance case names the class of cause (§4.3)
- [ ] `Ctrl+Z` is named in the message
- [ ] It is **not** a dialog
- [ ] One GUI pass under the GUI rules: the message appears for a `<!--` commit
      and **does not appear** for an ordinary edit. Capture the second — an
      absence is the harder thing to prove and the easier thing to get wrong

## 6. What must not have happened

- [ ] `omriss-core` still accepts every edit. This is a warning, never a
      rejection
- [ ] No new dependency in any crate
- [ ] No modal

## 7. Report content

1. Gate output and the test count.
2. The §2 table as results.
3. The §3 enumeration.
4. The parity test's deliberate-break output.
5. The GUI evidence, including the absence case.
6. What was not run, or not proven.
