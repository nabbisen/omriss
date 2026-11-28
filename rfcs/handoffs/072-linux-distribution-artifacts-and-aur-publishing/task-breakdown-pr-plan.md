# RFC-072 — Task Breakdown and PR Plan

Related RFC: `../../proposed/072-linux-distribution-artifacts-and-aur-publishing.md`

Lifecycle status: inherited from RFC-072.

## PR 1 — packaging files, verified locally

- `packaging/linux/`: the `.desktop` file (§4.1), the 256×256 icon from
  `StoreLogo.png` (§9.1), the AppStream metainfo (§9.2, RFC-074 §6).
- `packaging/arch/omriss/PKGBUILD` and `packaging/arch/omriss-bin/PKGBUILD`,
  with `pkgver` as a template value.
- A script that builds the tarball exactly as PR 2's job will. CI calls the
  same script, so the two cannot drift.
- Local proof: `makepkg` in a clean chroot or container for both packages
  (from the local tarball, with the source package built from a local `git
  archive`), `namcap` on the PKGBUILDs and the packages,
  `desktop-file-validate`, `appstreamcli validate`. Install, check the menu
  entry and the icon, check that the default `.md`/`.json` handler is
  unchanged, then `pacman -R` and check that nothing is left.

## PR 2 — the Linux release job and the GitHub Release

- A job in `release.yml` that runs PR 1's script, with `--locked`, and
  uploads the tarball and `.sha256`.
- Creating the release: bare-version title, notes from `CHANGELOG.md`.
  Skipped in dry-run mode.
- `RELEASE_CHECKLIST.md` and RFC-037 §3 updated.

## PR 3 — AUR jobs, off until the owner turns them on

- One job per package, on the protected environment, gated by `AUR_PUBLISH`.
  The package name comes from an input, so the owner's dry run can point at a
  scratch package.
- `RELEASE_CHECKLIST.md`: the owner's one-time steps (RFC-072 §9) and the
  per-release check.
