# RFC-076: Editable Plain File Text

**Project:** omriss — Omriss Editor
**Milestone:** M13 — editing completeness (0.18.0)
**Status.** Proposed — implemented on `main` for 0.18.0 in four PRs (draft resync, commit path, view, documentation), awaiting architect review and a release tag before it moves to `done/`.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-004, RFC-017, RFC-032, RFC-053
**Related RFCs:** RFC-065, RFC-066, RFC-071

---

## 1. Summary

`docs/src/known-limitations.md` has said since v0.3.0 that the plain file text
view is read-only. The owner's position, stated 2026-11-03, is that this cannot
stand: **omriss is an editor, so a user must be able to edit a plain text
file.**

This RFC makes the plain file text view editable.

**Source:** owner decision, 2026-11-03, during the known-limitations schedule
audit.

## 2. Why it was deferred, and which half of that still holds

The limitation gives two reasons. They are not equally strong.

1. *"Re-indexing the entire document after every commit."* This was true when
   it was written and is now largely answered: RFC-067 §3.1's revision-keyed
   structure cache took a keystroke from 21.1 ms to 4.7 ms, and a raw commit is
   one commit, not one per keystroke. A full re-parse on commit is exactly what
   the cache was built to absorb.
2. *"Handling the case where heading titles are changed during a raw edit
   requires additional focus-remapping logic."* **This one still holds**, and
   §4.2 is this RFC's answer to it.

## 3. The shape of the problem

A raw edit is not a section edit. The user may replace the entire document,
including every heading. Afterwards:

- node identity is not preserved — the focused node may no longer exist, or may
  exist with different content at a different offset;
- the outline may be entirely different;
- the undo history must still be byte-exact.

So the question is not "how do we re-index" but **"what does the application
owe the user about where they were"**.

## 4. Design

### 4.1 A raw commit is a whole-document replacement

One `EditRecord`, one revision bump, byte-exact undo (RFC-066's P1 applies
unchanged).

**Corrected 2026-11-04: use `replace_range`, and add nothing to
`omriss-core`.** This section named `Document::replace_all`, which does not
exist. `Document::replace_range(range, text, base_revision)` already checks the
revision by strict equality, routes through `apply_replacement` so history and
revision update as one unit, and is format-neutral by design — RFC-054 J1 built
it that way so a mutation need not be heading-shaped. A whole-document
replacement is `replace_range` over `0..source.len()`.

So **`omriss-core` does not change at all** for the most dangerous feature in
the backlog, because someone generalised the primitive a release earlier.

The commit therefore carries the same strict-equality revision check as every
other structural command — a raw editor that has been open while the document changed
underneath must be refused, not merged.

### 4.2 Focus returns to the outline. It is not remapped.

**The application does not guess where the user was.** After a raw commit,
focus returns to the whole-document outline.

The alternative — remapping the previous focus onto the new structure by title,
by position, or by similarity — is the kind of cleverness that is right most of
the time and silently wrong the rest. A user who rewrote their headings and is
then dropped into *a* section, with no way to tell whether it is the one they
were in, has been misled by the application. Returning to the outline is
honest, costs one click, and can never be wrong.

If a cheap exact case is wanted later — the focused node's heading line is
byte-identical and still present — it can be added as an optimisation whose
failure mode is the outline. It is not in this RFC.

### 4.3 The structure-change warning is mandatory here

RFC-071 warns when a commit changes the outline. In a raw edit that is not an
edge case, it is the normal case. The warning must name what changed — sections
gained, lost, renamed — because the user's undo is the only way back and they
need to know it is worth using before they type again.

This is the strongest argument for RFC-071 in the product, and **RFC-076 should
not land before it**.

### 4.4 Unsaved-draft interaction

Opening the raw view with an uncommitted section draft, or leaving the raw view
with an uncommitted raw draft, must go through the same `commit_or_block` gate
as every other navigation (RFC-069). Two drafts of the same document must never
be live at once: entering the raw view commits or blocks, and so does leaving.

### 4.5 Read-only stays reachable

The view keeps a way to look without editing. A user who opens plain file text
to *inspect* bytes — which is what RFC-017 built it for — should not be one
keystroke from changing them.

## 5. Non-goals

1. **No syntax highlighting, no line numbers, no editor features.** This is a
   text area over the document's source.
2. **No partial raw edits.** The unit is the whole document.
3. **No focus remapping** — §4.2.
4. **No new format support.** The raw view shows the file's bytes whatever the
   format; JSON and Markdown behave the same here.

## 6. Validation and test plan

- **P1 extends to raw commits**: replace-all then undo reproduces the source
  byte for byte, over the generator (RFC-066).
- A raw commit that changes nothing produces no history entry.
- A raw commit against a stale revision is refused.
- Entering and leaving the raw view with a pending draft commits or blocks, and
  never discards silently.
- The structure-change warning fires with an accurate count.
- Opening a file, entering raw view, saving without editing, leaves the file
  byte-identical — the same invariant the whole project rests on.

## 7. Acceptance criteria

1. A user can edit the plain file text and save it.
2. Undo after a raw commit restores the previous bytes exactly.
3. Focus after a raw commit is the outline, always.
4. The structure-change warning (RFC-071) is in place first.
5. `docs/src/known-limitations.md` loses the entry rather than rewording it.
