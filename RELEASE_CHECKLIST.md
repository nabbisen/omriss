# Release Checklist

This checklist must be completed before any public release of omriss.
Per RFC-042 §5, public release requires explicit confirmation from the
product owner. No automated process may bypass this gate.

---

## Pre-Release Gate

- [ ] All workspace tests pass: `cargo test --workspace`
- [ ] No open issues labelled `release-blocker`
- [ ] `CHANGELOG.md` entry written for this version
- [ ] Version bumped in `Cargo.toml` workspace package
- [ ] RFC index up to date: `./scripts/check-rfcs.sh`
- [ ] Package roles verified before publication:
      `omriss-core` is the publishable library package, and the `omriss` app
      package remains `publish = false` unless an explicit release review
      authorises crates.io app distribution.

---

## Data Integrity Tests (RFC-040, RFC-042 top criterion)

- [ ] Source-preservation golden tests pass
- [ ] Fixture catalog tests pass (all fixtures)
- [ ] Structural operation tests pass
- [ ] Undo round-trip tests pass for all operation types

---

## Artifact Matrix (RFC-037)

| OS | Artifact | Status |
|----|----------|--------|
| Any | `omriss-X.Y.Z.tar.gz` (Cargo source) | ✓ produced by release process |
| Linux | Binary tarball (future) | planned |
| macOS | App bundle zip/dmg (future) | planned |
| Windows | Zip/MSI (future) | planned |

For source releases, include checksums:

```sh
sha256sum omriss-X.Y.Z.tar.gz > omriss-X.Y.Z.tar.gz.sha256
```

---

## Platform Smoke Tests (RFC-038)

Run the following workflow on each supported platform before release.
Record pass/fail and the OS version tested.

### Smoke Workflow

1. **Launch** — app opens without error
2. **Open** — open `tests/fixtures/academic-paper.md` via Ctrl+O
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
OS: 
Version: 
Artifact: omriss-X.Y.Z.tar.gz
Fixtures: academic-paper.md, structured-sample.json
Date: 
Tester: 
Steps completed: 
Failures (if any): 
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
Technical reviewer: _________________  Date: ___________
```
