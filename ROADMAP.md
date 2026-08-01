# Roadmap

The authoritative roadmap, milestone definitions (M0–M15), and theme
breakdown live in the design documents:

- RFC index: [`rfcs/README.md`](rfcs/README.md)
- Lifecycle policy: [`rfcs/done/000-rfc-lifecycle-policy.md`](rfcs/done/000-rfc-lifecycle-policy.md)

Status snapshot (v0.16.0+): RFCs 000–053 and RFC-062 are implemented; M0–M11
themes are complete. The desktop MVP runs on
Linux/macOS/Windows (the desktop shell requires platform WebView libraries to
build — see `PLATFORMS.md`). The outline/tree UI uses `dioxus-swdir-tree`.

**M10 (UI role separation) is complete.** RFC-048 through RFC-051 shipped in
v0.16.0 with explicitly deferred residual scope; RFC-062 (crate package name
exchange) is implemented on `main` and awaits a release tag. Each carries its
deferral list in its Status field.

**M11 — Format Adapter Foundation is complete.** RFC-052 (policy) and RFC-053
(the adapter boundary in `omriss-core`) are implemented on `main`. The boundary
serves Markdown with byte-for-byte unchanged behavior; nothing is wired into
`EditorSession` yet. RFC-053's Status field records three items carried into
RFC-054, one of which — the Markdown-specific `&mut Document` mutation
signature — must be resolved before a second format can be implemented.

**The next theme is not yet chosen.** Remaining roadmap work tracked in
[`rfcs/README.md`](rfcs/README.md):

- M12: RFC-054 through RFC-056, JSON, TOML, and the YAML feasibility spike.
- M13: RFC-057, multi-document workspace and tabs.
- M14: RFC-058, placement-complete Document Map creation controls.
- M15: RFC-060 and RFC-061, accessibility/screen-reader and non-technical-user
  validation.
- M10 follow-up: RFC-059, keyboard focus and pending-draft UX hardening.

The RFC index is authoritative for exact titles, lifecycle state, and next free
RFC number.
