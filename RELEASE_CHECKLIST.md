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

### Release-Blocking Failures (RFC-038 §4)

Any of the following block release:

- app fails to launch on a supported platform
- open/save corrupts unrelated bytes in the Markdown file
- Ctrl+S / Cmd+S cannot save
- keyboard navigation traps the user (no Esc/Tab escape)
- save failure falsely reports success
- documented source-preservation invariant violated

### Smoke Test Evidence

```
OS: 
Version: 
Artifact: omriss-X.Y.Z.tar.gz
Fixture: academic-paper.md
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
