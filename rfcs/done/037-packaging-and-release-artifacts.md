<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-037: Packaging and Release Artifacts

**Project:** omriss — Omriss Editor  
**Milestone:** M8 — Cross-Platform Delivery  
**Status.** Implemented (v0.11.0)

> **Superseded in part, 2026-10-21.** §3's "No app store distribution
> requirement" no longer holds: omriss ships on the Microsoft Store, and two AUR
> packages are registered. The artifact matrix is also inaccurate — it lists a
> Cargo source tarball as "produced by the release process" and Linux/macOS/
> Windows binaries as "planned", while `release.yml` produces only a Store MSIX
> and no checksum for anything (AUDIT-0170-016).
>
> GitHub releases stopped at `0.6.1` (2025-02-08), under the former name
> `layered`, which did publish three-platform binaries. The last 46 tags have no
> release. **RFC-072** rebuilds the Linux half of this in CI and corrects the
> matrix; the Store channel is recorded in RFC-064 §9 and
> `.git-exclude/release/0.17.0/ci-and-store-publishing.md`.  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  

---

## 1. Summary

Define release artifact types, versioning, and distribution expectations.

## 2. Goals

- Create artifact matrix by OS.
- Define versioning and release notes.
- Document unsigned build limitations if applicable.
- Support reproducible-ish build process where feasible.

## 3. Non-Goals

- No paid code-signing requirement in early planning.
- No app store distribution requirement.
- No auto-update system.

## 4. Design

### Artifact Matrix

| OS | Candidate Artifacts |
|---|---|
| Linux | tar.gz, AppImage or distro package later |
| macOS | app bundle zip/dmg later |
| Windows | zip/msi later |

### Versioning

Use semantic versioning for public releases. Pre-1.0 versions may introduce breaking changes but must document data/file safety expectations.

### Release Notes Template

```text
Version
Highlights
Data integrity notes
Known limitations
Platform notes
Checksums
```

### Unsigned Build Policy

If builds are unsigned, documentation must clearly explain expected OS warnings and safe verification steps such as checksums.

## 5. Validation and Test Plan

- CI produces expected artifacts.
- Artifact launches smoke app where possible.
- Checksums generated.
- Release notes include known limitations.

## 6. Acceptance Criteria

- Users know which artifact to download.
- Release process does not imply unsupported trust guarantees.
- Packaging does not alter core file-preservation behavior.

## 7. Dependencies

- RFC-035
- RFC-038
- RFC-042

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
