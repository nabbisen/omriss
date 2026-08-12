# RFC-054: JSON Structure View and Focus Editing

**Project:** omriss — Omriss Editor
**Milestone:** M12 — Structured Format Support
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-053 (Implemented — see §0 for what it did and did not leave ready)
**Related RFCs:** RFC-055, RFC-056

---

## 0. Inherited state — read before anything else

**Added after RFC-053's disposition.** This RFC was written before the adapter
boundary existed. RFC-053 is now implemented, and it hands RFC-054 three items
plus one decision. None of them are optional preliminaries — the first is a
prerequisite for writing any JSON mutation at all.

### 0.1 The mutation signature is Markdown-specific — resolve this first

RFC-053 §7 specifies `apply_validated_edit(&mut Document, …)` and
`structure_command(&mut Document, …)`. `Document` is the canonical **Markdown**
model (RFC-002/006/007): it owns a heading `Outline`, and its public edit
operations are section-shaped (`replace_section_body(node_id, …)`).

That is correct for `MarkdownAdapter` — wrapping those operations is why S4b
preserved every byte-preservation guarantee. A JSON adapter cannot use it: it
needs to replace an arbitrary byte range and have the change recorded in undo
history with the revision incremented as one unit. `Document` does this
internally; it does not expose it.

**Decision: add a public, format-neutral replacement operation to `Document`.**

```rust
pub fn replace_range(
    &mut self,
    range: ByteRange,
    text: String,
    base_revision: DocumentRevision,
) -> Result<EditResult, EditError>;
```

It must route through the same internal path the section operations already use,
so history and revision update together (RFC-053 §7.1). JSON rides on `Document`
and simply ignores the heading outline that `Document::parse` builds over JSON
source — wasted work, conceptually untidy, functionally inert.

**Rejected alternative: extract a format-neutral `TextDocument` core** and make
Markdown's `Document` a wrapper over it. That is the better long-term
architecture and should be revisited once TOML (RFC-055) gives a second
non-Markdown case. It is rejected *now* because it restructures RFC-002/004/008/044 —
the code every byte-preservation guarantee depends on — speculatively, before a
single non-Markdown format exists to show what the abstraction needs. M11
succeeded by not touching that machinery; generalizing it on one hypothetical
case would spend that safety for nothing.

Revisit trigger: when RFC-055 lands, two real non-Markdown formats will exist.
If both are still ignoring a Markdown outline to get at a byte-range splice,
extract the core then, with evidence.

### 0.2 Carried from RFC-053 — criterion 7's end-to-end half

RFC-053 proved the parse-failure mechanism at the adapter boundary: a failed
`build_structure` cannot mutate source, and `PlainTextAdapter` supplies a
fallback structure. **Nothing wires it.** The session choosing that fallback, and
the UI offering "Show plain file text" when a JSON file will not parse, is
RFC-054's work — and it is the first time any adapter code becomes reachable
from the running app.

### 0.3 Carried from RFC-053 — criterion 10, the message table

`StructureErrorKind` production is complete; the kind → friendly-message mapping
was never built. Its home is fixed as `omriss-ui` (RFC-053 §11), matching the
`CapabilityReason` precedent: `omriss-core` must never name a catalog key. JSON
is the first format that will actually show these messages to a user, so
RFC-054 builds the table. New catalog keys are expected here, in both `en` and
`ja`.

### 0.4 The revision discipline (RFC-053 §7.0, S6)

`build_structure` takes the live revision explicitly:

```rust
adapter.build_structure(document.source(), document.revision())
```

The session must pass `document.revision()` **at the moment of building**, never
a cached value. Getting this wrong produces a rejected command rather than a
corrupted document — a safe failure, but a confusing one. This is the single
most likely wiring mistake in §0.2's work.

### 0.5 Binding decisions from RFC-052

- **Strict JSON only (RFC 8259)** — §14.1. No JSONC, no comments, no trailing
  commas. A `.json` file containing comments is reported as invalid with a
  plain-file-text escape hatch, not silently reparsed. This supersedes §15
  question 1 below.
- **JSON is visible by default** once this RFC is accepted — §14.4, owner
  decision of 2026-07-30. No global "experimental formats" toggle exists or is
  to be introduced.

## 1. Summary

This RFC defines JSON support as the first non-Markdown structured plain-text format for omriss.

Initial JSON support should be delivered in two stages:

