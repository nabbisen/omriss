# Testing Strategy and Regression Policy

This document describes the omriss test pyramid, the non-negotiable invariant
that all tests must protect, and the process for handling regressions.

---

## The Non-Negotiable Invariant

> **A committed edit to section S must not rewrite any byte belonging to a
> section S' where S' ≠ S and S' is not in the subtree of S.**

Every RFC that introduces an edit operation must add tests that verify this
invariant for the new operation. No release may ship with a known violation.

---

## Test Pyramid

```
──────────────────────────────────────────────────────────
 Few     Manual smoke tests per platform (RELEASE_CHECKLIST)
──────────────────────────────────────────────────────────
 Some    omriss-ui integration tests (session + view state)
         omriss app pure-logic tests (keyboard mapping, recent files)
──────────────────────────────────────────────────────────
 Some    omriss-core property tests (generated documents, RFC-066)
──────────────────────────────────────────────────────────
 Many    omriss-core golden fixture tests
         omriss-core structural operation tests
──────────────────────────────────────────────────────────
 Many    omriss-core unit tests (range, revision, history)
──────────────────────────────────────────────────────────
```

---

## Test Locations

| Suite | Path | What it tests |
|-------|------|---------------|
| Core unit | `crates/core/src/` (inline) | data structures, range arithmetic, UTF-8 boundaries |
| Source preservation | `crates/core/tests/source_preservation.rs` | golden byte-exact edit tests |
| Structural ops | `crates/core/tests/structural_ops.rs` | promote/demote/move/split/delete/merge + undo |
| Property tests | `crates/core/tests/properties/` | generated-document invariants over the non-negotiable invariant above (RFC-066) |
| Fixture catalog | `crates/core/tests/fixture_catalog.rs` | all fixtures load, outline is correct, round-trip edit preserves source |
| UI unit | `crates/ui/src/tests/` | i18n parity, session behavior, search, commands |
| Desktop pure-logic | `crates/app/src/input/keyboard.rs`, `crates/app/src/storage/settings.rs` | keyboard shortcut mapping, recent-files dedup/cap |
| Benchmarks | `crates/core/benches/indexing.rs` | performance regression detection (run manually) |

## On Dioxus testing styles

The Dioxus guide describes component testing (dioxus-ssr snapshots), hook
testing (a hand-driven `VirtualDom`), and end-to-end testing (Playwright).
This project uses none of them, by design. All business logic lives in the
Dioxus-free `omriss-core` and `omriss-ui` crates and is covered by plain
Rust tests, so the shell needs no component-level harness. There are no
custom hooks to test, HTML snapshots would be brittle against ordinary
markup edits, and Playwright targets web builds rather than this desktop
WebView. If a web or TUI target is ever added (see ROADMAP), end-to-end
testing should be reconsidered then.

---

## Fixture Catalog

Fixtures live in `crates/core/tests/fixtures/`. Each fixture must have
a brief comment explaining its purpose. New fixtures are added when:

- a reported bug involves a Markdown structure not covered by existing fixtures;
- a new operation is added that requires a realistic document;
- performance profiling identifies a new document shape of concern.

Current fixtures:

| Fixture | Purpose |
|---------|---------|
| `nested_atx.md` | Standard H1/H2/H3 ATX hierarchy |
| `duplicate_titles.md` | Identical heading titles at different levels |
| `japanese.md` | Multibyte UTF-8 text in titles and bodies |
| `code_fences.md` | Heading-like text inside fenced code blocks |
| `skipped_levels.md` | H1 → H3 without H2 |
| `no_trailing_newline.md` | File ending without a final newline |
| `setext.md` | Setext-style H1/H2 underlines |
| `yaml_front_matter.md` | YAML front matter block |
| `toml_front_matter.md` | TOML front matter block |
| `crlf.md` | Windows-style CRLF line endings |
| `html_content.md` | Inline HTML and HTML comments |
| `empty_bodies.md` | Sections with no body text |
| `no_headings.md` | File with no Markdown headings |
| `academic-paper.md` | Realistic academic paper structure (RFC-034) |
| `technical-rfc.md` | RFC-style document with code and tables (RFC-034) |
| `large-10k-words.md` | ~15 000 words for performance testing (RFC-034) |

