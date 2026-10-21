# RFC-072: Linux Distribution Artifacts and AUR Publishing

**Project:** omriss — Omriss Editor
**Milestone:** M13 — distribution (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, release engineer
**Depends on:** RFC-037, RFC-042
**Related RFCs:** RFC-035, RFC-038, RFC-064, RFC-074

---

## 1. Summary

The owner has registered two AUR packages —
[`omriss`](https://aur.archlinux.org/packages/omriss) (build from source) and
[`omriss-bin`](https://aur.archlinux.org/packages/omriss-bin) (prebuilt binary).
Neither can be populated correctly from this repository as it stands.

This RFC makes Linux a first-class distribution target: a published binary
artifact with a checksum, the desktop integration files a Linux package is
expected to install, and **both AUR packages updated by CI on tag** rather than
by hand.

## 2. Prerequisite: `Cargo.lock` must be committed

This is not a preference. The [Arch Rust packaging
guidelines](https://wiki.archlinux.org/title/Rust_package_guidelines) specify
`cargo fetch --locked` in `prepare()` and `cargo build --frozen --release` in
`build()`. `--locked` **fails when the lockfile is absent**, and the documented
workaround — adding `cargo update` — is explicitly accepted only for VCS
packages, with the wiki noting the result "will not be a fully reproducible
build because the dependencies will be resolved at build time."

`Cargo.lock` is currently gitignored (`.gitignore:8`), so it is absent from the
git tag tarball a source PKGBUILD fetches. Without it, `omriss` either cannot
follow the guidelines or ships a knowingly non-reproducible build.

This is the fourth consumer to need it: CI cannot pass `--locked`, the Store
MSIX is not reproducible from a tag, the 0.17.0 audit flagged it
(AUDIT-0170-017), and now the AUR source package requires it. The stock comment
above the ignore line states the rule — commit the lockfile for a workspace that
ships a binary.

**This RFC cannot be implemented without it.** The decision is the owner's.

## 3. What is missing today

| Needed | Present? |
|---|---|
| Linux x86_64 binary on a GitHub release | **No.** The newest release is `0.6.1` (2025-02-08), under the former name `layered`. The last 46 tags have no release at all. |
| Published `sha256sum` | **No.** `README.md` tells users to verify one; none is published (AUDIT-0170-016). |
| `.desktop` entry | **No.** `packaging/` contains only `windows/`. |
| Linux icon | **No.** Only Windows tile PNGs. |
| `LICENSE` file | **No**, despite `Cargo.toml` declaring Apache-2.0. |
| `Cargo.lock` | **No** — §2. |

Note the shape of the regression: release `0.6.1` published
`layered-linux-x64-gnu-0.6.1.tar.gz`, `layered-macos-arm64-0.6.1.tar.gz` and
`layered-windows-x64-0.6.1.zip`. Three-platform artifacts existed and were lost,
presumably in the RFC-047 rename. **Rebuilding this in CI rather than by hand is
the point of §5** — a manual process already failed once, silently.

## 4. Desktop integration

### 4.1 `.desktop` entry

`packaging/linux/io.github.nabbisen.omriss.desktop`, installed to
`/usr/share/applications/io.github.nabbisen.omriss.desktop`.

**Named after the Flatpak application ID, deliberately** (RFC-074 §4.1, owner
decision of 2026-10-21). Flathub requires the desktop entry, icon and MetaInfo
to carry the application ID, and shipping one canonical name upstream beats a
rename step inside the Flatpak build that breaks silently when someone edits
the wrong copy. `Icon=` resolves through `hicolor` either way, so this costs
the AUR package nothing.

```ini
[Desktop Entry]
Type=Application
Name=Omriss
Comment=A text editor that helps clarify ideas and refine them, layer by layer
Exec=omriss %f
Icon=io.github.nabbisen.omriss
Terminal=false
Categories=Office;TextEditor;Utility;
MimeType=text/markdown;application/json;
StartupWMClass=omriss
```

`%f` (single file) rather than `%F`: omriss opens one file per process
(RFC-063), and `%F` would promise multi-file handling it does not have until
RFC-057.

### 4.2 File associations: handler, never default

`MimeType=` registers omriss as *an* option under "Open With". It does **not**
make omriss the default handler — that requires a `mimeapps.list` entry, which a
package must not write on the user's behalf.

This is deliberate and must stay that way. Claiming `application/json` as
default would silently take `.json` away from a developer's existing editor,
which is a hostile surprise on exactly the machines most likely to install from
the AUR. Owner-approved 2026-10-21.

The packaging must not run `xdg-mime default` or install a `mimeapps.list`.

### 4.3 Icon

`packaging/linux/io.github.nabbisen.omriss.svg` installed to
`/usr/share/icons/hicolor/scalable/apps/io.github.nabbisen.omriss.svg`
(§4.1's naming rule).

Prefer SVG. The existing Windows assets are fixed-size tile images in the wrong
aspect ratios for `hicolor`; if no vector source exists, install
`StoreLogo.png` resized into `256x256` and say so, rather than scaling tile art
into the wrong boxes.

### 4.4 `LICENSE`

Add the Apache-2.0 text as `LICENSE` at the repository root. `Cargo.toml` has
declared `license = "Apache-2.0"` since v0.1.0 with no such file, so neither a
package nor a crates.io consumer can read the terms.

### 4.5 AppStream metadata

`packaging/linux/io.github.nabbisen.omriss.metainfo.xml`, installed to
`/usr/share/metainfo/`. Specified in **RFC-074 §6**, which Flathub requires it
for; it is listed here because GNOME Software and KDE Discover read it for
natively installed packages too, so the AUR package installs the same file. It
is not duplicated in this RFC.

## 5. CI: build, publish, and update the AUR

### 5.1 Linux release job in `release.yml`

A job alongside `package`, on `ubuntu-latest`, producing:

```text
omriss-<version>-x86_64-unknown-linux-gnu.tar.gz
omriss-<version>-x86_64-unknown-linux-gnu.tar.gz.sha256
```

The tarball contains the release binary, `LICENSE`, `README.md`, the `.desktop`
file, and the icon, so `omriss-bin`'s `package()` is a copy rather than a
reconstruction.

Build with `--locked` once §2 lands.

### 5.2 Create a GitHub Release

`release.yml` currently ends at the Store. It must also create a GitHub Release
for the tag and attach §5.1's artifacts. `omriss-bin`'s `source=` points at that
URL, so the release is the package's actual dependency — not a courtesy.

### 5.3 AUR publish job

One job per package, `needs:` the release job, using
[`KSXGitHub/github-actions-deploy-aur`](https://github.com/KSXGitHub/github-actions-deploy-aur).
Required inputs are `pkgname`, `pkgbuild`, `commit_username`, `commit_email` and
`ssh_private_key`; `updpkgsums` (default `false`) regenerates checksums, and
`test` (default `false`) can build-test the package before pushing.

PKGBUILD templates live in the repo, at `packaging/arch/omriss/PKGBUILD` and
`packaging/arch/omriss-bin/PKGBUILD`, with `pkgver` substituted at publish time.
The AUR repository is a publish target, not the source of truth — otherwise the
packaging drifts from the code that produced it, which is how the `layered`
artifacts were lost.

### 5.4 Security: same posture as the Store job

The SSH key can push to the AUR, so it is release-grade credential:

- a **dedicated** key pair generated for CI, never the owner's personal AUR key;
- stored as an **environment** secret on a protected environment with required
  reviewers, not a repository secret — repository secrets are readable by any
  workflow on any branch;
- the action **pinned to a commit SHA**, not a moving tag. It runs with that key
  in scope.

This mirrors RFC-064 §9's reasoning and `release.yml`'s existing `TODO` about
pinning the Store publisher action.

### 5.5 Dependencies for the source package

Verified present in Arch's repositories at the time of writing:

| Package | Provides |
|---|---|
| `webkit2gtk-4.1` | WebKitGTK and JavaScriptCore 4.1 |
| `gtk3` | GTK 3 |
| `xdotool` | `libxdo` |
| `openssl` | OpenSSL |

`depends` for both packages; `rust` and `pkgconf` as `makedepends` for `omriss`
only. `omriss-bin` needs no toolchain and should declare
`provides=('omriss')` / `conflicts=('omriss')`.

## 6. Non-goals

1. **Not macOS or Windows artifacts.** Those are a separate decision. This RFC
   is Arch Linux, because that is what was asked for.
2. **Not `aarch64`.** x86_64 only unless the owner asks otherwise.
   `PLATFORMS.md` invites community ARM work; nothing here blocks it.
3. **No Flatpak, Snap, AppImage, or `.deb`.** One channel done properly first.
4. **No default MIME handler** — §4.2.
5. **No omriss app on crates.io.** `crates/app` is `publish = false` (RFC-062)
   and stays so; `cargo install omriss` is not a supported path, which is
   precisely why a source PKGBUILD must build from the tag tarball.

## 7. Validation and test plan

- `makepkg --clean --cleanbuild` succeeds for both packages on a clean Arch
  container, from the published artifacts of a real tag.
- `namcap` reports no errors on either `PKGBUILD` or the built package.
- The installed app appears in the desktop application menu, and its icon
  renders.
- Opening a `.md` file through "Open With" launches omriss with that file; the
  system default handler for `.md` and `.json` is **unchanged** after install.
- `sha256sum -c` verifies the published tarball.
- `pacman -R` removes every installed file, leaving nothing behind.

Dry-run the AUR job against a scratch package name before pointing it at the
real ones. A bad force-push to the AUR is visible to every Arch user.

## 8. Acceptance criteria

1. A tag publishes a GitHub Release with a Linux x86_64 tarball and its
   `sha256sum`, produced by CI.
2. Both AUR packages are updated by that same workflow, with no manual step.
3. `makepkg` and `namcap` pass for both, verified on a clean Arch system.
4. The desktop entry and icon install correctly; no default MIME association is
   claimed.
5. `LICENSE` is present at the repository root.
6. `RELEASE_CHECKLIST.md`'s artifact matrix and RFC-037's §3 reflect what is
   actually produced — the audit found three documents describing three
   different releases (AUDIT-0170-016), and this must not add a fourth.
7. `README.md`'s checksum instruction is true.
