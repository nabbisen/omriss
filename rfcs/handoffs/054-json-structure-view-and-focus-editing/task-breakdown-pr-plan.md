# RFC-054 Task Breakdown / PR Plan

Six slices, strictly ordered, one review each. Read
`implementation-handoff.md` §5 (non-change scope) before the first commit, and
RFC-054 §0 before planning.

---

## J1 — `Document::replace_range`

**Touches shipped code. Contains no JSON.**

```rust
pub fn replace_range(
    &mut self,
    range: ByteRange,
    text: String,
    base_revision: DocumentRevision,
) -> Result<EditResult, EditError>;
```

Must route through the same internal path the shipped section operations use,
so history and revision update as one unit (RFC-053 §7.1).

**Tests:** undo restores byte-exactly after a `replace_range`; revision
increments by one; a stale `base_revision` is rejected; every shipped section
operation still behaves identically.

**Done when:** the primitive exists, is proven to record history, and all 239
baseline tests pass unmodified.

---

## J2 — JSON parse and structure projection (read-only)

Replace the `JsonAdapter` stub's `build_structure`. Strict RFC 8259 only.

Node identity is JSON-pointer-like paths (RFC-054 §6), with an ordinal
distinguishing duplicate keys.

**Tests:** objects, arrays, scalars, nesting, empty containers, duplicate keys;
malformed input returns a typed error without panicking; RFC-053 §13.3 identity
determinism and stability.

**Not in this slice:** any session wiring. Nothing calls `JsonAdapter` yet.

---

## J3 — Session wiring and parse-failure fallback

**The first user-visible slice.** Opening a `.json` file produces a Document
Map; a malformed one preserves the source and offers plain file text.

Pass `document.revision()` at the moment of building (RFC-053 §7.0) — never a
cached value.

Closes RFC-053 criterion 7's end-to-end half.

**Tests:** a `.json` file opens and renders; a malformed one falls back without
losing source.

---

## J4 — The friendly-message table

`StructureErrorKind` → localized message, in **`omriss-ui`** (RFC-053 §11).
`omriss-core` must never name a catalog key.

New keys in both `en` and `ja`. Make the mapping exhaustive so a future variant
without a message fails the build rather than falling back silently.

Closes RFC-053 criterion 10.

---

## J5 — Scalar value editing

Text, number, on/off, and empty values per RFC-054 §8's validation rules.
Invalid drafts block save and show plain guidance (RFC-053 §9.3).

**Tests:** validation per value kind, and **byte preservation** — editing one
value changes only that value's bytes; indentation, key order, and line endings
are untouched.

**When this lands,** move JSON from "Planned" to "Supported" in
`docs/src/file-formats.md` — and not before.

---

## J6 — Container raw focused editing

Show a selected object or list as text, validate the replacement, apply it as
one range replacement, rebuild.

**Tests:** invalid input is rejected without mutation; a valid replacement
preserves surrounding bytes.

---

## Out of scope

RFC-054 §13 Phase 4 — add, delete, rename, move. Those synthesize punctuation,
which is where the preservation risk concentrates (RFC-054 §15, question 4).
They need their own RFC detail and handoff.

---

## Sequencing summary

```text
J1 replace_range        ← touches shipped code; no JSON
  └─ J2 JSON projection ← read-only; nothing calls it
       └─ J3 session wiring       ← FIRST USER-VISIBLE SLICE
            └─ J4 message table
                 └─ J5 scalar editing   ← JSON becomes "Supported" in docs
                      └─ J6 container raw editing
```
