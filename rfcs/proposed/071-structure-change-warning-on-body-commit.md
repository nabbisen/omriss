# RFC-071: Structure-Change Warning on Body Commit

**Project:** omriss — Omriss Editor
**Milestone:** M13 — editing feedback (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-004, RFC-049
**Related RFCs:** RFC-017, RFC-039, RFC-050, RFC-065, RFC-066

---

## 1. Summary

Text a user types into one section's body can change whether *other* sections
are headings at all. Typing `<!--` and committing turns every following heading
into inert HTML-comment content until a closing `-->`; typing `# Foo` creates a
new heading. Both are correct Markdown. Neither is announced.

This RFC builds the warning RFC-004 promised in v0.1.0 and never delivered.

**Source:** found by RFC-066's P3 property during RFC-065's implementation, and
triaged as *not* a defect — see §2.

## 2. This is not corruption, and the distinction is the whole point

The bytes are preserved exactly. Verified:

```text
source  "# One\nbody\n\n# Two\nmore\n"      write "<!--" into One's body
after   "# One\n<!--\n# Two\nmore\n"
        "# Two" still literally present     undo byte-exact: true
        outline ["One"]                     "Two" is no longer a heading
```

And the mirror:

```text
write "# Injected" into One's body
outline ["One", "Injected", "Two"]          a node is GAINED
```

omriss stored what the user wrote and re-indexed correctly. CommonMark says an
unclosed HTML block runs to end of input regardless of blank lines, so `# Two`
genuinely is not a heading in that document any more.

**Compare AUDIT-0170-002**, which *is* a defect: there the user typed
`"typed text"` and *omriss's own splice* welded it into a heading the user never
touched. The source-preservation invariant is about bytes omriss changes on its
own initiative. Here it holds.

So this RFC adds feedback, not a refusal. Refusing would mean omriss cannot
store valid Markdown its user deliberately wrote — an unclosed comment spanning
sections is legal, occasionally intentional, and none of the editor's business
to forbid.

## 3. What RFC-004 promised

> *UI may later warn if the result visually collapses into a child heading, but
> core must not silently rewrite.*

Half of that sentence was amended on 2026-09-01 (RFC-065): core now inserts the
minimum separator needed to keep structure intact, because "visually collapses"
badly understated a heading ceasing to exist. **The other half — the warning —
was never built, and is still needed**, because the separator fix cannot help
here. A blank line does not close an HTML block.

## 4. Design

### 4.1 Detect, at commit, that the outline changed

`replace_section_body` already reports an `EditResult`. The session can compare
the outline's title multiset and node count across the commit, which is exactly
the check `source_preservation.rs` gained in RFC-065's follow-up.

The comparison is cheap and already computed for other purposes; no new parse.

### 4.2 Tell the user what happened, and let them undo

A non-modal status message, not a dialog. The edit succeeded, the file is
correct, and the user may well have meant it:

```text
Your text changed this document's structure — 2 sections are no longer
headings. Ctrl+Z to undo.
```

Three shapes, distinguished because the causes are different and a single
generic message would be useless:

| Change | Message shape |
|---|---|
| headings disappeared | "N sections are no longer headings" |
| headings appeared | "N new sections were added" |
| both | "This document's structure changed" |

Catalog keys in `omriss-ui` per RFC-043, with `en` and `ja`. The count is an
interpolated value — this is the second concrete need for interpolation after
J7A-IMPL-002, and the two should be solved together.

### 4.3 Make the cause discoverable

The disappearance case is worth one extra sentence, because it is otherwise very
hard to diagnose: an unclosed `<!--`, `<?`, `<script>`, `<!DOCTYPE`, or
`<![CDATA[` earlier in the body. Detecting *which* is not required; naming the
class is.

## 5. Non-goals

1. **No refusal, no auto-repair.** omriss does not close the user's comment for
   them, and does not reject the edit.
2. **No change to `omriss-core`'s behaviour.** This is a UI affordance over an
   existing, correct result. Core keeps storing what it is given.
3. **No live preview of structural consequences while typing.** At commit only —
   the outline is not rebuilt per keystroke, and RFC-067 exists to keep it that
   way.

## 6. Validation and test plan

- Committing `"<!--"` into a section followed by a heading produces the
  disappearance warning with the right count.
- Committing `"# Foo"` produces the appearance warning.
- Committing inert text produces no warning at all — the common case must stay
  silent, or the warning becomes noise and is ignored.
- Undo restores both the bytes and the outline, and clears the warning.
- Every message key present in `en` and `ja`.

## 7. Acceptance criteria

1. A body commit that changes the outline is announced; one that does not is
   silent.
2. The message distinguishes appearance, disappearance, and both.
3. Undo is offered and works.
4. No core behaviour changes.
5. RFC-004's deferred warning is recorded as delivered, and its header updated.
