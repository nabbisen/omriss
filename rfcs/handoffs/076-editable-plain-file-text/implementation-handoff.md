# RFC-076 — Implementation Handoff

Related RFC: `../../proposed/076-editable-plain-file-text.md`

Lifecycle status: inherited from RFC-076.

---

## 1. What this is, and what makes it dangerous

The plain file text view becomes editable. The owner's position settles the
question of whether to do it: **omriss is an editor, so a user must be able to
edit a plain text file.**

What makes it unlike every other edit in the product: every existing mutation
is scoped — a body, a title, a subtree. This one replaces **the whole
document**, including every heading, in a single commit. The structure the user
navigates by can be entirely different afterwards, and the only way back is
undo.

So the three things that must hold are not negotiable: the commit is one
transaction, undo is byte-exact, and the user is told when the structure
changed.

## 2. Two things reading the tree changed

### 2.1 No new core API — `replace_range` already does this

RFC-076 §4.1 names `Document::replace_all(new_source, base_revision)`. **It
does not exist**, and it should not be written.

`Document::replace_range(range, text, base_revision)` (`document.rs:232`)
already does exactly what is needed: it checks the revision by strict equality,
routes through `apply_replacement` so history and revision update as one unit,
and is format-neutral by design — it was built for JSON by RFC-054 J1 precisely
so a mutation need not be heading-shaped.

A whole-document replacement is `replace_range` over `0..source.len()`.

**This means `omriss-core` does not change at all.** For the most dangerous
feature in the backlog, that is the single largest risk reduction available,
and it was available because someone else generalised the primitive a release
ago. Use it. If you find a reason it cannot serve, that is an escalation, not a
licence to add `replace_all`.

### 2.2 RFC-069 §2.1 is a hard prerequisite, and it is not in the scope table

This is my scoping error, the same shape as RFC-071's interpolation.

RFC-069 §2.1: `commit_or_block` never resyncs the draft signal after a
successful commit, so `is_pending` stays true forever. Its consequences, in the
RFC's own words:

- the dirty dot stays lit after a byte-correct save;
- **every navigation, preview toggle or search open re-commits an identical
  body** — a no-op that still bumps the revision and pushes an undo record, so
  **the next Ctrl+Z appears to do nothing**;
- Open right after a save falsely warns about unsaved changes.

Read that middle one against this feature. RFC-076 §4.2 returns focus to the
outline immediately after a raw commit — a navigation, on the path that
re-commits. And RFC-076's entire safety story is *undo is byte-exact and
available*. Shipping a whole-document replace on top of a gate that pollutes
the undo stack on navigation would undermine the one guarantee that makes the
feature acceptable.

**RFC-069 §2.1 is PR 1 of this item**, credited to RFC-069 and recorded in its
Status — the same arrangement as `t_args`. The fix is one call,
`draft_sync::sync`, at the three sites that skip it: `handle_save`,
`open_search_if_available`, and `AppCommand::TogglePreview`. `do_blur` already
does it correctly, which is why blurring clears the dot and saving does not.

## 3. The design that must not be softened

### 3.1 Focus returns to the outline. It is not remapped.

After a raw commit, focus goes to the whole-document outline. Not to the
section the user was in, not to a section with the same title, not to the
nearest match.

The temptation is obvious and the RFC refuses it for a reason worth repeating:
remapping is right most of the time and silently wrong the rest. A user who
rewrote their headings and lands in *a* section, unable to tell whether it is
the one they were in, has been misled by the application. Returning to the
outline costs one click and can never be wrong.

**Do not add a "cheap exact case" optimisation.** RFC-076 §4.2 contemplates one
and excludes it from this RFC.

### 3.2 The structure-change warning is mandatory here

RFC-071 landed in item 3 and this is its most important consumer. In a raw edit
a structure change is not an edge case — it is the normal case. The warning
must fire, with an accurate count, and it must say `Ctrl+Z`.

The detector lives in `omriss-ui` and brackets every source-rewriting
operation. A raw commit is one of those, and RFC-071's coverage test will fail
if it is not routed — which is the test doing its job.

**Expected delta for a raw commit is "anything may change".** That needs a
decision in `expected_change`: a raw commit cannot predict a delta, so it
should warn whenever the outline differs at all. Say in the review request how
you expressed that, because "no expectation" and "expects zero" are not the
same thing and only one of them is right here.

### 3.3 Two drafts must never be live at once

Entering the raw view with an uncommitted section draft, and leaving it with an
uncommitted raw draft, both go through `commit_or_block`. A section draft and a
raw draft of the same document cannot coexist: one of them is stale by
definition, and the user has no way to tell which.

### 3.4 Read-only stays reachable

RFC-017 built this view so a user could *inspect* bytes. Someone who opens it
to look should not be one keystroke from changing them. How that is expressed
is a design choice — an explicit edit affordance, a mode, something else — but
"it is now a textarea" is not an answer on its own.

## 4. Constraints

1. **`omriss-core` does not change** (§2.1). If you believe it must, stop.
2. No focus remapping (§3.1).
3. No new crate dependency.
4. The raw commit is **one** transaction and **one** undo step. A user who
   rewrites a document and presses Ctrl+Z once gets their document back.
5. Do not weaken RFC-071's coverage test to accommodate the new call site.
   Route it.

## 5. What this must not break

The existing read-only path is used by `RELEASE_CHECKLIST.md` check D and by
the JSON malformed-file recovery (step 20): *the app opens, the source is
preserved, the Document Map is empty, and "Show plain file text" still works*.
That is the last resort when a file cannot be parsed, and it must stay a last
resort that works.
