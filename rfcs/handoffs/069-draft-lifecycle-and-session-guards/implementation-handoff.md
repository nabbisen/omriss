# RFC-069 — Implementation Handoff (§2.3, §2.4, §2.5)

Related RFC: `../../proposed/069-draft-lifecycle-and-session-guards.md`

Lifecycle status: inherited from RFC-069.

---

## 1. §2.3 — dirtiness by exact text, not a hash

`SavedFingerprint` (`crates/ui/src/session.rs:30-50`) holds `revision`, `len`
and a `DefaultHasher` digest. Its `matches()` already takes a fast path when the
revision is unchanged; otherwise it compares length and digest, so a collision
reports a modified document as clean and skips the unsaved-changes prompt.

**The fix:** keep the revision fast path, and replace `len` and `hash` with the
saved text itself, compared exactly. The document is already in memory; one
more copy of it is nothing, and the comparison only runs when the revision has
moved.

Do **not** switch to "dirty when the revision differs" alone. Undoing back to
the saved text would then report the document dirty — a correct-sounding
answer to the wrong question. Exact text comparison makes "undo to the saved
state is clean" fall out for free.

## 2. §2.4 — `on_navigate` goes through `commit_or_block`

`crates/app/src/shell/app.rs:342-357`: the search panel's `on_navigate` calls
`current_snapshot()` and `commit_focused_body()` directly — the Markdown
section-body path, with no format check. On a JSON document it is harmless only
because a JSON node id happens not to resolve in the Markdown outline. Nothing
in the code expresses that intent.

**The fix:** route it through `draft_sync::commit_or_block`, which is already
format-aware, and delete the inline copy.

**Then make the bypass hard to reintroduce.** Add a source-scanning test, in the
same style as RFC-071's coverage test: outside `draft_sync` and test code, no
file in `crates/app/src` or `crates/components/src` calls
`commit_focused_body`. Put it in `omriss-components`, so it runs in the plain
`cargo test` as well. The bug class here is "a second, unguarded copy of the
commit sequence", and a test that fails when one appears is the cheapest
permanent fix.

Checked while writing this: `on_navigate` re-reads the draft from the newly
focused section right after committing, so RFC-069 §2.1's resync symptom does
**not** survive on this path. The problem is the missing format check, as the
RFC says — nothing more.

## 3. §2.5 — a typed request instead of status sentinels

The Document Map asks the root component to open modals by writing sentinel
strings into the **user-visible status signal**, and the root matches them and
writes signals **during render** (`app.rs:192-272`).

**The fix:**

- A `StructureRequest` enum in `omriss-components` — it is a contract between
  the Document Map and its host — with the five live variants: add top-level,
  add inside, add after, rename, delete.
- A `Signal<Option<StructureRequest>>` passed down the way `status` is now.
- The root consumes it in a `use_effect`, never in render, and sets it back to
  `None` once handled.
- `status` goes back to being display-only.

**Delete the `struct.split.pending` branch.** RFC-069 §2.5 calls it a
conflation with add-inside; it is actually dead — five sentinels have producers
and split has none. Do not invent a `Split` variant to preserve it.

## 4. Constraints

1. No behaviour change visible to a user, except that a hash collision can no
   longer hide unsaved changes.
2. `omriss-core` does not change.
3. No new crate dependency.
4. The window-close guard (§2.2) is out of scope. Do not work around the
   framework; RFC-069 §2.2 explains why a workaround is worse than the gap.

## 5. GUI verification

Use `scripts/gui/`, and run `bash scripts/gui/selftest.sh` first. The relevant
behaviour to drive: each of the five Document Map actions still opens its modal
(row menu → Rename, Delete; the creation strip → Top, Inside, After), and
cancelling each leaves the document unchanged by its sha256.
