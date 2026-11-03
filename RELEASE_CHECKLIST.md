# Release Checklist

This checklist must be completed before any public release of omriss.
Per RFC-042 §5, public release requires explicit confirmation from the
product owner. No automated process may bypass this gate.

---

## Pre-Release Gate

- [ ] All workspace tests pass: `cargo test --workspace`
- [ ] No open issues labelled `release-blocker`
- [ ] `CHANGELOG.md` entry written for this version. **Entries accumulate under
      `## [Unreleased]`, and that heading becomes `## [X.Y.Z] - YYYY-MM-DD` at
      the moment the tag is cut — the date is the release date and is never
      written in advance.** 0.17.0's entry was created months early carrying
      `2026-08-16`, which would have shipped as a false release date had it not
      been caught.
- [ ] Version bumped in `Cargo.toml` workspace package
- [ ] RFC index up to date: `./scripts/check-rfcs.sh`
- [ ] Internal dependency versions match the release: the `version` of
      `omriss-core` and `omriss-ui` in `[workspace.dependencies]` is the minor
      being released, not `"0"`. That field is what a published package
      carries, and `"0"` lets a published crate resolve against a later,
      API-breaking 0.x.
- [ ] Package roles verified before publication:
      `omriss-core` is the publishable library package, and the `omriss` app
      package is **permanently** `publish = false` (RFC-062 §10.1).
- [ ] **crates.io publish, in this order** (RFC-062 §10.1), by hand after the
      gate passes — a published version can never be deleted, only yanked:
      1. `cargo publish -p omriss-core`
      2. `cargo publish -p omriss-ui` — this *requires* step 1, since it
         resolves `omriss-core` from the registry, not from the path
      3. confirm the README badges resolve: crates.io, docs.rs and deps.rs all
         point at `omriss-core`, and all three 404 until step 1 lands

---

## Data Integrity Tests (RFC-040, RFC-042 top criterion)

- [ ] Source-preservation golden tests pass
- [ ] Fixture catalog tests pass (all fixtures)
- [ ] Structural operation tests pass
- [ ] Undo round-trip tests pass for all operation types

---

## Artifact Matrix (RFC-037)

What a release actually produces, as of 0.17.0. Rows say what happens, not
what was once intended — the previous version of this table promised an
`omriss-X.Y.Z.tar.gz` that no part of the release process has ever built.

| Target | Artifact | How |
|---|---|---|
| Windows | `omriss-X.Y.Z.msix` → Microsoft Store | **automatic** — `release.yml` on a bare-semver tag, after `cargo test --workspace` passes |
| Any | crates.io: `omriss-core`, then `omriss-ui` | **by hand** at the tag, in that order (RFC-062 §10.1) |
| Any | source archive of the tag | GitHub serves one for every tag; the release process does not build it |
| Linux | binary tarball, AUR `omriss` and `omriss-bin` | planned — RFC-072 |
| Linux | Flatpak via Flathub | planned — RFC-074 |
| macOS | app bundle | **not planned.** No Mac has been available to build or test on, and RFC-038 forbids claiming a platform that has not been run |

Checksums apply to a downloadable binary, which today means the MSIX. The
Store distributes it; a published checksum matters once RFC-072's tarball
exists, and that RFC specifies it.

---

## Platform Smoke Tests (RFC-038)

Run the following workflow on each supported platform before release.
Record pass/fail and the OS version tested.

### Smoke Workflow

