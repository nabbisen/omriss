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

---

## 9. First Measurement Run — 2026-10-24

Run at commit `1509aae`, WebKitGTK 2.54.1 / GTK 3.24.52 / at-spi2-core 2.62.1,
**AT-SPI tree only — Orca was not run**. Instruments, 28 captured trees and the
event logs are in `rfcs/handoffs/060-accessibility-screen-reader-validation-plan/`.
This section exists because §7 requires deferred findings to be tracked in
source control, and a decision record outside the repository is not that.

**No §6 Blocking condition was observed.** omriss is exposed to assistive
technology with real structure: every dialog is a named modal with focus
inside, every landmark but two is named, row-menu items carry their disabled
reasons as descriptions, failed-save feedback is assertive and carries its
recovery action, and there is no unnamed `button`, `entry`, `heading` or
`menu item` in any captured state.

### 9.1 Findings and their owners

| # | Finding | Disposition |
|---|---|---|
| **F-1** | The Document Map is not a tree to assistive technology: no `role="tree"`/`treeitem`, no accessible name per row, and **no node in any state reports `selected`**. Every row's menu trigger is named identically ("Section actions"), so nine sections give nine indistinguishable controls | §9.2 — the markup comes from `dioxus-swdir-tree`, a separate repository |
| **F-3** | Confirm-delete focuses the destructive button on open | §9.3 — **fixed before 0.17.0** |
| **F-4** | Focus is not restored when a dialog or the palette closes | RFC-059 §6.2 |
| **F-5** | `Permission denied (os error 13)` reaches the user and the AT | RFC-073 §3.2 |
| **F-6** | A successful save can produce no AT event at all; the dirty indicator's name can contradict the status | RFC-059 §6.4, RFC-069 §2.1 |
| **F-7** | A malformed `.json` shows a Markdown instruction and no error | RFC-067 §3.2 |
| **P-1** | The web document's accessible name is `Dioxus app` | §9.4 |
| **P-2** | The footer (`contentinfo`) is unnamed in every state; `main` is unnamed whenever nothing is focused | §9.4 |
| **P-3** | Menu triggers are exposed as `combo box`; the Settings popup declares `role="menu"` but contains a `<select>` | §9.4 |
| **P-4** | The toolbar is named `menu.file` — "File", a menu that does not exist | §9.4 |

### 9.2 F-1 crosses a repository boundary

The Document Map's rows are rendered by **`dioxus-swdir-tree`**, a crate under
the same ownership as omriss (`github.com/nabbisen/dioxus-swdir-tree`,
Apache-2.0). Its `role` identifiers are *glyph* roles in its theme; it emits no
ARIA at all.

So F-1 is not a limitation to work around and not a line omriss can add: it is
a feature in a second repository — `role="tree"`, `treeitem`, `aria-selected`,
`aria-expanded`, `aria-level`, and a per-row accessible name — followed by a
version bump here. That sequencing is the reason this is recorded rather than
scheduled: it needs a release of another crate.

The workflow remains possible without it (the Subsections buttons and the
breadcrumb are named and operable), which is why it is Major and not Blocking.
But `omriss-external-design` describes "a keyboard-operable tree", and to
assistive technology there is no tree.

### 9.3 Dialog defaults are a conformance failure, and were fixed before the tag

`omriss-external-design` §11 states that shipped dialogs "trap focus and
**default to the safest action**", naming Delete Section and File Changed on
Disk among them. Two did not:

| Dialog | Focused on open | If the user presses Enter |
|---|---|---|
| Delete Section | **Delete** | the section and its children are deleted; Undo recovers them |
| File Changed on Disk | **Overwrite** | another program's changes on disk are overwritten; **omriss's Undo does not bring them back** |

The second was not in the measurement run — it was found while verifying the
first — and it is the worse of the two, because the data it destroys was never
in omriss's history.

Neither is §6 Blocking: both dialogs describe their consequence, so the action
is not available "without understandable context". Both are a recorded
decision not being met, which is a different and more fixable thing than a
design question. Dispatched and fixed for 0.17.0.

### 9.4 P-1 … P-4, and a note on the obvious fix for P-1

These are Minor: wording and mapping, not workflow. They are grouped here
because they want one design pass rather than four patches.

For P-1 specifically, the obvious fix does not work. `dioxus-desktop` ships
`<title>Dioxus app</title>` in its index and inserts `custom_head` immediately
before `</head>` — *after* that title. HTML takes the **first** `<title>` in
tree order, so `Config::with_custom_head("<title>Omriss</title>")` leaves the
accessible name unchanged. The document title has to be set at runtime, or the
whole index replaced.

### 9.5 What this run did not establish

Orca was not run, so nothing here was *heard*: every statement about
announcement is inference from the tree an AT receives. Event capture was
intermittent and is reported as observations, not as a log. One platform, one
WebKit build. No participant — that remains RFC-061's.

The next run should either route Orca's speech or state again that it did not,
rather than let "Orca not run" harden into a permanent footnote.