1. **Read-only structure view:** open `.json`, build a Document Map, select nodes, and show focused content.
2. **Focused value editing:** safely edit selected values or raw selected containers without reformatting the whole file.

JSON is chosen first because it has a strict grammar, a clear object/list/value hierarchy, and no standard comments to preserve.

## 2. Goals

- Open standard `.json` files.
- Build a Document Map from JSON object, list, and value hierarchy.
- Let users select a JSON node and inspect its focused content.
- Support safe focused editing for scalar values.
- Support conservative raw focused editing for objects/lists when validation succeeds.
- Preserve source text outside the edited node.
- Avoid full-file reserialization during normal save.
- Use plain-language labels for non-technical users.

## 3. Non-goals

- JSONC support is not included.
- Comments in JSON-like files are not supported by this RFC.
- Schema validation is not included.
- Automatic formatting of the whole file is not included.
- Full visual form generation is not included.
- Drag-and-drop reordering of array items is not included in the first implementation.
- Network schema fetching is prohibited.

## 4. User experience

### 4.1 Open JSON file

When opening a valid JSON file:

```text
Document Map

▾ root
  ▾ package
    name
    version
  ▾ dependencies
    serde
    pulldown-cmark
```

The right panel shows the selected item.

### 4.2 Plain labels

| JSON concept | omriss label |
|--------------|--------------|
| object | group |
| array | list |
| property/key | name |
| string | text |
| number | number |
| boolean | on/off |
| null | No value |

### 4.3 Selected group

```text
package

This group contains 2 items.
Use the Document Map to choose an item.

[Show this part as text]
```

### 4.4 Selected text value

```text
name

Text
[ omriss ]
```

### 4.5 Selected number value

```text
version

Number
[ 3 ]
```

If invalid:

```text
This number is not valid yet.
```

> **Corrected after the J7b review.** Both mockups previously showed a
> `[Done]` button. That contradicts binding, shipped design: **RFC-048 §9.3** —
> "The UI must not expose a primary Done action for this lifecycle" — and
> **RFC-050 §9** — "omriss uses apply-on-navigation with a single user-visible
> dirty state. There is no primary `Done` action." Both are Implemented
> (v0.16.0); this RFC was drafted before them and was never reconciled.
>
> A focused value applies on navigation, save, preview, search, or blur, exactly
> as a Markdown section body does. `DraftState` (RFC-053 §9.3) blocks navigation
> while the draft is invalid — which only makes sense *because* navigation is
> the commit trigger; a `[Done]` button would render that property inert.
>
> **RFC-055 faces the identical question for TOML values. The answer is the
> same: no Done control.**

### 4.6 Selected on/off value

```text
enabled

[ On ] [ Off ]

[Done]
```

### 4.7 Selected empty value

```text
description

This is empty.

[Change to text] [Change to number] [Change to on/off]
```

Type-changing controls are optional for the first editable version. If not implemented, show:

```text
This is empty.
Use Show plain file text to change its type.
```

## 5. JSON structure model

The adapter should produce nodes for:

- root document;
- object;
- object property;
- array;
- array item;
- scalar value: string, number, boolean, null.

Recommended map behavior:

```text
object property with scalar value → one row named by key
object property with object value → group row named by key
object property with array value  → list row named by key
array item scalar                 → row named “Item 1”, “Item 2”, etc.
array item object/list            → row named “Item 1”, etc., with children
```

## 6. Node identity

Use JSON-pointer-like paths:

```text
/                         root
/package                  object property
/package/name             scalar
/dependencies/serde       scalar
/items/0                  array item
/items/0/title            scalar under array item
```

### 6.1 The duplicate-key tension, and what identity actually derives from

**Settled after the J2 review.** As originally written, §6 and §15.2 contradict
each other: a literal key-based path gives both members of
`{"a": 1, "b": 2, "a": 3}` the identity `/a`, which either collides or forces
the merge §15.2 forbids.

**The settled answer, matching RFC-053 §13.2's resolution of the identical
problem for TOML:**

- **array items** — identified by index. An array is ordered; index *is*
  identity, and inserting before an item genuinely changes which item it is.
- **object members** — identified by **key plus an occurrence ordinal** among
  same-named siblings (`a[0]`, `a[1]`). A member's identity is its key, not its
  position: adding an unrelated sibling does not change what `"version"` is.

