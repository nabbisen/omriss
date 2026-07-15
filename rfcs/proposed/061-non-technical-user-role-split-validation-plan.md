# RFC-061: Non-Technical User Role-Split Validation Plan

**Project:** omriss — Omriss Editor
**Milestone:** M15 — Product validation (proposed)
**Status.** Proposed
**Document type:** Follow-up validation RFC
**Primary audience:** Product owner, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-050, RFC-051
**Related RFCs:** RFC-041

---

## 1. Summary

This RFC owns the non-technical-user validation that was originally listed
inside the RFC-048/M10 QA package.

RFC-048/M10 keeps narrow role-split smoke checks. It does not claim that
participant validation has been completed.

## 2. Motivation

The RFC-048 role split is intended to make omriss easier for non-technical
Markdown users:

```text
Left side: organize sections.
Right side: write the selected section.
```

That product claim needs real participant validation. It should not be closed as
release paperwork unless the walkthrough is actually run and recorded.

## 3. Scope

In scope:

- participant criteria;
- task statement;
- privacy rules;
- observer notes;
- pass/fail/defer policy;
- source identity requirements.

Out of scope:

- screen-reader/accessibility validation, owned by RFC-060;
- broad UX research unrelated to the RFC-048 role split;
- analytics or telemetry collection.

## 4. Privacy Rules

Do not record participant names, contact details, employer, or other personal
data. Use an anonymized label such as `Participant A`.

Record only:

- participant fit;
- task outcome;
- observed hesitation;
- unclear labels;
- workflow blockers;
- whether coaching was required.

## 5. Required Walkthrough

Ask the participant to complete this workflow without explaining the product's
internal design:

```text
Open the Markdown file, choose a section, write in it, organize at least one
section, save the file, and undo one organization change.
```

Required observations:

- participant can identify where to choose or organize sections;
- participant can identify where to write;
- participant understands the left/right role split well enough to proceed;
- participant can add or rename a section without coaching;
- participant can save after editing;
- participant can undo an organization change;
- participant does not rely on a visible Done/Commit workflow;
- participant does not use plain file text as the primary writing path.

## 6. Severity Policy

Blocking:

- participant cannot complete open, select, write, organize, save, and undo
  without direct coaching;
- labels cause the participant to use plain file text as the primary writing
  surface;
- participant cannot distinguish organizing from writing.

Major:

- participant completes the task but only after repeated hesitation around the
  role split;
- add/rename/save/undo labels are unclear but recoverable.

Minor:

- wording polish or isolated hesitation that does not affect task completion.

## 7. Acceptance Criteria

- At least one representative participant completes the required walkthrough.
- Observations are recorded without personal data.
- No blocking role-split finding is open.
- Major findings are fixed, explicitly deferred, or accepted by the owner with
  release-note coverage.
- Documentation and release notes do not claim non-technical-user validation
  before this RFC is complete.

## 8. Relationship To RFC-048/M10

RFC-048/M10 may proceed with narrow smoke checks if:

- role labels are visible and plain;
- structural controls are not visible in the Writing Area;
- normal manual and keyboard workflows are recorded;
- this RFC remains open as the owner of participant validation.

RFC-048/M10 must not mark the full non-technical-user walkthrough as passed.
