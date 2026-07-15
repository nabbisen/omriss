# RFC-060: Accessibility and Screen-Reader Validation Plan

**Project:** omriss — Omriss Editor
**Milestone:** M15 — Accessibility validation (proposed)
**Status.** Proposed
**Document type:** Follow-up validation RFC
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-051
**Related RFCs:** RFC-027, RFC-028, RFC-029, RFC-030, RFC-059

---

## 1. Summary

This RFC owns the full accessibility and screen-reader validation program that
was originally listed inside the RFC-048/M10 QA package.

RFC-048/M10 keeps narrow smoke checks for obvious role-split regressions. It
does not claim that full assistive-technology validation has passed.

## 2. Motivation

Accessibility and screen-reader validation is broad enough to need its own
plan, evidence rules, severity policy, and acceptance decision. Keeping it as
embedded M10 paperwork risks either blocking the release indefinitely or closing
the pass too shallowly.

This RFC prevents accidental weak closure by making the deferred gate explicit
and source-controlled.

## 3. Scope

In scope:

- screen-reader smoke and workflow validation;
- landmark and accessible-name validation;
- dialog, menu, status, and error announcement checks;
- severity policy for accessibility findings;
- build/source identity requirements for each validation run;
- pass/fail/defer rules.

Out of scope:

- general keyboard-only QA already tracked by RFC-059;
- non-technical-user participant validation, owned by RFC-061;
- full WCAG conformance certification.

## 4. Validation Requirements

Each validation run must record:

- app version and commit/source identity;
- operating system;
- desktop runtime/WebView;
- assistive technology used;
- test fixture or document;
- tester role;
- pass/fail/blocked result for each scenario.

## 5. Required Scenarios

- Header/toolbar exposes useful accessible names.
- Document Map is discoverable as the structure/navigation region.
- Writing Area is discoverable as the primary editing region.
- Quick Actions is discoverable and dismissible.
- Show plain file text is discoverable and described as read-only.
- Row menus expose useful labels and disabled explanations.
- Dialogs announce purpose, field labels, primary actions, and safe cancel
  paths.
- Status/footer messages are exposed through live regions or equivalent
  assistive-technology feedback.
- Errors do not expose byte ranges, node IDs, parser internals, or raw system
  errors.
- Known limitations are announced or documented clearly enough that users are
  not misled.

## 6. Severity Policy

Blocking:

- assistive-technology users cannot open, edit, save, or recover from a common
  dialog;
- destructive action can be triggered without understandable context;
- focus disappears in a way that prevents recovery;
- status/error feedback for failed save or refused navigation is unavailable.

Major:

- important labels are unclear but the workflow remains possible;
- disabled controls lack enough explanation;
- known limitation is not discoverable.

Minor:

- wording polish, announcement order, or redundant labels.

## 7. Acceptance Criteria

- Required scenarios are run and recorded.
- No blocking accessibility finding is open.
- Critical/high accessibility risks are fixed or explicitly accepted by the
  owner with release-note coverage.
- Any deferred findings are tracked in source control.
- Documentation and release notes do not claim full accessibility validation
  before this RFC is complete.

## 8. Relationship To RFC-048/M10

RFC-048/M10 may proceed with only narrow smoke checks if:

- role labels are visible and plain;
- no known new critical/high accessibility issue is open;
- known limitations are documented;
- this RFC remains open as the owner of full accessibility/screen-reader
  validation.

RFC-048/M10 must not mark the full accessibility/screen-reader pass as passed.
