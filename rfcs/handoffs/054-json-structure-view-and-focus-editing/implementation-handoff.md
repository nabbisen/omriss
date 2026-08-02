# RFC-054 Implementation Handoff

**Governing RFC:** [RFC-054](../../proposed/054-json-structure-view-and-focus-editing.md)
**Milestone:** M12 — Structured Format Support
**Supersedes:** the RFC-054 stub, written during M10 when the adapter boundary
did not yet exist. Its own exit condition — "replace after RFC-053 is accepted"
— is met.
**Prerequisite:** RFC-053 implemented and disposed (`2ee0fa2`).

---

## 1. Purpose

Make JSON the first non-Markdown format omriss can open, navigate, and edit —
through the adapter boundary RFC-053 built, without weakening the
source-preservation guarantees that boundary exists to protect.

**This is the first work in the sequence that changes what users see.** Every
RFC-053 slice was contract-bound to change nothing observable. That constraint
ends here: `.json` files become openable, and adapter code becomes reachable
from the running app for the first time.

## 2. Read RFC-054 §0 before planning anything

RFC-054 was written before the adapter existed. §0 records what RFC-053 actually
left ready, and one prerequisite that blocks all JSON mutation:

**`apply_validated_edit` and `structure_command` take `&mut Document`, which is
Markdown-specific.** A JSON adapter cannot use it — it needs an arbitrary
byte-range replacement recorded in undo history with the revision incremented as
one unit. §0.1's decision is to add a public `Document::replace_range`, **not**
to extract a `TextDocument` core. The rejected alternative and its revisit
trigger are recorded there; do not re-open it.

§0 also carries three items from RFC-053 into this work: session wiring for
parse-failure recovery (§0.2), the friendly-message table (§0.3), and the
revision discipline (§0.4).

## 3. Applicable requirements

- RFC-054 in full; §0 is the entry point.
- RFC-053 — the adapter boundary, implemented. Treat its trait as fixed.
- RFC-052 §5.1 (extension mapping), §5.2 (`PlainText` policy), §14.1 (**strict
  JSON only**), §14.4 (**JSON visible by default**, owner decision).
- RFC-001 crate boundaries. RFC-033 render boundary. RFC-043 i18n.

## 4. Slices

Six, in order. J1 is a prerequisite that touches shipped code and contains no
JSON at all; J2–J6 build the format on top of it.

| Slice | Content | User-visible? |
|---|---|---|
| **J1** | `Document::replace_range` — the format-neutral mutation primitive | No |
| **J2** | JSON parse + structure projection, read-only; `JsonAdapter::build_structure` replaces the stub | No |
| **J3** | Session wiring: open a `.json` file, render its Document Map, fall back to plain file text on parse failure | **Yes — first visible change** |
| **J4** | `StructureErrorKind` → message table in `omriss-ui`, with `en`/`ja` keys | Yes |
| **J5** | Scalar value editing with validation (RFC-054 §8) — `omriss-core` only | No |
| **J6** | Container raw focused editing (RFC-054 §7.4) — `omriss-core` only | No |
| **J7** | App wiring: `EditorSession` focus, right-panel editors, `DraftState`, save/undo | **Yes — JSON becomes editable** |

The J5/J6 rows previously read "Yes" while their detail sections listed
core-only tests. Settled after the J5 scope question: **J5 and J6 are
`omriss-core` only; J7 is the slice a user can see.** Separating the
byte-preservation-critical adapter logic from a new Dioxus component keeps each
reviewable on its own terms, and keeps the preservation proof from depending on
GUI verification — which has failed once and succeeded once in this
environment.

RFC-054 §13 Phase 4 (add/delete/rename/move) is **out of scope for this
handoff**. §15's resolution of question 4 explains why: those operations
synthesize punctuation, which is where the real preservation risk lives. If
wanted, they get their own RFC detail and their own handoff.

**J1 is the risk concentration point** — the only slice that modifies shipped
byte-preservation code.

## 5. Non-change scope

1. **No JSONC.** Strict RFC 8259 (RFC-052 §14.1). A `.json` file with comments
   is invalid, reported plainly, with the plain-file-text escape hatch. Do not
   add tolerant parsing "just for convenience."
2. **Do not extract a `TextDocument` core.** §0.1 decided against it, with a
   recorded revisit trigger at RFC-055.
3. **Do not change any shipped Markdown behavior.** The 239 baseline tests and
   the `structural_ops*` golden suites remain unmodified. This has held through
   six RFC-053 slices; J1 is where it is most at risk.
4. **Do not change Markdown node-id assignment** (RFC-053 §13.2).
5. **Do not introduce a global "experimental formats" setting** (RFC-052 §14.4).
6. **Do not merge duplicate JSON keys** (RFC-054 §15.2). Both appear, each
   independently editable.
7. **Do not implement type-changing of null values** (RFC-054 §15.3) — that is
   a structure change, not a value edit.
8. No `mod.rs`. Files over 500 ELOC get split; consider splitting at 300.

## 6. Design constraints

**J1's `replace_range` must route through the same internal path the shipped
section operations use.** If it bypasses that path, an edit can land without an
undo entry or without incrementing the revision — the exact failure RFC-053 §7.1
exists to prevent, and one that no JSON test would catch.

**Adapters own no state.** They receive `&str` plus the live revision, and
mutate only through `Document`.

**The session passes `document.revision()` at the moment of building**
(RFC-053 §7.0). Never cache it. A stale revision surfaces as a rejected command
— a safe failure, but a confusing one to debug.

**Node identity** is JSON-pointer-like paths (RFC-054 §6), with an ordinal
distinguishing duplicate keys. RFC-053 §13.1's rules apply — deterministic, and
stable under unrelated edits — and both need tests per §13.3.

