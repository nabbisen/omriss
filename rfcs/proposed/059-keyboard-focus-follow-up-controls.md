# RFC-059: Keyboard Focus Follow-Up Controls

**Project:** omriss — Omriss Editor
**Milestone:** M10 follow-up hardening (proposed)
**Status.** Proposed
**Document type:** Follow-up RFC design seed
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-049, RFC-050, RFC-051
**Related RFCs:** RFC-014, RFC-022, RFC-028, RFC-044, RFC-058

---

## 1. Summary

This RFC records the unresolved keyboard and focus-control follow-ups found
during RFC-048 M10 keyboard-only QA.

The follow-up theme is:

- make primary work surfaces directly reachable by keyboard;
- make transient overlays and Quick Actions fully self-contained keyboard
  contexts;
- define the pending-draft undo/redo contract;
- clarify dirty-state indicators.

This RFC is a tracking/design seed, not implementation authorization.

## 2. Motivation

RFC-048 introduced the split Document Map and Writing Area workflow. Manual
keyboard-only QA showed that the workflow can be completed without pointer
input, but several parts are still inefficient, unclear, or partially broken.

The issues are not all the same kind:

- some are accessibility/focus containment defects;
- some are interaction-design questions;
- some are wording/state-model questions;
- one is a direct keyboard navigation affordance gap.

Keeping these items only in `.git-exclude/qa/` would make them easy to lose
after the M10 QA package is closed. This RFC promotes the deferred items into
source-controlled design tracking.

## 3. Source Issues

This RFC tracks the following M10 QA findings:

- `RFC048-QA-010`: header and toolbar controls are reachable only after many
  Tab presses, which is technically keyboard-accessible but poor UX.
- `RFC048-QA-019`: Quick Actions is partially keyboard-fixed, but Shift+Tab can
  still escape or fail to cycle inside the palette.
- `RFC048-QA-020`: Ctrl+Z/Ctrl+Y behavior with a pending Writing Area draft is
  unclear and sometimes requires extra key presses before visible undo occurs.
- `RFC048-QA-021`: local draft dirty state and document unsaved state are both
  visible, but their different meanings are not obvious.
- `RFC048-QA-022`: keyboard users need direct focus shortcuts or commands for
  the Writing Area editor and the current active Document Map node.

The `+ Top` creation-strip question from `RFC048-QA-013` is intentionally not
tracked here. It is already tracked by
[RFC-058](./058-document-map-placement-complete-creation-controls.md).

`RFC048-QA-005` is also not tracked here because it is a manual retest gate for
the merge implementation, not a new design follow-up.

## 4. Goals

- Define a predictable keyboard path to the primary header/toolbar actions.
- Keep Quick Actions focus inside the palette until it is dismissed or a
  command is executed.
- Define whether pending-draft undo/redo belongs to the textarea's native undo
  stack, the app document undo stack, or an explicit staged model.
- Clarify dirty indicators so users can distinguish "draft not applied" from
  "document not saved".
- Add explicit keyboard commands or shortcuts for:
  - focusing the Writing Area editor;
  - focusing the active Document Map node.

## 5. Non-goals

- This RFC does not change Document Map creation placement; RFC-058 owns that.
- This RFC does not require drag-and-drop.
- This RFC does not change source-preserving edit semantics.
- This RFC does not replace the existing command registry model.
- This RFC does not require implementation before M10 release unless the owner
  classifies one of the tracked issues as release-blocking.

## 6. Design Questions

### 6.1 Header and Toolbar Reachability

The current problem is not that header controls are unreachable. The problem is
that reaching them through repeated Tab presses is slow and hard to predict.

Design options:

1. Add a command or shortcut that focuses the header action group.
2. Reorder the tab sequence so the header action group is encountered earlier.
3. Treat the header as command-palette-first and ensure every header action is
   discoverable and executable from Quick Actions.
4. Accept current behavior as a documented residual.

The review should decide whether this is a release-blocking issue or a tracked
post-M10 keyboard polish item.

### 6.2 Quick Actions Focus Containment

