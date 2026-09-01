# RFC-066 Acceptance / QA Checklist

Tick only what was executed and observed.

## Properties

- [ ] P1 implemented and **fails** on current `main`
- [ ] P2 implemented and **fails** on current `main`
- [ ] Each failure shrunk to a minimal counterexample, recorded verbatim
- [ ] Each counterexample mapped to an audit finding, or flagged as new

## Generator

- [ ] Produces a document ending in a bare heading, no body, no trailing newline
- [ ] Produces documents with no trailing newline
- [ ] Produces skipped heading levels
- [ ] Produces empty bodies
- [ ] Produces CRLF documents
- [ ] Produces setext headings
- [ ] Produces headings with inline markup and link URLs
- [ ] Produces root-parented sections with following siblings
- [ ] Bias documented in a module comment

## Fuzz

- [ ] `cargo-fuzz` target over `scanner::parse` builds
- [ ] Documented run duration, no crash — or explicitly deferred with a reason

## Hygiene

- [ ] No file under `crates/*/src/` modified
- [ ] No existing test modified
- [ ] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Suite time before/after recorded
- [ ] `TESTING.md` property section added
