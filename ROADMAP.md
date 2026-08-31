# Roadmap

The authoritative roadmap, milestone definitions (M0–M15), and theme
breakdown live in the design documents:

- RFC index: [`rfcs/README.md`](rfcs/README.md)
- Lifecycle policy: [`rfcs/done/000-rfc-lifecycle-policy.md`](rfcs/done/000-rfc-lifecycle-policy.md)

Status snapshot (v0.16.0+): RFCs 000–054, RFC-062, and RFC-063 are implemented;
M0–M11 themes are complete and M12 is under way. The desktop MVP runs on
Linux/macOS/Windows (the desktop shell requires platform WebView libraries to
build — see `PLATFORMS.md`). The outline/tree UI uses `dioxus-swdir-tree`.

**M10 (UI role separation) is complete.** RFC-048 through RFC-051 shipped in
v0.16.0 with explicitly deferred residual scope; RFC-062 (crate package name
exchange) is implemented on `main` and awaits a release tag. Each carries its
deferral list in its Status field.

**M11 — Format Adapter Foundation is complete.** RFC-052 (policy) and RFC-053
(the adapter boundary in `omriss-core`) are implemented on `main`.

**M12 is under way. JSON is done** — RFC-054 is implemented on `main` across ten
commits: `.json` files open, navigate, and edit, with byte preservation proven
per value and Markdown behavior unchanged throughout. RFC-063 (a command-line
file argument) was added mid-sequence so rendering could be verified without
synthetic input. Each RFC's Status field records what was deliberately left out.

## 0.17.0 is blocked on M12 hardening

**Verified so far.** CI runs on Linux and Windows (431 tests, both platforms
green — the first time the app has ever been built on Windows). The Linux smoke
run completed all 21 checklist steps with no release-blocking failure, including
external byte verification of a JSON save and a malformed-file recovery.

**Then a full-project audit found three release-blocking defects**, each
reproduced independently before acceptance:

- author-supplied HTML in a Markdown file executes inside omriss's own WebView
  (RFC-064);
- typing into a file that ends in a bare heading writes the text into the
  heading (RFC-065);
- moving a section in a file with no trailing newline destroys a heading
  (RFC-065).

The last two are release-blocking under `RELEASE_CHECKLIST.md`'s own list. All
three are localised fixes, and none indicates the architecture is wrong. They
reached an audit rather than a test because every one of the 431 tests is
example-based, which is what RFC-066 addresses.

**0.17.0 ship gate:**

| RFC | Scope | Why it gates |
|---|---|---|
| [RFC-064](rfcs/proposed/064-preview-html-sanitization-and-webview-trust-boundary.md) | preview sanitization, link schemes, CSP | the only defect whose blast radius leaves the open document |
| [RFC-065](rfcs/proposed/065-structural-operation-boundary-integrity.md) | five splice-boundary defects | two are release-blocking by the checklist |
| [RFC-066](rfcs/proposed/066-property-based-and-fuzz-testing.md) | two round-trip properties, JSON fuzz target | how we know RFC-065 is complete, not just aimed at five shapes |
| [RFC-067](rfcs/proposed/067-structure-projection-caching.md) §3.1–3.2 | revision-keyed structure cache | JSON costs ~25 ms/keystroke at 400 KB; it is the release's headline feature |

`SECURITY.md` is in place, which RFC-064 required before its finding could be
discussed publicly.

Also required before the tag: the **Windows smoke run**, which has not happened.
Windows is what the Microsoft Store ships, and CI proves the binary compiles,
not that it behaves.

## After 0.17.0

- **M12 remainder:** RFC-055 (TOML) and RFC-056 (the YAML feasibility spike).
  RFC-067 §3.1's cache should land before a second structured format, not after.
- **M13:** RFC-068 (durable writes and path integrity), RFC-069 (draft lifecycle
  and session guards), RFC-057 (multi-document workspace and tabs).
- **M14:** RFC-058, placement-complete Document Map creation controls.
- **M15:** RFC-060 and RFC-061, accessibility/screen-reader and
  non-technical-user validation.
- **M10 follow-up:** RFC-059, keyboard focus and pending-draft UX hardening —
  now also the home for the shortcut-delivery defects found by the Task 011
  smoke run, which share one root cause with the documented focus limitation.

The RFC index is authoritative for exact titles, lifecycle state, and next free
RFC number.
