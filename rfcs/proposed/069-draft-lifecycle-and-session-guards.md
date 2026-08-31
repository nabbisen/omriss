# RFC-069: Draft Lifecycle and Session Guards

**Project:** omriss — Omriss Editor
**Milestone:** M13 — session-state hardening (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-016, RFC-044, RFC-054
**Related RFCs:** RFC-033, RFC-049, RFC-059

---

## 1. Summary

"Is there unsaved work, and where does it live?" is answered in four different
places by three different mechanisms, none of which agree. The visible symptoms
are a dirty dot that will not clear, a false unsaved-changes warning
immediately after a successful save, an undo that appears to do nothing, and a
window that closes without asking.

This RFC makes the draft lifecycle one thing with one owner.

**Source of this RFC:** AUDIT-0170-007, -019, -021 and -025, plus findings 1
and 5 of the Task 011 Linux smoke run.

## 2. The defects

### 2.1 The draft signal is never resynced after commit

`commit_focused_body` stores `"typed\n"` — the newline appended for a non-last
section — while the draft signal still holds `"typed"`. `is_pending` therefore
stays permanently true.

Consequences, in increasing order of user harm:

- the dirty dot stays lit after a truthful, byte-correct save;
- every navigation, preview toggle or search open **re-commits an identical
  body** — a no-op that still bumps the revision and pushes an undo record, so
  the next Ctrl+Z appears to do nothing;
- `handle_save` never resyncs, so **Open right after a successful save falsely
  warns about unsaved changes**.

The fix is one call: `draft_sync::sync` after every successful
`commit_or_block`. Three sites skip it — `handle_save`,
`open_search_if_available`, and `AppCommand::TogglePreview`. `do_blur` already
does it correctly, which is why blurring clears the dot and saving does not.

### 2.2 No window-close guard

RFC-016 §4 lists "closing app" among the warn-before guards, §6 requires "Close
with unsaved changes prompts", and §7's acceptance criterion is "No user text is
discarded without explicit confirmation." The file-level guards exist
(`Modal::UnsavedBeforeOpen`, `Modal::UnsavedBeforeNew`); the window-close guard
was never built. No `CloseRequested` or `WindowEvent` handling exists anywhere
in `crates/app/`.

Already disclosed in RFC-016's header and in `known-limitations.md`. This RFC is
where it gets built.

### 2.3 Dirtiness is decided by a hash

`ui/session.rs:41-46` answers "is the document dirty?" with a 64-bit
`DefaultHasher` digest plus length, not the text. A collision reports a modified
document as clean and skips the unsaved-changes prompt. `DefaultHasher::new()`
is unseeded, so the mapping is deterministic rather than randomised per run.

The probability is negligible; the design is still wrong for a product whose
promise is not losing work, and the document is already in memory, so the exact
comparison is available for free.

### 2.4 Status is simultaneously a display string and a command bus

`app/shell/app.rs:192-272`. The Document Map requests modals by writing sentinel
strings (`"struct.delete.pending"`, `"struct.rename.pending"`) into the
**user-visible status signal**, which the root component matches on and mutates
signals in response **during render**.

Writing signals from a render body is the pattern most likely to produce a
render loop later. It also conflates `struct.split.pending` and
`struct.add_inside.pending` onto one action.

## 3. Design

1. **One rule: a successful commit is always followed by a sync.** Enforce it by
   making `commit_or_block` do the sync itself, so the unsynced form is
   unwritable — the same technique RFC-065 applies to splices and
   AUDIT-0170-018 requests for history recording. Three callers forgetting the
   same follow-up call is a signature that the call should not be the caller's
   responsibility.
2. **Handle `WindowEvent::CloseRequested`**, routing to the existing unsaved
   modal with Save / Discard / Cancel, per RFC-016 §5's wireframe.
3. **Key dirtiness off retained text or off revision plus undo depth**, not a
   digest.
4. **Replace the status sentinel bus** with a typed
   `Signal<Option<StructureRequest>>` consumed in an effect rather than in
   render, and split the conflated action.

## 4. Non-goals

1. **No autosave**, and no recovery of unsaved buffers after a crash.
2. **No focus-restoration work.** Findings 2, 7 and 8 of the Task 011 run share
   a different root cause — `onkeydown` is bound to the root `div.app`, so
   shortcuts are not delivered once a view change moves focus to `<body>` — and
   belong to RFC-059, which exists for exactly that.

## 5. Validation and test plan

- After a commit, `is_pending` is false: asserted at session level for a
  non-last section whose body lacks a trailing newline, which is the shape that
  triggers it.
- Save, then Open: no unsaved-changes modal.
- Navigate twice after one edit: exactly one undo record, and one Ctrl+Z
  restores the pre-edit text.
- Close with a pending edit: the modal appears; Cancel leaves the buffer intact.
- No signal is written during a render body — enforced by review, since it is
  not expressible as a test.

## 6. Acceptance criteria

1. The four symptoms in §2.1 are gone, each with a regression test.
2. RFC-016's §7 acceptance criterion is met, and its header note is updated to
   say so rather than removed — the record of the gap stays.
3. Dirtiness is exact.
4. No modal is requested through the status string.
