# RFC-078: Setext Heading Level Changes

**Project:** omriss — Omriss Editor
**Milestone:** M13 — editing completeness (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer
**Depends on:** RFC-053, RFC-065
**Related RFCs:** RFC-004, RFC-066, RFC-071

---

## 1. Summary

Setext headings — underlined with `===` or `---` — cannot be promoted or
demoted at all. This RFC makes the safe half of that capability work and
refuses the rest **for the real reason, stated to the user**.

**Source:** known-limitations schedule audit, 2026-11-03. Architect's analysis
accepted by the owner the same day.

## 2. The documented reason is not the real one

`known-limitations.md` says:

> ATX headings use a simple prefix change (`##` → `#`); Setext headings require
> replacing the underline character on a different line, which creates a more
> complex source range replacement.

That is an implementation-difficulty argument. It is also beside the point.

**The real constraint is that setext has only two levels.** `===` is H1, `---`
is H2, and there is no setext H3. That splits the feature cleanly in a way the
current all-or-nothing refusal does not:

| Operation | What it takes | Risk |
|---|---|---|
| H2 → H1, H1 → H2 | swap the underline character on one line | **none beyond an ATX prefix change** — no punctuation is synthesised, no boundary is crossed, the user's chosen style is preserved |
| H2 → H3 and beyond | rewrite `Title\n---` as `### Title` | **rewrites how the user wrote their heading** |

Today both are refused. The first refusal is not justified by anything.

## 3. Design

### 3.1 Within two levels: swap the underline

Promote H2 → H1 replaces the `---` run with `===`; demote H1 → H2 does the
reverse. **The run's length is preserved** — a heading underlined with twelve
dashes comes back with twelve equals signs. CommonMark ignores the length, so
this is pure fidelity to what the user typed, and it costs one `repeat`.

Nothing else on the line changes. In particular the **title line is not
touched**, which §5 turns into a test.

### 3.2 Beyond two levels: refuse, and say why

`Capability::Disabled { reason }` (RFC-053) already exists for this. The reason
must name the actual cause:

> Setext headings have only two levels. Going further would rewrite this
> heading in `#` style.

Today the user gets a refusal with no explanation, which teaches them nothing
and reads as a bug.

### 3.3 Style is never converted as a side effect

Converting `Title\n---` into `### Title` changes how the user wrote their
document. It must never happen as a consequence of asking for Demote.

If such a conversion is ever offered it is **its own action**, named for what it
does ("Convert to `#` style"), chosen deliberately. RFC-071 exists because
silent structure changes are the thing this project refuses to do to people's
files; silently restyling a heading is the same category.

## 4. What this changes about RFC-065 §2.9

**Read this before touching the tests.**

RFC-065 §2.9 fixed `promote`/`demote` overwriting a setext title whose text
begins with `#`. The fix was to refuse setext headings properly, and
`structural_ops_hash_word_promote.rs` asserts that refusal.

Under this RFC, promoting a setext **H2** no longer refuses — it swaps the
underline. So two of those tests change meaning:

| Today | Under RFC-078 |
|---|---|
| `promote` of a setext `#x` is refused and changes nothing | `promote` of a setext **H2** `#x` **succeeds**, swapping `---` → `===`, and **the title `#x` is byte-identical afterwards** |
| `demote` likewise refused | `demote` of a setext **H1** `#x` succeeds the same way |

The invariant those tests were written to protect — **a level change must not
touch the title text** — is unchanged and becomes the stronger assertion. The
refusal case remains for levels beyond two.

A test that is weakened rather than re-aimed would silently give back §2.9's
defect. That is the failure mode this section exists to prevent.

## 5. Validation and test plan

- H2 → H1 and H1 → H2 on a setext heading: the title line is **byte-identical**
  and only the underline changed.
- The underline's length is preserved.
- A setext heading whose title starts with `#`, `#x`, survives a level change
  unchanged — RFC-065 §2.9's case, re-aimed per §4.
- H2 → H3 is refused, and the reason reaches the UI.
- An ATX heading's behaviour is unchanged, including the indented `  ## x` case
  Task 023 pinned.
- RFC-066's P2 title-multiset clause holds across setext level changes; the
  generator already emits setext headings and leading-`#` titles.

## 6. Acceptance criteria

1. Promote and demote work for setext headings within H1/H2.
2. The title text is never modified by a level change.
3. Refusals beyond two levels carry the §3.2 reason.
4. No implicit style conversion exists anywhere in the code.
5. RFC-065 §2.9's tests are re-aimed, not weakened (§4).
6. `docs/src/known-limitations.md` is rewritten: the entry survives only for
   the beyond-two-levels case, with the true reason.