**`omriss-core` must never name a catalog key** (RFC-001). J4's table lives in
`omriss-ui`, matching the `CapabilityReason` precedent exactly.

## 7. Required tests

Per slice, plus RFC-054 §12's own requirements:

- **J1** — `replace_range` records undo and increments revision; a stale
  `base_revision` is rejected; shipped section operations behave identically.
- **J2** — projection for objects, arrays, scalars, nesting, empty containers,
  and **duplicate keys**; malformed JSON returns a typed error without
  panicking; identity determinism and stability (RFC-053 §13.3).
- **J3** — opening a `.json` file produces a Document Map; a malformed one
  preserves source and offers plain file text.
- **J4** — every `StructureErrorKind` maps to a key present in both `en` and
  `ja`. An unmapped variant must fail the build or a test, never fall back
  silently.
- **J5** — RFC-054 §8 validation rules per value kind; **byte preservation**:
  editing one value changes only that value's bytes, leaving indentation, key
  order, and line endings untouched.
- **J6** — container replacement validates before applying; invalid input is
  rejected without mutation.

Baseline: **299 passed, 12 suites.** Counts may only grow.

### Which tests may never change, and which expire by design

**Refined after the J2 review**, where two tests legitimately had to change.
"No shipped test may be modified" was one rule doing two jobs; it is two:

| Category | Rule |
|---|---|
| **Protected** — the 239 Markdown baseline and the `structural_ops*` golden suites | **Never modified.** A change here means the slice is wrong and the test is not the thing to fix. RFC-053 §19 criterion 5's discipline, unchanged. |
| **Scaffolding** — tests asserting "X is not implemented yet" | **Expire by design** when X lands. Changing them is expected. |

A scaffolding test may only be changed when all three hold:

1. the property it proves is preserved, or the test is genuinely obsolete
   because its subject no longer exists;
2. the change is narrowly scoped to the expired premise — nothing else in the
   file moves;
3. it is **flagged in the review request**, with the reasoning, not slipped in.

The distinction is about protecting the *oracle*. A test changed to accommodate
a defect is always wrong. A test whose premise this very slice was mandated to
invalidate is a different thing, and pretending otherwise would mean either
lying in the test or leaving the suite red.

J2's two changes are the worked example: one test asserting `JsonAdapter`
refuses everything (removed — its premise was J2's mandate), and one whose
fixture was *accidentally* invalid JSON rather than *genuinely* invalid
(fixture corrected; name, structure, and proved property unchanged — and the
test is now stronger, exercising real validation instead of a stub's blanket
refusal).

## 8. Acceptance criteria

1. `.json` opens and renders as a hierarchy; `.md` behavior is unchanged.
2. Malformed JSON preserves source text and offers plain file text — closing
   RFC-053 criterion 7's end-to-end half.
3. Every `StructureErrorKind` has a localized message in `en` and `ja` —
   closing RFC-053 criterion 10.
4. Scalar edits validate before save and preserve unrelated bytes exactly.
5. Duplicate keys are displayed and edited independently.
6. No JSONC tolerance anywhere.
7. All 239 baseline tests pass unmodified; `structural_ops*` untouched.
8. Gates green:

```sh
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check-rfcs.sh
```

## 9. Documentation updates

- `docs/src/file-formats.md` — **only once JSON actually works**, which is
  **J7**, not J5. RFC-052 §12's two-column table exists for exactly this: move
  JSON from "Planned — not available yet" to "Supported" when a user can open
  *and edit* a JSON file. J5 and J6 add adapter logic no user can reach;
  promoting on those would be the precise overclaim RFC-052 exists to prevent.
  (This line named J5 before the scope question settled J5/J6 as core-only.)
- `docs/src/architecture.md` — describe the adapter boundary and JSON's place in
  it, after J3.
- `CHANGELOG.md` — per slice. J3 and J7 **are** user-facing; say so plainly
  rather than reusing the "no user-facing change" phrasing from RFC-053's
  slices.

## 10. Known risks

| Risk | Severity | Mitigation |
|---|---|---|
| J1 bypasses the history path, so edits land without undo | **High** | §6; J1 tests assert undo and revision explicitly |
| J1 regresses shipped Markdown editing | **High** | golden suites unmodified; criterion 7 |
| JSON byte-preservation drifts — reformatting, reordered keys, normalized whitespace | **High** | §7 J5; full-file reserialization is prohibited (RFC-052 §4.1) |
| Documentation promotes JSON before it works | Medium | §9 |
| Duplicate keys silently merged | Medium | §5.6, §7 J2 |
| A later `StructureErrorKind` variant has no message | Medium | §7 J4 — make the mapping exhaustive so the compiler catches it |

## 11. Required review-request content

Per organization workflow §9.2, plus per slice:

1. the §8 acceptance criteria table with per-row status;
2. per-suite counts against the 299 baseline, **counted, not estimated**;
3. for J1 — evidence that `replace_range` routes through the shipped history
   path, with the call site named;
4. for J5 — a byte-level before/after showing that only the edited value's bytes
   changed;
5. from J3 onward — what changed that a **user** can see. These slices are no
   longer inert, and the review needs to know what to look at. The
   "zero user-visible change" framing from RFC-053 does not apply and must not
   be reused.

## 12. Escalation triggers

Stop and file a clarification request if:

- a shipped test must change to pass;
- `replace_range` cannot be implemented without restructuring the history
  machinery — that would mean §0.1's decision was wrong and needs revisiting,
  not working around;
- JSON preservation cannot be achieved without reserializing the file;
- a slice cannot be completed without touching another slice's scope.
