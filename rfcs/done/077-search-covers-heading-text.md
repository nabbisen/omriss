# RFC-077: Search Covers Heading Text

**Project:** omriss — Omriss Editor
**Milestone:** M13 — defect (0.18.0)
**Status.** Implemented (0.18.0) — `08da3e1`: Markdown heading titles and JSON keys are searchable in every scope, as their own result kind, on RFC-085's search (§8).
**Document type:** Defect design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-053
**Related RFCs:** RFC-067, RFC-059

---

## 1. Summary

Searching finds text inside section bodies and **not** the text of the headings
themselves. A user who searches for a heading title they can read in the
Document Map is told "No results".

**This RFC reclassifies that from a limitation to a defect.** It has been
documented as a limitation since v0.3.0, with no owning RFC anywhere in the
tree — found by the known-limitations schedule audit of 2026-11-03, and
accepted as a defect by the owner the same day.

## 2. Why it is a defect, not a limitation

A limitation is a capability deliberately not built. This is a search function
that silently excludes the most searchable text in the document.

The user-facing test is the one that matters: the application shows the user a
heading, the user searches for exactly that text, and the application says the
text is not there. It is not merely incomplete — **it answers a question
wrongly**, which is the line this project draws between a missing feature and a
defect.

The current workaround ("use the Document Map to navigate to a heading by
name") is a navigation instruction offered in place of a correct search result.

## 3. Cause

The search index covers section **body** ranges and excludes the heading line.
It is an indexing boundary, not a decision anyone recorded.

## 4. Design

### 4.1 Headings join the corpus

A section's heading title text is searchable. The match is a property of the
node, not of a body offset.

### 4.2 A heading match is a distinct result kind

Because a heading match has no body offset, it cannot be reported as one. The
result carries what it is — a heading match on node N — so that:

- activating it focuses that section, rather than seeking to a position inside
  a body it is not in;
- the result list can show it as what it is.

Conflating the two is how a correct index produces a wrong jump.

### 4.3 One section may match twice

A query matching both the heading and the body of the same section produces two
results, not a merged one. They are different places, and a user following the
second expects the body.

### 4.4 Setext headings

The title of a setext heading is its first line; the underline is not title
text and is not searchable. Searching `---` must not return every setext
heading in the document.

## 5. Non-goals

1. **No fuzzy matching, no ranking changes.** Same matching rules as today,
   applied to more text.
2. **No regular expressions.**
3. **No change to the search UI's shape** beyond §4.2's distinction.

## 6. Validation and test plan

- Searching a heading title visible in the Document Map returns that section —
  the exact case the limitation describes.
- A heading match activates to that section's focus, not to a body offset.
- A query matching heading and body in one section yields two results.
- `---` and `===` return no spurious setext matches (§4.4).
- Case handling matches body search exactly; no second set of rules.
- **Performance:** the corpus grows by the heading lines. RFC-067 §3.4's search
  hot loop is the place where that cost would show, so the two should be
  measured together rather than one after the other.

## 7. Acceptance criteria

1. Searching for a heading the user can see finds it.
2. Heading and body matches are distinguishable and both activate correctly.
3. No regression in body search results or ordering.
4. `docs/src/known-limitations.md` loses the entry.

## 8. On RFC-085's search (2026-11-27)

RFC-085 gave search one walk over every node, in every format, with each
node's own text cached by revision. This RFC adds titles to that walk, for
every format.

1. **A title is text from the file.** A Markdown heading's title, and a JSON
   object member's key (decoded, as the Document Map shows it). A list's
   members have **positional** labels ("Item 3") that omriss makes up. They are
   not in the file and are not searchable; otherwise "3" or "Item" would match
   every list. This is a rule of the structure model, not of JSON: a `List`'s
   members are positional in every format. Core's doc comment on
   `StructureNodeKind::List` says so; that comment is core's only change.
2. **A title match is its own result kind** (§4.2), on the same walk and the
   same cache. It opens its node, with no offset inside the text. In the result
   list it carries a visible word for what matched: "Heading" for Markdown,
   "Key" for JSON, in both locales. Not colour alone (RFC-080 §5.4 rule 1).
3. **Order:** results follow the document, and a node's title result comes
   before its text results (§4.3).
4. **Scope:** "Current Section" includes the focused node's own title and every
   title under it.
5. **Case and matching:** exactly the text search's rules, one function.

### Validation, added to §6

- JSON: a key is found; "Item" and an index number find nothing; a key and
  its value matching give two results, and both open the node.
- The "Heading" / "Key" word in the result list, both locales (Level 2).
- Measured on RFC-085's 8,000-item fixtures: search per call, before and
  after. Titles join the cache, so the change should be small; report it.

## 9. Accepted (2026-11-27)

- A title with several occurrences of the query is **one** result: a title is
  opened, not searched into.
- A Markdown title is what the Document Map shows (the flattened text), not
  its markup.
- Not this RFC's: a query matching thousands of places draws thousands of
  result rows. That belongs with RFC-085 §8.4's large structures.