1. **Launch** — app opens without error
2. **Open** — open `crates/core/tests/fixtures/academic-paper.md` via Ctrl+O
3. **Overview** — top-level sections appear as heading cards
4. **Zoom in** — press Enter on the first heading card
5. **Edit body** — type a word, confirm textarea is responsive
6. **Save** — Ctrl+S saves without error; status shows "Saved"
7. **Byte check** — open saved file in external editor; verify unedited sections unchanged
8. **Raw source** — Ctrl+` shows the full Markdown source
9. **Zoom out** — Esc returns to overview
10. **Keyboard nav** — Tab + arrow keys navigate without trapping
11. **Search** — Ctrl+F opens search panel; type a query; result appears
12. **Close/reopen** — close and reopen the saved file; content intact

### Structured-Format Workflow (RFC-054, from 0.17.0)

Steps 1–12 exercise Markdown only. From 0.17.0 omriss also edits JSON, and a
release cannot be certified on a workflow that never opens the format the
release is for. Run these on each supported platform alongside the Markdown
workflow.

Use `crates/core/tests/fixtures/structured-sample.json`. It is formatted
deliberately badly — irregular indentation, `1.50`, `1e3`, `-0`, a duplicate
key, a `null`, and non-ASCII text — precisely so that step 17 can detect an
editor that reserializes the file instead of patching it. Do not tidy it;
`structured_fixture_smoke` fails if those constructs are removed.

13. **Open from the command line** — `omriss crates/core/tests/fixtures/structured-sample.json`
    opens with the Document Map populated (RFC-063). This is also the cheapest
    way to reach a rendered document for the remaining steps.
14. **Navigate** — click a text value, a number, a group, and a list; each shows
    its own editor or summary in the right panel. Confirm both `duplicate` keys
    appear as separate rows, and that `maintainer` (a `null`) is shown but its
    editor is unavailable
15. **Edit a value** — change a text or number value; the dirty indicator appears
16. **Invalid draft blocked** — type a non-number into a number field; an inline
    message appears, and navigating away is refused with the draft preserved
17. **Save** — Ctrl+S; then **inspect the file in an external editor**: only the
    edited value's bytes changed. Indentation, key order, and line endings
    elsewhere are untouched
18. **Undo** — the original bytes return *and* the edited node stays focused
19. **Container raw edit** — select a group or list, use "Show this part as
    text", replace it with valid JSON of the same kind, commit
20. **Malformed file** — open a `.json` with a syntax error: the app opens, the
    source is preserved, the Document Map is empty, and "Show plain file text"
    still works
21. **Markdown unaffected** — repeat steps 2–8 on a `.md` file in the same
    session

### Reduced Run for a CI-Covered Platform (from 0.17.0)

Steps 1–21 were written before CI ran the full suite on each platform. Where
`ci.yml` already runs `cargo test --workspace` on that platform — today Linux
and Windows — most of the steps re-check logic that CI proves on every push:
invalid-draft rejection, container raw edit, malformed-file recovery, undo
bytes, and (once the save round-trip test lands) byte preservation through the
real save path.

A human run on such a platform only needs to cover what automation cannot
reach: the WebView actually rendering, keyboard delivery inside the WebView,
the native file dialog, and switching files inside one running process.

The full 1–21 run remains required for a platform **without** CI coverage
(macOS today) and for the first release on any new platform.

**Before you start.**

| | |
|---|---|
| Fixtures | `crates/core/tests/fixtures/academic-paper.md` and `crates/core/tests/fixtures/structured-sample.json` |
| **Work on copies** | Checks C and F write to the file. Copy both fixtures somewhere scratch and open the copies. The tracked fixtures are the baseline every later run compares against, and `structured_fixture_smoke` fails if the JSON one is tidied |
| Check I needs a file you make | A small `.md` containing raw HTML — `<b>bold</b>`, a `<table>`, and ideally `<img src=x onerror="document.title='XSS'">`. If the window title stays "Omriss", no script ran |
| Check H | Only on a Japanese-language Windows install. Otherwise record "not run" — not "passed" |
| Settings | Point the app at a throwaway config directory if you can, so the run does not read or write your real recent-files list |

| Check | What to do | Replaces |
|---|---|---|
| **A** | Launch. The toolbar and the Welcome screen appear. The Document Map stays empty until a file is open (RFC-048 layout). | 1 |
| **B** | Ctrl+O, open `academic-paper.md` through the dialog. Section cards appear. | 2, 3 |
| **C** | Click a card, type a word in the body, Ctrl+S. Status shows "Saved". | 4, 5, 6 |
| **D** | Try Esc, Ctrl+\`, and Ctrl+F, then type into the search field. Note which work. This checks `known-limitations.md`, so record what happened rather than pass/fail. | 8–11 |
| **E** | Without restarting, Ctrl+O and open `structured-sample.json`. Both `duplicate` keys show as separate rows. | 13, 14, 21 |
| **F** | Click **`scale`** (its value is `1e3`), change it to `2500`, Ctrl+S, then Ctrl+Z. The old value returns and stays selected. Then check the file in a text editor: **`version` still reads `1.50` and `offset` still reads `-0`.** Those two are deliberately odd and are the canaries — an editor that reserialized the file instead of patching one value would have normalised them. | 15, 17, 18 |
| **G** | Close the app, reopen both files. The saved changes are there. | 12 |
| **I** | Open `crates/core/tests/fixtures/html_content.md` — a tracked fixture containing an HTML comment, `<em>` and a `<div>` block — and view the preview. The markup must show as **visible text**, never rendered. Live markup here is release-blocking (RFC-064). | — |
| **H** | Only on a Japanese-language Windows install: omriss starts in Japanese. Otherwise record as not run. | 22 |

Steps 7, 16, 19 and 20 have no human check here: CI covers them on this
platform. Step 17's byte inspection moves to an automated round-trip test
through `open_markdown_path` and `save_markdown`, run on `windows-latest`.

### Windows-Only Step (from 0.17.0)

22. **OS language** — on a Windows install whose display language is Japanese,
    with no saved language preference, omriss starts in Japanese. If no Japanese
    Windows installation is available, record the step as not run rather than
    passed.

### Release-Blocking Failures (RFC-038 §4)

Any of the following block release:

- app fails to launch on a supported platform
- open/save corrupts unrelated bytes in the Markdown file
- **open/save corrupts unrelated bytes in a JSON file** — step 17
- Ctrl+S / Cmd+S cannot save
- keyboard navigation traps the user (no Esc/Tab escape)
- save failure falsely reports success
- documented source-preservation invariant violated
- **a malformed structured file loses or rewrites the user's source** — step 20

### Smoke Test Evidence

Record one block per platform. "Steps completed" must name the ranges actually
run — `1–12` alone means the structured-format workflow was not exercised, and
from 0.17.0 that is an incomplete smoke test, not a passing one.

```
OS: Windows
Version: Windows 11 Home 25H2
Artifact: ~~omriss-X.Y.Z.tar.gz~~ Commit `ec667eb1c6f25c6cab32d0bea4819de72e377a60` used via `cargo run --release -p omriss`.
Fixtures: ~~copies of crates/core/tests/fixtures/{academic-paper.md, structured-sample.json}~~
          ~~plus the hand-made raw-HTML file for check I~~