Quick Actions should behave as a keyboard-owned overlay:

- opening Quick Actions moves focus into the palette;
- ArrowUp, ArrowDown, Home, and End move the active result;
- Enter executes the active result;
- Tab and Shift+Tab remain inside the palette;
- Escape closes the palette and restores focus predictably.

The unresolved point from QA is Shift+Tab containment after partial fixes.
Implementation should not add more ad hoc traps until the intended focus owner
and restoration target are explicit.

### 6.3 Pending-Draft Undo and Redo

The Writing Area has two potential undo contexts:

- native textarea undo for the in-progress draft;
- app-level undo/redo for committed document edits.

The user-facing contract must be explicit. Candidate contracts:

1. While the textarea owns focus and has a pending draft, Ctrl+Z/Ctrl+Y operate
   only on native textarea undo/redo.
2. App-level undo/redo is disabled while a pending draft exists, and the app
   shows a status hint telling the user to apply or cancel the draft first.
3. Ctrl+Z first undoes within the pending draft, then crosses into app-level
   undo only after the draft is applied or empty.

The accepted contract must define visible status behavior and must not discard
unapplied text.

### 6.4 Dirty-State Indicators

Current behavior exposes two states:

- local draft dirty: the focused section draft differs from the current section
  snapshot and has not yet been applied to the in-memory document;
- document dirty: the in-memory document differs from the file on disk.

The design should decide whether the UI needs clearer labels, separate icons,
or documentation-only clarification.

### 6.5 Direct Focus Commands

Keyboard-only users need direct movement between the two primary work surfaces.

At minimum, the design should define commands for:

- focus the Writing Area editor for the active section;
- focus the active Document Map node, not only the Document Map container.

The commands should be available from Quick Actions. Shortcut assignment should
avoid conflicts with platform and browser/WebView defaults.

## 7. Required Design Output

Before implementation, the design review should settle:

- whether each tracked issue is M10-blocking or post-M10 follow-up;
- the accepted Quick Actions focus containment model;
- the pending-draft undo/redo ownership contract;
- the dirty-state language or visual model;
- command identifiers and shortcut policy for direct focus movement;
- manual QA additions for keyboard-only and screen-reader passes.

## 8. Implementation Constraints

- Focus trapping must be centralized where possible, not duplicated across
  individual overlays.
- Keyboard event handlers must avoid reentrant signal borrow failures.
- Command registry additions must remain discoverable from Quick Actions and
  documentation.
- Any shortcut must be documented in the keyboard reference.
- Pending-draft handling must preserve user text even when navigation or undo
  is refused.

## 9. Manual QA Requirements

Manual QA must cover:

- opening Quick Actions, using Arrow keys, Enter, Tab, Shift+Tab, and Escape;
- focus restoration after Quick Actions closes;
- direct keyboard focus to the Writing Area editor;
- direct keyboard focus to the active Document Map node;
- header/toolbar action access without excessive Tab traversal;
- pending-draft Ctrl+Z/Ctrl+Y behavior before and after blur/apply;
- dirty-state indicators before draft apply, after draft apply, and after save;
- screen-reader naming for new commands and status messages.

## 10. Acceptance Criteria

- Every source QA issue listed in this RFC has a settled release classification.
- Quick Actions has a reviewed focus containment contract.
- Pending-draft undo/redo has a reviewed user-facing contract.
- Dirty-state indicators distinguish draft and document dirty states, either in
  UI or documentation.
- Direct focus commands are defined for the Writing Area editor and active
  Document Map node.
- Follow-up implementation includes regression coverage or manual QA evidence
  for keyboard-only workflows.

## 11. Open Questions

1. Should `RFC048-QA-019` remain blocking for M10, or can it be deferred with a
   clear release note?
2. Should direct focus commands use visible shortcuts immediately, or launch as
   command-palette-only commands first?
3. Should local draft dirty state remain a small section-level marker, or move
   into the global status/footer language?
4. Should header reachability be solved through direct focus, command coverage,
   or tab-order redesign?
