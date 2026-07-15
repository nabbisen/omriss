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

1. Run `cargo test -p omriss-core -p omriss-ui` on every pull request.
2. Run `cargo clippy --workspace -- -D warnings` to enforce lint hygiene.
3. Run `cargo fmt --check` to enforce formatting.
4. Fail if any test fails or any warning is present.

Benchmarks and platform smoke tests are optional in CI but must be run
before any public release.