Implementations must keep derivation deterministic across rebuilds
(RFC-053 §13.1). If a key is hashed to reach a `NodeId`, the hash must be
fixed-seed rather than a randomly seeded default.

**J2 shipped pure positional identity instead** — object members keyed by
position, not name. That is accepted as an interim, and is *correct within this
RFC's scope*: value edits never change sibling positions, so identity is stable
exactly where RFC-054 operates, and node ids are derived fresh on every
`build_structure` and never persisted, so nothing durable depends on the scheme.

**It must change before any structural mutation of objects exists** — §13's
Phase 4 (add, delete, rename, move), or any later RFC that inserts or removes
object members. Under positional identity, adding a member before a focused one
silently moves focus to a different node: the confusing kind of bug, not the
obvious kind. That trigger, not a date, is the deadline. Rework is deliberately
**not** required now, because nothing in J1–J6 exercises the difference.

The user must never see raw JSON pointer syntax in the normal UI. Breadcrumbs should use plain labels:

```text
root › package › name
```

## 7. Source preservation strategy

### 7.1 Canonical source

The original `.json` text remains canonical.

### 7.2 Structural index

The JSON adapter must produce byte ranges for selected values and containers.

A standard value parser alone is not enough if it loses source ranges. The implementation should use a JSON scanner / parser that can retain byte ranges, or pair validation with a lexical structure index.

### 7.3 Focused value replacement

For scalar values, replacement should touch only the selected value range.

Example:

Before:

```json
{
  "name": "old",
  "version": 1
}
```

Edit `name` to `new`.

After:

```json
{
  "name": "new",
  "version": 1
}
```

Unrelated bytes, including indentation and line endings, must remain unchanged.

### 7.4 Container raw editing

For object/list nodes, the right panel may offer `Show this part as text`.

If edited, the replacement must:

- parse as valid JSON;
- be valid in the selected location;
- preserve outside bytes exactly;
- refresh the Document Map after apply.

This is a power feature and may be deferred after scalar editing.

## 8. Validation rules

### 8.1 Text value

The UI may allow plain text entry and the adapter will JSON-escape it.

Example:

User enters:

```text
hello "world"
```

Adapter writes:

```json
"hello \"world\""
```

### 8.2 Number value

User input must be a valid JSON number. No leading plus sign, no NaN, no Infinity.

Friendly error:

```text
This number is not valid yet.
```

### 8.3 On/off value

The UI should use a toggle or two buttons. It must write `true` or `false`.

### 8.4 Empty value

For `null`, default view is read-only unless type-changing is implemented.

### 8.5 Object/list raw text

The edited text must parse as a valid JSON value and be appropriate to the selected node.

If the selected node is an object, replacing it with an array should be allowed only if the design explicitly supports type-changing. Conservative first implementation should preserve container type.

## 9. Structure operations

### 9.1 First editable version

The first JSON editable version should support:

- edit scalar value;
- edit raw selected object/list text, optional;
- delete property/item, optional only after confirmation;
- add property/item: deferred;
- rename property: deferred;
- reorder array items: deferred.

### 9.2 Later operations

Future JSON structure operations may include:

- add name/content inside group;
- add item to list;
- rename object property;
- delete property;
- delete array item;
- move array item up/down.

These must be implemented as source-preserving edits and should be added by follow-up RFC or amendment.

## 10. Error handling

| Cause | User-facing message |
|-------|---------------------|
| file parse failure | “This file does not look like valid JSON.” |
| invalid number | “This number is not valid yet.” |
| invalid raw value | “This text does not look like valid JSON yet.” |
| unsafe replacement | “This change could not be made safely.” |
| unsupported operation | “This JSON change is not supported yet.” |

**This table is not the `StructureErrorKind` mapping, and does not replace it.**
Recorded after J4, which built that mapping. Two layers exist:

- **Generic, kind-based** (RFC-053 §11, built in J4, lives in `omriss-ui`):
  one message per `StructureErrorKind`. Used wherever only the kind is known.
- **Context-specific** (this table, J5/J6): chosen at a call site that knows
  more than the kind does. "invalid number" and "invalid raw value" are separate
  rows here but are both `InvalidSyntax` at the kind level, so a kind-based map
  structurally cannot express them.

J5 and J6 must supply these strings at their own call sites, with their own
catalog keys. Reusing J4's generic message where this table specifies a
particular one is a defect, not a shortcut — it is the difference between
"This file does not look valid." and "This number is not valid yet."

