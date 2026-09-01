# RFC-064 Acceptance / QA Checklist

## Escaping

- [ ] `<img src=x onerror="...">` renders as text, not as an element
- [ ] `<iframe src="...">` renders as text
- [ ] `<script>` renders as text
- [ ] `<a href="x" onmouseover="...">` loses the handler
- [ ] `srcdoc` covered

## Link schemes

- [ ] `javascript:` destination dropped, link text kept
- [ ] `data:text/html` destination dropped
- [ ] `http`, `https`, `mailto` and relative paths still work

## CSP

- [ ] CSP set on the WebView
- [ ] A test fails if it is absent
- [ ] `connect-src 'none'` present
- [ ] App styles still render correctly under it

## Preservation

- [ ] Source text byte-identical after a preview render
- [ ] Saving a document containing raw HTML round-trips unchanged

## Documentation

- [ ] `known-limitations.md` records the behaviour change
- [ ] `CHANGELOG.md` entry written
- [ ] Any changed preview test flagged as scaffolding, with reasoning

## Hygiene

- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace` — count recorded
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