The checked out project was used.
Date: 2026-11-03
Tester: nabbisen (owner)
Steps completed: A-H
Failures (if any): (All were reported to architect on each)
Sign-off: 
```

---

## Unsigned Build Policy (RFC-037)

Early releases are not code-signed. Documentation must explain:

- **macOS**: Right-click → Open to bypass Gatekeeper on first run.
- **Windows**: SmartScreen may warn; click "More info" → "Run anyway".
- Verify the SHA-256 checksum against the release page before running.

---

## Release Notes Template (RFC-037)

```markdown
## omriss vX.Y.Z — YYYY-MM-DD

### Highlights
...

### Data Integrity Notes
All source-preservation tests pass. No known data-corruption issues.

### Known Limitations
- Raw source editing: read-only in this release.
- No plugin system.
- No collaboration features.
- Unsigned builds: see RELEASE_CHECKLIST.md for verification steps.

### Platform Notes
- Tested on: [OS list]
- Not tested on: [OS list]

### Checksums
sha256: [hash]  omriss-X.Y.Z.tar.gz
```

---

## Final Sign-Off

Release requires confirmation from the product owner before publication.
Automated CI may produce artifacts; only the product owner may authorise
their public distribution.

**Sign-off:**
```
Product owner: _____________________  Date: ___________
Technical reviewer: architect         Date: 2026-11-03
```

### Technical reviewer's record — 0.17.0

What was verified, so the signature means something specific:

| | |
|---|---|
| Gates, re-run at `6c0d59f` | `cargo test --workspace` 528 passed / 0 failed / 1 ignored; `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `scripts/check-rfcs.sh` passed; `mdbook build docs` clean |
| Version consistency | `Cargo.toml` 0.17.0, `AppxManifest.xml` `Version="0.17.0.0"`; `release.yml`'s verify job gates the tag against both |
| Platform runs | Linux reduced run A–I (dev team, Task 015/016); **Windows reduced run A–I by the owner on Windows 11 Home 25H2 with Japanese as the primary language**, so check H ran rather than being recorded as not run. macOS not run — and the artifact matrix no longer claims it |
| Findings from those runs | two, both from check D, both documented limitations with keyboard-reachable recovery, both dispositioned to RFC-059 §6.2a/§6.2b. No release-blocking condition from `RELEASE_CHECKLIST.md`'s own list was observed on either platform |
| Data-integrity work | RFC-065's seven boundary defects plus §2.7–§2.9, each reproduced by the architect before acceptance and each covered by a test that fails against the old code. The `#` predicate family is closed by construction: every `'#'`, `b'#'` and `0x23` in `crates/core/src` is classified in RFC-065 §2.9 |
| The one ignored test | `split_section`'s property, `#[ignore]`d with RFC-070 named as its owner, per RFC-066's criterion 6 |

**Not verified, and not claimed:** macOS, the packaged MSIX as installed from
the Store (the Windows run used `cargo run --release`), and script execution in
the preview under WebView2 (RFC-064 §9b explains why the escaping evidence
still carries).
