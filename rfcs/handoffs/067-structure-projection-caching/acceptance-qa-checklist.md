# RFC-067 Acceptance / QA Checklist

## Caching

- [ ] Structure cached on `EditorSession`, keyed on `DocumentRevision` alone
- [ ] Commit invalidates the cache
- [ ] Undo invalidates the cache
- [ ] A draft edit does **not** invalidate it
- [ ] Adapters remain stateless

## Outline skip

- [ ] `build_outline` not called for non-Markdown formats
- [ ] Markdown behaviour unchanged

## Measurements

- [ ] `document_map_nodes()` before/after recorded
- [ ] `focus()` before/after recorded
- [ ] `structured_draft_state()` before/after recorded
- [ ] Same method as the audit: release build, `Instant::now()`, comparable fixture
- [ ] Per-keystroke parse count is zero on an unchanged revision

## No behavioural change

- [ ] Every existing JSON suite passes **unmodified**
- [ ] `git diff --stat` on the test tree shows no modification

## Documentation

- [ ] `known-limitations.md` carries real numbers
- [ ] RFC-054 §0.4 amended
- [ ] `CHANGELOG.md` entry

## Hygiene

- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace` — count recorded
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `bash scripts/check-rfcs.sh`
