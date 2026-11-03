# RFC-064 H3 — Acceptance and QA Checklist

Related RFC: `../../done/064-preview-html-sanitization-and-webview-trust-boundary.md`

Lifecycle status: inherited from RFC-064.

---

## 1. Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace` — `csp.rs`'s tests changed, so state the count
      and name each changed test with what it now asserts
- [ ] `bash scripts/check-rfcs.sh`
- [ ] `mdbook build docs`

## 2. The application still works — looked for, not assumed

Run under the GUI rules. Both failure modes are silent in a build log.

- [ ] **It is not blank.** The window renders a document
- [ ] **It is not unstyled.** Compare against a capture from before the change:
      same window size, same theme, same file. This is the one place in this
      task where a picture *is* the evidence, because "unstyled" is what it
      detects
- [ ] **The WebView console carries no CSP violation.** Read it; do not infer
      from the absence of a crash
- [ ] Edit a section, save, undo — the loopback websocket streams every UI
      update, so if `connect-src` were wrong this is where it would show
- [ ] A dialog opens and closes (the modal focus script runs through `eval`)
- [ ] Ctrl+F opens search, the palette opens — both use injected script

## 3. The policy does what it is for

- [ ] A Markdown file with `![x](https://…)` previews with the image
      **blocked**, and the block appears in the console as an `img-src`
      violation. That is RFC-064 §9a's tracking pixel, stopped
- [ ] A `data:` image still renders — the policy allows it deliberately
- [ ] Check I from `RELEASE_CHECKLIST.md`: raw HTML in a preview still shows as
      visible text. H1 is what guarantees this; confirm H3 did not disturb it

## 4. The documentation matches the behaviour

- [ ] `PRIVACY.md` §4 no longer says a preview may fetch remote images
- [ ] `CHANGELOG.md` records the loss of remote images under **Changed**
- [ ] `docs/src/known-limitations.md` has an entry naming `Owner: RFC-064`

## 5. Platforms

- [ ] Linux: all of §2 and §3
- [ ] **Windows: §2 only, by the owner, mid-release.** A WebView2 difference
      here is a blank or unstyled application in the Microsoft Store build,
      and discovering it during the final run would force a change after the
      scope freeze
- [ ] macOS: not run, and not claimed

## 6. Report content

1. Gate output, and each changed test with what it now asserts.
2. The §2 evidence, including the before/after styling capture and the console
   read.
3. The §3 evidence, including the `img-src` violation text.
4. The documentation diffs.
5. What was not run, or not proven.

## 7. If it cannot be made to work

Say so and stop. Leaving H3 unwired with the reason recorded is an acceptable
outcome; shipping a third `'unsafe-*'` to make the window appear is not.
