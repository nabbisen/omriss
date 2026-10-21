# RFC-074: Flatpak Packaging and Flathub Publishing

**Project:** omriss — Omriss Editor
**Milestone:** M13 — distribution (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, release engineer
**Depends on:** RFC-063, RFC-068, RFC-072
**Related RFCs:** RFC-036, RFC-037, RFC-039, RFC-042, RFC-064

---

## 1. Summary

RFC-072 makes omriss installable on Arch. This RFC makes it installable on
every other desktop Linux: Flathub is the default software source on Fedora
Silverblue, the Steam Deck, elementary OS and Linux Mint, and a first-class one
on Fedora Workstation and Ubuntu.

It specifies the Flatpak manifest, the AppStream metadata Flathub requires, the
sandbox permissions omriss may ask for, the GitHub Actions integration, and —
most importantly — **four properties that must be proven inside a sandbox
before omriss is submitted**, because three of them are invisible to every test
we currently run.

## 2. Why this is a separate RFC from RFC-072

RFC-072 and this RFC share assets and a prerequisite, and differ in everything
else:

| | AUR (RFC-072) | Flathub (this RFC) |
|---|---|---|
| Who builds | the user's machine, or our CI | **Flathub's buildbot** |
| What CI publishes | a `PKGBUILD` push | a manifest push; the build happens elsewhere |
| Runtime libraries | the user's system | a pinned `org.gnome.Platform` |
| Filesystem access | unrestricted | portal-mediated, sandboxed |
| Review | none | human review, days to weeks |

The sandbox is the reason this cannot be a section of RFC-072. It changes what
the *application* must do — not only what packaging must contain — and three of
omriss's existing behaviours meet it (§7, §8).

## 3. What is already satisfied

Flathub's requirements state that **where a portal covers the use case, using
it is mandatory**. For a text editor, that requirement is usually what forces
`--filesystem=home` and the "potentially unsafe" marking on the listing.

omriss already complies, by a default nobody overrode: `rfd` 0.15.4 declares

```toml
default = ["xdg-portal", "async-std"]
```

and `Cargo.toml:31` takes `rfd = "0.15"` with default features. The Linux file
dialog is therefore the XDG Desktop Portal chooser, not a GTK3 dialog — visible
in Task 015's and Task 016's screenshots, which show the portal's "Files"
chooser.

**Consequence:** omriss needs **no static filesystem permission at all**. Every
file it touches is one the user chose through the portal.

## 4. Owner decisions required

### 4.1 The application ID

Flathub IDs are reverse-DNS, must match the MetaInfo `<id>`, and renames are
granted conservatively and not soon after acceptance. It is effectively a
one-way door, and it is needed **before RFC-072 is implemented** (§9).

> **Decided by the project owner on 2026-10-21: `io.github.nabbisen.omriss`.**
> This is a final decision, not a recommendation. RFC-072's asset names were
> amended to match on the same day.

| Option | How it is verified | Listing |
|---|---|---|
| `io.github.nabbisen.omriss` | log in to Flathub's Developer Portal with the GitHub account that **owns** `nabbisen/omriss` | **verified badge** |
| `com.reflerd.omriss` (not chosen) | a token at `https://reflerd.com/.well-known/org.flathub.VerifiedApps.txt`, or a `_flathub.reflerd.com` TXT record | **verified badge** |

`io.github.nabbisen.omriss` satisfies every ID rule: an allowed code-hosting
prefix, exactly four components, no reserved suffix. It is also the ordinary
choice on Flathub for a project whose home is a code host.

**An earlier draft of this section claimed the code-hosting ID could not be
verified. That was wrong.** Flathub's login-provider path verifies an
`io.github.<user>.<app>` ID through the GitHub account that owns the
repository — no domain, no `.well-known` file, no DNS record. Both options
reach the same badge, so the only real difference is what the ID is tied to:
the domain path survives a move off GitHub, the login path does not. Renames
are granted conservatively, so an eventual move would mean living with the old
ID — which many published applications do.

### 4.2 Network access

omriss makes no network request by design. One behaviour contradicts that: the
Markdown preview renders `<img src="https://…">` and the WebView fetches it
(RFC-064 §9a).

| | With `--share=network` | Without |
|---|---|---|
| Remote images in preview | load | **do not load** |
| Tracking pixel in a received document | fetches, revealing IP and open-time | cannot fetch |
| Attack surface | WebView has outbound network | WebView has none |

**Architect's recommendation: do not request network access.** It converts
RFC-064 §9a from a documented risk into an impossibility, and the feature it
costs — remote images in a *local* document preview — is one no user has asked
for.

This is a per-channel behaviour difference, so it must be written down where
users read about behaviour: `PRIVACY.md` §4 currently tells every reader that
preview may fetch remote images. If one channel does not, that section says so
(§8.3).

## 5. The manifest

`packaging/flatpak/io.github.nabbisen.omriss.yaml`, kept upstream in this repository
and copied into the Flathub repository by CI (§10).

### 5.1 Runtime and SDK

`wry` 0.53.5 reaches WebKitGTK through `webkit2gtk-sys` 2.0.1, which declares

```toml
[package.metadata.system-deps.webkit2gtk_4_1]
name = "webkit2gtk-4.1"
version = "2.4"
```

So the runtime must provide the **`webkit2gtk-4.1` pkg-config module** (the
GTK3 / libsoup3 ABI), plus GTK 3 and `libxdo` for `tao`. `org.freedesktop.Platform`
does not ship WebKitGTK; `org.gnome.Platform` does.

Flathub currently offers `org.gnome.Platform` 49, 50 and 51. **Pin the newest
runtime that still provides `webkit2gtk-4.1`, and prove it rather than assume
it** — gate G4 in §7. A runtime that has moved to `webkitgtk-6.0` only would
require either an older runtime (which Flathub does not accept once EOL) or
bundling WebKitGTK (which this RFC rejects, §11).

```yaml
runtime: org.gnome.Platform
runtime-version: '51'          # G4 decides this
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: omriss
```

### 5.2 Offline build: `Cargo.lock` is a hard prerequisite, again

**Flatpak builds have no network access.** `--share=network` in `build-args` is
explicitly rejected by Flathub. Cargo dependencies are supplied as a generated
source list:

```sh
python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
```

`flatpak-cargo-generator.py` reads **`Cargo.lock`** and nothing else: no
lockfile, no source list, no build. RFC-072 §2 already made the lockfile a hard
prerequisite for AUR, where `cargo fetch --locked` fails without it. Two
distribution channels now fail for the same missing file, by two unrelated
mechanisms. It is no longer a packaging preference; it is a precondition for
Linux distribution of any kind.

Build options follow the generator's documented shape:

```yaml
build-options:
  append-path: /usr/lib/sdk/rust-stable/bin
  env:
    CARGO_HOME: /run/build/omriss/cargo
    CARGO_NET_OFFLINE: 'true'
```

and the build is `cargo build --offline --release -p omriss`.

### 5.3 `finish-args`: the whole list, with a reason each

```yaml
finish-args:
  - --socket=wayland          # show a window
  - --socket=fallback-x11     # show a window where Wayland is absent
  - --share=ipc               # required alongside X11
  - --device=dri              # WebKit rendering; see G1
```

That is the complete set. Specifically **absent**, and each absence is a
decision, not an omission:

| Not requested | Why |
|---|---|
| `--share=network` | §4.2 |
| `--filesystem=…` | §3 — the portal covers every file omriss opens |
| `--talk-name=…` | portals need no static permission; omriss has no tray icon, no D-Bus service of its own |
| `--env=WEBKIT_DISABLE_COMPOSITING_MODE=1` | a workaround for a symptom we have not observed. Add it only if G1 shows it is needed, with the observed symptom named in a comment |

### 5.4 Installed files

```text
/app/bin/omriss
/app/share/applications/io.github.nabbisen.omriss.desktop
/app/share/icons/hicolor/scalable/apps/io.github.nabbisen.omriss.svg
/app/share/metainfo/io.github.nabbisen.omriss.metainfo.xml
/app/share/licenses/io.github.nabbisen.omriss/LICENSE
```

The licence path is a Flathub requirement, and it needs RFC-072 §4.4's
`LICENSE` file to exist — `Cargo.toml` has declared `license = "Apache-2.0"`
since v0.1.0 with no such file in the tree.

## 6. AppStream metadata

Flathub requires a MetaInfo file that passes validation, and requires it to be
**integrated upstream** rather than added in the submission pull request. So it
is repository content, not packaging content: `packaging/linux/io.github.nabbisen.omriss.metainfo.xml`,
installed by both the Flatpak manifest and the AUR package.

Minimum contents:

- `<id>` identical to the application ID and to the desktop file's basename
- `<name>`, `<summary>`, `<description>` — the summary is a single sentence
  with no marketing and no trailing period
- `<metadata_license>` and `<project_license>Apache-2.0</project_license>`
- `<launchable type="desktop-id">io.github.nabbisen.omriss.desktop</launchable>`
- `<screenshots>` — at least one, at an absolute HTTPS URL, showing the
  application doing its actual job. A screenshot of an empty Welcome screen is
  worse than none
- `<releases>` — one `<release version="…" date="…">` per tag, which CI appends
  (§10.2); the version must match `Cargo.toml`
- `<content_rating type="oars-1.1"/>` — omriss has no user-generated content
  exchange, so every attribute is `none`
- `<developer id="…"><name>…</name></developer>` and `<url>` entries for
  homepage, bugtracker and vcs-browser

Validation is `appstreamcli validate --explain`, run in CI (§10.1) — not left
to a Flathub reviewer to discover.

**This metadata is not Flatpak-specific.** GNOME Software and KDE Discover read
it for natively installed packages too, so the AUR package should install it as
well. RFC-072 gains it by reference, not by duplication.

## 7. Four gates that must be proven inside a sandbox

Three of these are invisible to `cargo test --workspace`, because they are
properties of the *environment*, not of the code. None may be assumed.

### G1 — The UI renders at all

`dioxus-desktop` binds `TcpListener::bind((127.0.0.1, 0))` and drives **every**
UI mutation over that loopback WebSocket (`edits.rs:148`). Without
`--share=network`, Flatpak places the app in its own network namespace.

Loopback is expected to exist inside that namespace, which would make the
socket *less* reachable than it is natively — the host cannot connect to it at
all. That is a better answer to RFC-064's trust-boundary problem than the CSP
is, and it costs nothing.

If it is wrong, omriss does not render and the channel is closed. **Prove it
before anything else in this RFC is built.**

### G2 — Saving a file does not corrupt it

`write_atomic` (`file_dialog.rs:100-108`) writes `path.with_extension("tmp.omriss")`
— a **sibling of the target** — then renames over the target.

Inside the sandbox, the target is not `/home/user/notes.md`; it is
`/run/user/1000/doc/<id>/notes.md`, a FUSE directory holding a single document.
Create-sibling-then-rename is the pattern the document portal's FUSE
implementation was rewritten to support, so this is expected to work — but
"expected" is not a standard this project applies to the save path. This is the
source-preservation invariant, under a filesystem we have never tested on.

**Proof required:** run the `round_trip_tests` save path against a
portal-provided path inside the sandbox, and confirm byte equality and that
undo restores the original bytes. A manual save-and-diff is not sufficient.

If G2 fails, the fix is in RFC-068's durable-write design — temp file in the
same directory only when that is possible, write-in-place with `fsync` as the
documented alternative. **It is not `--filesystem=home`** (§11).

### G3 — Opening a file from outside the sandbox

RFC-063 lets omriss take a path argument, and RFC-072 §4.2 registers it as a
MIME handler. A sandboxed application cannot read a path merely handed to it on
its command line.

The desktop entry must therefore use Flatpak's file-forwarding token so the
host path is translated into one the sandbox can reach:

```ini
Exec=omriss @@u %u
```

`%u` rather than RFC-072's `%f`, because forwarding operates on URIs;
`@@u` makes `flatpak run` convert them back to paths before `main` sees them,
so RFC-063's argument handling is unchanged.

**One upstream desktop file cannot serve both channels** — `@@u` is meaningless
to a native install. The Flatpak manifest patches the `Exec` line at build time
with a documented one-line substitution, and §12 asserts the result rather than
trusting it.

**Proof required:** `flatpak run io.github.nabbisen.omriss ~/notes.md` opens the file,
and "Open With" from a file manager does too.

### G4 — The runtime provides `webkit2gtk-4.1`

```sh
flatpak install -y flathub org.gnome.Sdk//51
flatpak run --command=sh --devel org.gnome.Sdk//51 \
  -c 'pkg-config --modversion webkit2gtk-4.1'
```

Pin the newest runtime for which this prints a version ≥ 2.4. Record the
version in the manifest and in this RFC's Status when it is settled.

## 8. Behaviour that differs under the sandbox

These are not defects, and they must not be discovered by a user.

### 8.1 The recent-files list empties itself

`settings.rs:104-109` filters recent files through `Path::exists()`. A stored
host path does not exist inside the sandbox, so the list does not error — it
**silently becomes empty**, and the user is told nothing.

Under RFC-073 §4.3's typed `OpenFailure` this becomes expressible: a recent
entry that cannot be reached is a `PermissionDenied`, not a non-existent file,
and the two deserve different words. Scheduled there; recorded here because the
sandbox is what makes the distinction visible.

### 8.2 The footer shows a path the user does not recognise

A portal-opened file arrives as `/run/user/1000/doc/ab12cd/notes.md`. Task 015
finding O-4 already noted that the footer shows a full path when opened through
the dialog. Inside the sandbox that path is machine-generated and meaningless.

**Decision:** when the path is under `/run/user/*/doc/`, the footer shows the
file name alone. Owned by whichever RFC next touches the footer; stated here so
it is not rediscovered as a Flatpak defect.

### 8.3 Remote images in preview

Per §4.2, they do not load. `PRIVACY.md` §4 must say which channel it is
describing. A privacy document that overstates what the application does is as
misleading as one that understates it.

## 9. Relationship to RFC-072 — and what must change there

| Asset | Today in RFC-072 | After this RFC |
|---|---|---|
| Desktop entry | `packaging/linux/omriss.desktop` | `packaging/linux/io.github.nabbisen.omriss.desktop` |
| Icon | `packaging/linux/omriss.svg` | `packaging/linux/io.github.nabbisen.omriss.svg` |
| MetaInfo | absent | `packaging/linux/io.github.nabbisen.omriss.metainfo.xml`, installed by both |
| `Exec=` | `omriss %f` | unchanged natively; patched to `omriss @@u %u` in the Flatpak build |
| `LICENSE` | §4.4 adds it | also installed to `/app/share/licenses/$FLATPAK_ID/` |

Flathub requires the desktop file, icon and MetaInfo to be named after the
application ID. Naming them so **upstream** gives one canonical name across
both channels; the alternative — renaming inside the Flatpak build — creates
two names for one file and a rename step that breaks silently when someone
edits the wrong copy.

`Icon=io.github.nabbisen.omriss` resolves correctly for a native install through
`hicolor`, so this costs the AUR package nothing.

**RFC-072 is still Proposed, so this is an amendment and not a correction to
shipped work — provided the application ID is decided before RFC-072 is
implemented.** That ordering is the one piece of this RFC that is urgent.

## 10. CI

Flathub **builds on its own infrastructure**, from a `flathub/io.github.nabbisen.omriss`
repository created after review, to which the maintainer gets push access. Our
CI therefore does two different jobs, and neither of them produces the published
artifact.

### 10.1 Validation, on pull requests that touch packaging

```yaml
- uses: flatpak/flatpak-github-actions/flatpak-builder@<pinned-sha>
  with:
    bundle: omriss.flatpak
    manifest-path: packaging/flatpak/io.github.nabbisen.omriss.yaml
    cache-key: flatpak-builder-${{ github.sha }}
```

plus `appstreamcli validate --explain` on the MetaInfo and
`desktop-file-validate` on the desktop entry.

**Trigger on `packaging/**`, `Cargo.lock` and tags — not on every push.** The
job downloads a multi-gigabyte runtime and builds the whole dependency tree;
making every documentation commit wait for it would be a tax on the work this
project actually does.

Its value is specific: a manifest or MetaInfo error is caught by us, in
minutes, instead of by a volunteer reviewer, in days.

### 10.2 Publication, on a release tag

1. Regenerate `cargo-sources.json` from the tagged `Cargo.lock`.
2. Point the manifest's `sources` at the tagged archive from RFC-072 §5.2,
   with its `sha256`.
3. Append a `<release version="X.Y.Z" date="…"/>` to the MetaInfo.
4. Push to `flathub/io.github.nabbisen.omriss`, whose buildbot builds and publishes.

**This job runs only after the release job that verifies tests pass**, exactly
as the Microsoft Store job does. The same rule applies for the same reason:
nothing reaches a user that has not passed the gate.

### 10.3 Security posture

Identical to RFC-072 §5.4 — a dedicated deploy key for the Flathub repository
and nothing else, a protected environment the owner must approve, and every
third-party action pinned to a commit SHA rather than a tag.

**Alternative worth weighing:** Flathub's `flatpak-external-data-checker` can
open the version-bump pull request itself when it sees a new tag, leaving the
merge to the owner. It is slower by one manual step and **stores no credential
of ours anywhere**. For a solo maintainer that trade is defensible; §13 lists
it as an owner decision rather than resolving it here.

## 11. Non-goals

1. **No bundled WebKitGTK.** If no supported runtime provides
   `webkit2gtk-4.1` (G4), that is a reason to reopen this RFC, not to vendor a
   browser engine into our build.
2. **No `--filesystem=host`, `--filesystem=home`, or any static filesystem
   permission** — including as a fallback if G2 fails. A sandbox escape hatch
   added to avoid fixing the write path is the opposite of what this channel is
   for, and it marks the listing as potentially unsafe.
3. **No Snap, AppImage, `.deb` or `.rpm`.** Each is its own channel with its
   own review and its own failure modes.
4. **No beta or nightly Flatpak channel.** Flathub's stable repository is for
   stable software, and new applications must enter it first.
5. **No change to omriss's behaviour outside the sandbox.** Everything here is
   packaging, except the §8 items, which are recorded against their owning RFCs.

## 12. Validation and test plan

- **G1–G4 pass**, each with the command and its output recorded, not a claim.
- `appstreamcli validate --explain` is clean; `desktop-file-validate` is clean.
- The exported desktop entry contains the forwarding token:
  `grep -- '--file-forwarding' ~/.local/share/flatpak/exports/share/applications/io.github.nabbisen.omriss.desktop`
- `flatpak run io.github.nabbisen.omriss` and
  `flatpak run io.github.nabbisen.omriss <path>` both open correctly (G3).
- **The reduced smoke run (checks A–I) is executed against the Flatpak build**,
  not only against a `cargo build --release` binary. Flatpak is a distinct
  platform under RFC-038's rules, and check I in particular must be re-proven:
  a different WebKitGTK build is exactly where escaping behaviour could differ.
- The window's icon and name resolve in the shell's dock — `StartupWMClass`
  matches what `tao` sets.
- `flatpak run --command=sh io.github.nabbisen.omriss -c 'ls /'` shows no host home
  directory.

## 13. Acceptance criteria

1. `io.github.nabbisen.omriss` is accepted on Flathub, installs from a clean
   machine with `flatpak install flathub io.github.nabbisen.omriss`, and is
   **verified** through the GitHub login path (§4.1) — an unverified listing of
   a single-maintainer app gives a user no way to tell it from a repackaging.
2. G1–G4 are proven, with recorded evidence, **before submission**.
3. The sandbox requests no filesystem permission and no network access, or the
   deviation is recorded in this RFC with the owner's decision.
4. `Cargo.lock` is committed (§5.2, RFC-072 §2).
5. A tagged release reaches Flathub without a hand-edited file.
6. `PRIVACY.md` describes preview image loading per channel (§8.3).
7. RFC-072's asset names match the application ID before RFC-072 is implemented
   (§9).
8. The reduced smoke run passes against the Flatpak build.

## 14. Open decisions for the owner

**Resolved:** the application ID — §4.1, decided 2026-10-21.

1. **Network access** — §4.2. Recommendation: no.
2. **Publication mechanism** — §10.3: hold a deploy key and push, or let
   `flatpak-external-data-checker` open a pull request we merge.
3. **Screenshots** — §6 requires at least one hosted at an HTTPS URL. The
   repository has none today; they are also what a prospective user looks at
   first.
