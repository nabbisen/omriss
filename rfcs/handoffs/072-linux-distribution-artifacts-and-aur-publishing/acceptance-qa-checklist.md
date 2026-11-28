# RFC-072 — Acceptance and QA Checklist

Related RFC: `../../proposed/072-linux-distribution-artifacts-and-aur-publishing.md`

Lifecycle status: inherited from RFC-072.

---

## Gates (each PR)

- [ ] `cargo fmt --all --check`, clippy `-D warnings`, `cargo test --workspace`
      (product code unchanged, counts unchanged)
- [ ] `bash scripts/check-rfcs.sh`, `mdbook build docs`
- [ ] Workflow YAML: `actionlint`, if available, or say how it was checked

## Proofs

- [ ] `desktop-file-validate`, `appstreamcli validate`: clean
- [ ] `makepkg` clean build of both packages; `namcap` output quoted
- [ ] Installed: menu entry, icon, "Open With" lists omriss, default handlers
      unchanged (`xdg-mime query default` before and after)
- [ ] `pacman -R` leaves no file behind
- [ ] The tarball's contents listed; `sha256sum -c` passes
- [ ] Actions pinned to SHAs; secrets only on the environment
- [ ] Dry-run mode: shown to create no release and push nothing (from the
      YAML, and `act` if usable)

## Report

Gates, each proof's output, what the owner must do (exact steps), and what
was not run.
