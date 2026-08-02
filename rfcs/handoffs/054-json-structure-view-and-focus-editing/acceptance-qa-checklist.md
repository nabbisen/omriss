# RFC-054 Acceptance / QA Checklist

Record `Pass`, `Fail`, or `Blocked`. **Unchecked boxes are not evidence.**

## Per-slice gates

```text
[ ] cargo fmt --all --check                                 clean
[ ] cargo test --workspace                                  counts only grow
[ ] cargo clippy --workspace --all-targets -- -D warnings   exit 0
[ ] bash scripts/check-rfcs.sh                              passes
```

Baseline: **299 passed, 12 suites.**

## J1 — replace_range

```text
[ ] undo restores byte-exactly after a replace_range
[ ] revision increments by exactly one
[ ] a stale base_revision is rejected
[ ] the call site routing into the shipped history path is named in the review
[ ] all 239 baseline tests pass UNMODIFIED
[ ] structural_ops* suites byte-identical
```

## J2 — JSON projection

```text
[ ] objects, arrays, scalars, nesting, empty containers
[ ] duplicate keys appear separately, each with its own identity and range
[ ] malformed JSON returns a typed error; no panic
[ ] JSONC input (comments, trailing commas) is REJECTED
[ ] rebuild determinism: same source, same ids
[ ] identity stable under an unrelated edit
[ ] nothing calls JsonAdapter yet
```

## J3 — session wiring (first user-visible slice)

```text
[ ] a .json file opens and renders a Document Map
[ ] a malformed .json preserves source and offers plain file text
[ ] .md behavior is unchanged — open, edit, organize, save, undo
[ ] document.revision() is passed at build time, not cached
[ ] the review request states what a user can now see
```

**Rendering check — required outcome, flexible evidence.** Rewritten after the
J3 review, where this item was written as "open a real `.json` and a real `.md`
in the same session" and could not be performed: synthetic input (`xdotool`)
did not reach the WebView at all, proven by a *visibly focused* button not
responding to `Return`.

What must be established, in priority order of evidence:

```text
[ ] A JSON-derived Document Map actually RENDERS — not merely that
    document_map_nodes() returns the right tree, which unit tests already cover.

    Acceptable evidence, best first:
      1. a click-through in a working GUI environment;
      2. a screenshot of the app with a .json file already open, if the app can
         be launched into that state without synthetic input (see below);
      3. code inspection plus unit tests, with this item recorded as
         NOT PERFORMED — never as passed.

[ ] Markdown is unaffected. This half does NOT need the GUI: it is closed by
    zero diff under crates/core/, the Markdown arm being pre-existing code
    unmoved, and the baseline suite passing unmodified. Verify it that way.
```

**Why option 2 does not exist yet.** `crates/app/src/main.rs` takes no
command-line arguments, so there is no way to launch omriss with a file already
open. Every path to a rendered document requires a click. That is what makes
this item unperformable when input synthesis is unavailable — the cause is a
product gap, not a harness one. Raised with the owner; if a file argument is
ever added, option 2 becomes the cheap default and this item stops being
fragile.

**Never record this item as passed on the strength of unit tests.** They
exercise `omriss-ui`, which is exactly the layer that cannot fail this check.

## J4 — message table

```text
[ ] every StructureErrorKind has a message in en
[ ] every StructureErrorKind has a message in ja
[ ] the mapping is exhaustive — a new variant fails the build or a test
[ ] omriss-core names no catalog key
```

## J5 — scalar editing

```text
[ ] text, number, on/off, empty validation per RFC-054 §8
[ ] invalid drafts block save with plain guidance
[ ] BYTE PRESERVATION: only the edited value's bytes change
[ ] indentation, key order, and line endings untouched
[ ] a byte-level before/after appears in the review request
[ ] docs/src/file-formats.md moves JSON to "Supported" — in THIS slice, not earlier
```

## J6 — container raw editing

```text
[ ] invalid container text is rejected without mutation
[ ] a valid replacement preserves surrounding bytes
```

## Scope discipline

```text
[ ] no JSONC tolerance anywhere
[ ] no TextDocument extraction
[ ] no global "experimental formats" setting
[ ] duplicate keys never merged
[ ] no null type-changing
[ ] no add/delete/rename/move operations (Phase 4 is out of scope)
[ ] no mod.rs; no file over 500 ELOC
```

## Final decision

```text
Decision:        Pending / Accepted / Corrections required
Date:
Recorder:
Source identity:
Deferred items:
```
