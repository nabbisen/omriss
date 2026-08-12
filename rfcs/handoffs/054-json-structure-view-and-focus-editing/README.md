# RFC-054 Handoff

Companion execution package for:

- RFC: `rfcs/done/054-json-structure-view-and-focus-editing.md`
- Milestone: M12 — Structured Format Support

Replaces the M10-era stub, whose exit condition ("replace after RFC-053 is
accepted") is met — RFC-053 is implemented and disposed.

## Contents

- `implementation-handoff.md` — purpose, six slices, non-change scope, risks
- `task-breakdown-pr-plan.md` — J1–J6 detail and sequencing
- `acceptance-qa-checklist.md` — per-slice verification

## What this work is

JSON becomes the first non-Markdown format omriss can open, navigate, and edit,
through the RFC-053 adapter boundary.

## What is different about it

**This is the first work since M10 that users can see.** Every RFC-053 slice was
contract-bound to change nothing observable; from J3 onward that no longer
holds. Reviews need to know what to look at, and the CHANGELOG needs to say what
changed — the "no user-facing change" phrasing from RFC-053's slices does not
apply here.

Start at **RFC-054 §0**, not §1. It records what RFC-053 left ready, and the
`Document::replace_range` prerequisite that blocks all JSON mutation until J1
lands.