## 11. Accessibility requirements

- Document Map rows must announce group/list/value kinds in plain language.
- Text fields must have visible labels.
- Number validation must be associated with the number input.
- On/off controls must expose selected state.
- Applying invalid edits must not move focus unexpectedly.

## 12. Testing requirements

### 12.1 Structure tests

- empty object;
- empty array;
- nested object;
- nested array;
- object in array;
- array in object;
- all scalar types;
- duplicate keys handling policy;
- strings with escapes;
- Unicode strings;
- deep nesting within limit.

### 12.2 Preservation tests

- edit string value preserves all outside bytes;
- edit number preserves all outside bytes;
- edit boolean preserves all outside bytes;
- edit nested value preserves all outside bytes;
- LF and CRLF are preserved;
- indentation style around edited value remains unchanged outside range;
- undo restores exact original file.

### 12.3 Error tests

- malformed JSON does not crash;
- invalid number is rejected;
- invalid container raw edit is rejected;
- stale node edit is rejected;
- unsupported operation returns friendly error.

## 13. Implementation phases

### Phase 1 — Read-only JSON tree

- detect `.json`;
- parse/validate JSON;
- build structure tree with node paths;
- render Document Map;
- render focused content summaries;
- malformed JSON opens friendly error/read-only source path.

### Phase 2 — Scalar editing

- string editing with escaping;
- number editing with validation;
- boolean editing with toggle;
- null read-only or type-change design;
- save/undo/redo integration.

### Phase 3 — Container raw focused editing

- show selected object/list text;
- validate replacement;
- apply range replacement;
- rebuild structure.

### Phase 4 — Optional structure operations

- delete property/item;
- add property/item;
- rename property;
- move array item.

Phase 4 may require additional RFC detail.

## 14. Acceptance criteria

- `.json` files can be opened without treating them as Markdown.
- Document Map shows JSON hierarchy with plain labels.
- Selecting a JSON node updates the right panel.
- Scalar string/number/boolean editing works safely.
- Invalid edits are rejected before changing source text.
- Normal save does not reformat the whole file.
- Unrelated bytes are preserved after focused edits.
- Undo restores exact prior source text.
- User-facing messages avoid technical parser details.

## 15. Resolved questions

Closed during RFC-054's pre-implementation review, so none reaches the
implementer.

**1. JSONC — no, and not as a future adapter either without new evidence.**
RFC-052 §14.1 already decided strict JSON (RFC 8259). JSONC reintroduces
comment preservation, which is the reason TOML is sequenced *after* JSON rather
than alongside it. Files such as `tsconfig.json` will report as invalid with a
plain-file-text escape hatch — the accepted consequence, recorded in RFC-052
§14.1. Revisit only on demonstrated demand, as its own RFC.

**2. Duplicate object keys — display all, edit each independently, never
merge.** RFC 8259 permits duplicates and does not define which wins. omriss must
not pick: the source is canonical, so both entries appear in the Document Map in
source order, each with its own node identity and its own byte range. Editing
one must not touch the other. This falls out of §6's ordinal-in-path identity
rule and needs a test, not a mechanism.

Deleting one of a duplicate pair is a structure operation and therefore Phase 4
at the earliest.

**3. Type-changing a null — no, not in the first editable version.** Editing an
"empty" value into a text, number, or on/off value changes the node's kind,
which changes its capabilities and its editor. That is a structure change
wearing a value edit's clothes. The first editable version keeps the rule
simple: a value edit may change a value, never its kind. A user who needs the
type changed uses "Show plain file text". Revisit in Phase 4 alongside the other
structure operations.

**4. Container raw editing before add/delete — yes, and deliberately.** §13's
phases already order it this way (Phase 3 before Phase 4), and that order is
correct: raw editing of an object or list is a *single validated replacement of
one byte range*, which is the same mechanism scalar editing already uses. Add
and delete require synthesizing punctuation — commas, separators, indentation —
which is where the real preservation risk lives. Ship the mechanism that reuses
proven machinery first.

Phase 4 remains explicitly optional for this RFC. If it is not reached, JSON is
still useful: view, navigate, and edit values.

## 16. Final decision summary

JSON is the first non-Markdown structured format target. Start with read-only structure view, then add focused scalar editing. Preserve source text outside edited ranges and avoid full-file reformatting.