---

## Property-Based Testing (RFC-066)

Every test above this line is example-based: it encodes a case a human
thought of. Three release-blocking defects in the 0.17.0 audit
(AUDIT-0170-002 through -005) each shipped past 431 example-based tests
because none of them thought of that exact shape — a heading that is the
last line of the file with no body and no trailing newline, in one case.
Properties encode the invariant itself and let a generator search for the
shape nobody imagined.

### When a property is required, not optional

A **new structural or content-editing operation** (anything that can change
the outline shape — promote, demote, move, split, delete, merge, and any
future operation in the same family) **must** be covered by
`crates/core/tests/properties/p2_structural_invariants.rs`'s node-count and
title-multiset pattern before it ships, in addition to its own example-based
tests. A **new body-addressing path** (anything that resolves a byte range
to read or write a section's content) **must** be covered by
`p3_body_addressing.rs`'s read-back pattern. Example-based tests alone are
not sufficient for either class of operation — that is the lesson of
AUDIT-0170-002 through -005, `delete_section` sharing the same defect
family via a call site RFC-065 §4.1 originally missed, and P1 itself
turning out not to catch AUDIT-0170-002 despite an earlier RFC-066 draft
claiming it did (see below).

A property is *optional* — nice to have, not blocking — for:

- pure read paths (nothing mutates, nothing to round-trip);
- UI-layer behavior (RFC-066 §4.3 scopes this to the layer that can be
  tested without a WebView — `crates/core`, not `crates/ui`/`crates/app`);
- a one-off fix to an already-covered operation, where the existing
  property already exercises the corrected code path.

### The three properties

```text
P1  For any document source S and any node N in its outline:
    replacing N's body with any string B, then undoing,
    reproduces S byte-for-byte.

P2  For any document source S and any structural operation O
    valid on node N: the outline node count after O differs from
    before by exactly the delta O defines, and every surviving
    node's title is unchanged unless O is rename or join.

P3  For any document source S and any node N: after replacing N's
    body with B, reading N's body back yields B, and no node's
    title has changed.
```

`P1` lives in `p1_replace_undo.rs`; `P2` is one `proptest!` function per
operation kind in `p2_structural_invariants.rs`, sharing a node-count-delta
helper and a title-multiset helper (multiset, not per-node identity — see
that file's module doc comment for why `NodeId`, being an ordinal-path hash,
is the wrong tool for "did this node's title survive"); `P3` lives in
`p3_body_addressing.rs`.

**Why three properties, not two — P1 does not catch AUDIT-0170-002, and
that was RFC-066's own error, not this test suite's.** `P1` is a
*reversibility* property; AUDIT-0170-002 is an *addressing* defect (a
section's `body_range` sits inside its own heading line, so a commit writes
the user's prose into the title). `undo` is byte-mechanical and faithfully
reverses whatever range was actually touched, so commit-then-undo is still
byte-reversible even when the commit wrote to the wrong place — both facts
are true at once, and `P1` is right to pass. `P3`'s read-back clause states
the invariant AUDIT-0170-002 actually violates: write `B`, read back `B`.
RFC-066 §3.1 carries the full trace and the architect's correction of the
original (wrong) claim. `P1` was deliberately **not** redefined to paper
over the error — a property whose English no longer matches what it checks
is worse than an honestly-scoped one that needs a sibling.

**`#[ignore = "fails until RFC-065"]`**: `p2_structural_invariants.rs`'s
promote/move/split/delete functions and `p3_body_addressing.rs`'s single
function all carry this attribute right now, because they are expected to
fail until RFC-065 lands — an unconditionally red `cargo test --workspace`
would destroy CI's signal for every unrelated change. The committed
`crates/core/tests/proptest-regressions/*.txt` seeds preserve each shrunk
counterexample regardless, so nothing is lost by ignoring. **RFC-065's
acceptance evidence is removing these attributes and the suite staying
green** — a sharper claim than "the properties pass," since it proves the
fix is what made them pass, not a coincidence of a different run.

### The generator

`crates/core/tests/properties/generator.rs` is a Markdown generator biased
toward the shapes that broke `structural.rs`, documented in its own module
comment: bare trailing headings with no body and no trailing newline
(elevated probability, not left to chance), no trailing newline generally,
skipped heading levels, empty bodies, CRLF, setext headings, inline
markup/link URLs in titles, and multiple root-parented top-level sections.
`generator::tests::generator_reaches_required_shapes` asserts each of these
is actually observed within a bounded number of draws — the generator's own
bias is a claim that must stay true, not just documentation.

### Tooling

[`proptest`](https://docs.rs/proptest), a dev-dependency of `omriss-core`
only. Shrinking is the reason: a failing 40-line generated document is not a
useful bug report on its own, and `proptest`'s shrinker reduces a failure to
a small reproducer, which is what becomes a permanent, named fixture-catalog
entry once the underlying defect is fixed. Default case count (256 per
property) runs in CI; a longer run is available manually via
`PROPTEST_CASES=<n> cargo test -p omriss-core --test properties`.

### Fuzzing

`cargo-fuzz` over `formats::json::scanner::parse` was scoped for this RFC
(§3.3) but deferred to 0.18.0, not part of the 0.17.0 ship gate: `scanner`
is a private module (`crates/core/src/formats/json.rs`, `mod scanner;`,
not `pub`), so an external `cargo-fuzz` target crate cannot call it
without a visibility change to a file under `crates/core/src/` — which
RFC-066's own implementation constraints forbid for this slice. Revisit
once a fuzz-friendly visibility boundary (a `#[cfg(fuzzing)]` re-export, or
moving the fuzz target inside the crate) is deliberately designed, not as a
drive-by visibility bump.

---

## Regression Policy

When a bug is reported or discovered:

1. **Reproduce** — write a minimal Markdown fixture or unit test that
   demonstrates the failure.
2. **Classify** — assign severity:
   - **Release blocker**: data corruption, silent save failure, crash on
     open/edit/save.
   - **High**: incorrect structural edit, undo produces wrong state.
   - **Medium**: UI glitch, wrong error message, missing keyboard action.
   - **Low**: cosmetic, documentation gap.
3. **Fix** — implement the fix after the test is in place.
4. **Keep the test** — regression tests are permanent unless explicitly
   superseded by a broader test that covers the same case.
5. **Note in CHANGELOG** — include the fixture/test name in the fix entry.

---

## Running Tests

```sh
# Default members (excludes the omriss app package, which requires GUI libraries)
cargo test

# Core only (fast, no GUI)
cargo test -p omriss-core

# UI only
cargo test -p omriss-ui

# Property tests only (RFC-066)
cargo test -p omriss-core --test properties

# App package check (requires platform WebView libraries)
cargo check -p omriss

# Benchmarks (optional, slow)
cargo bench -p omriss-core
```

The `omriss` app package is excluded from workspace default members
because it requires platform WebView libraries. It is tested manually via
the platform smoke test workflow in `RELEASE_CHECKLIST.md`.

---

## CI Requirements

A minimum CI configuration must:

1. Run `cargo test -p omriss-core -p omriss-ui` on every pull request,
   including `crates/core/tests/properties/` at the default `proptest` case
   count (RFC-066).
2. Run `cargo clippy --workspace -- -D warnings` to enforce lint hygiene.
3. Run `cargo fmt --check` to enforce formatting.
4. Fail if any test fails or any warning is present.

Benchmarks and platform smoke tests are optional in CI but must be run
before any public release.
