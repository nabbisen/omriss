# RFC-072 — Implementation Handoff

Related RFC: `../../proposed/072-linux-distribution-artifacts-and-aur-publishing.md`

Lifecycle status: inherited from RFC-072.

---

## 1. Build what RFC-072 says, as §9 updates it

## 2. Constraints

1. **Nothing outward-facing runs from your side.** No release is created, no
   tag pushed and nothing pushed to the AUR. Workflows are proven by `act` (if
   it works for these jobs), by YAML review and by the dry-run mode the owner
   runs. Say which.
2. **No default MIME handler** (RFC-072 §4.2): no `xdg-mime default`, no
   `mimeapps.list`.
3. Actions pinned to commit SHAs, with the tag in a comment.
4. Secrets only on the protected environment; no repository secret.
5. The PKGBUILDs follow the Arch Rust packaging guidelines: `cargo fetch
   --locked` in `prepare()` and `cargo build --frozen --release` in `build()`.
   `omriss-bin` has `provides`/`conflicts`.
6. `RELEASE_CHECKLIST.md`'s artifact matrix and owner steps, and RFC-037 §3,
   describe what is actually produced (RFC-072 §8.6). Do not leave a fourth
   description.
7. Product code does not change.
