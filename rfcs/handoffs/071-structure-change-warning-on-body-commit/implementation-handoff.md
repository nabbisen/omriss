# RFC-071 — Implementation Handoff

Related RFC: `../../proposed/071-structure-change-warning-on-body-commit.md`

Lifecycle status: inherited from RFC-071.

---

## 1. What this is

Text a user commits can change whether *other* sections are headings. Typing
`<!--` turns every following heading into comment content; typing `# Foo`
creates one. Both are correct Markdown, the bytes are preserved, and **nothing
tells the user**. RFC-004 promised this warning in v0.1.0 and it was never
built.

§2.1 widened it after 0.17.0: a **move** can do the same with no typing at all,
which is the sharper case, because there the user authored nothing — they asked
for a reordering and a heading elsewhere stopped being a heading.

## 2. Two things the RFC does not say

### 2.1 It needs interpolation, which 0.18.0 schedules *after* it

§4.2's messages carry a count — "2 sections are no longer headings" — and
`t()` cannot interpolate. That is RFC-073 §4.1's `t_args`, which 0.18.0 lists
as **item 9**, six places after this one. My ordering error.

**Resolution: RFC-073 §4.1 is the first PR of this item**, credited to RFC-073
and recorded in RFC-073's Status, not absorbed silently into RFC-071.
Interpolation belongs to RFC-073 and this is merely its first consumer. Item 9
then reduces to §4.3's typed failures.

Build `t_args` to RFC-073 §4.1 as written — named placeholders, `{count}` not
`{}` — including its enforced invariant that the placeholder *set* is identical
across `en` and `ja` for every key. That invariant is the reason to build it
properly now rather than inline a `format!` here and regret it in item 9.

### 2.2 The warning must fire on *unintended* change, not on change

This is the part that decides whether the feature is useful or maddening.

§2.1 extends detection to every mutation that rewrites the source. But
`delete_section` removes nodes **on purpose**. A detector that compares the
outline before and after would fire on every delete, every split, and every
promote — and a warning that fires when nothing is wrong is quickly ignored,
which costs us the one case it exists for.

So the rule is: **compare the actual outline delta against the delta the
operation itself intends.**

| Operation | Intended delta | Warn when |
|---|---|---|
| body commit | 0 nodes, titles unchanged | anything differs |
| `move_section` | 0 nodes, titles unchanged | anything differs |
| `split_section` | +1 node, no existing title changed | anything else differs |
| `delete_section` | −(subtree size), those titles gone | anything else differs |

RFC-066's P2 properties already encode exactly these expectations — they assert
the same deltas, operation by operation. The detector and the property suite
are two readings of one rule, and if they ever disagree one of them is wrong.
Say in the review request which function expresses it, so there is one.

## 3. Where each piece lands, after RFC-075

| Piece | Crate | Why |
|---|---|---|
| The detection (compare expected vs actual delta) | **`omriss-ui`** | §4.1 puts it in the session; it is renderer-independent and unit-testable there, with no WebView needed |
| The catalog keys and `t_args` | **`omriss-ui`** | i18n lives there (RFC-043) |
| Displaying the message | **`omriss-components`** | it is a status surface, and components now live in their own crate |

`omriss-ui` must still end with **one dependency**. `t_args` adds no crate.

## 4. The message

§4.2's three shapes, and they are three because one generic sentence would be
useless:

| Change | Shape |
|---|---|
| headings disappeared | "N sections are no longer headings" |
| headings appeared | "N new sections were added" |
| both | "This document's structure changed" |

**Non-modal.** The edit succeeded, the file is correct, and the user may have
meant it. A dialog would be wrong: it would interrupt a correct action and
train the user to dismiss it.

§4.3 adds one sentence for the disappearance case, because it is otherwise very
hard to diagnose: name the *class* of cause — an unclosed `<!--`, `<?`,
`<script>`, `<!DOCTYPE` or `<![CDATA[` earlier in the body. **Detecting which
one is not required**; saying what kind of thing does this is.

And the message must say `Ctrl+Z`, because undo is the whole remedy.

## 5. Constraints

1. **`omriss-core` must not refuse the edit.** It stores valid Markdown the
   user deliberately wrote. This is a warning, never a rejection — RFC-071 §5
   and the project's position since RFC-004.
2. **No modal.**
3. **No new crate dependency** anywhere.
4. Do not weaken RFC-066's P2 or P3 to match the detector. If they disagree,
   that is a finding, not an inconvenience.

## 6. A question to answer by construction

Every call site that rewrites the source must be routed through the detector,
or the warning silently misses cases — which is worse than not having it,
because the user learns to trust it. Enumerate them from
`crates/core/src/doc/structural/`'s public surface, not from the ones this task
happens to touch. RFC-065 §2.9's table is the model: list every one, say which
is covered, and why any is not.
