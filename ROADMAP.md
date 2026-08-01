# Roadmap

The authoritative roadmap, milestone definitions (M0–M15), and theme
breakdown live in the design documents:

- RFC index: [`rfcs/README.md`](rfcs/README.md)
- Lifecycle policy: [`rfcs/done/000-rfc-lifecycle-policy.md`](rfcs/done/000-rfc-lifecycle-policy.md)

Status snapshot (v0.16.0+): RFCs 000–052 and RFC-062 are implemented; M0–M10
themes are complete and M11 is in progress. The desktop MVP runs on
Linux/macOS/Windows (the desktop shell requires platform WebView libraries to
build — see `PLATFORMS.md`). The outline/tree UI uses `dioxus-swdir-tree`.

**M10 (UI role separation) is complete.** RFC-048 through RFC-051 shipped in
v0.16.0 with explicitly deferred residual scope; RFC-062 (crate package name
exchange) is implemented on `main` and awaits a release tag. Each carries its
deferral list in its Status field.

**M11 — Format Adapter Foundation is in progress.** The maintenance refactor and
RFC-052 (structured plain-text policy and its documentation) are complete on
`main`; RFC-053, the adapter architecture, is the active work. Remaining roadmap
work tracked in [`rfcs/README.md`](rfcs/README.md):

- M11: RFC-053, the document format adapter boundary in `omriss-core`.
- M12: RFC-054 through RFC-056, JSON, TOML, and the YAML feasibility spike.
- M13: RFC-057, multi-document workspace and tabs.
- M14: RFC-058, placement-complete Document Map creation controls.
- M15: RFC-060 and RFC-061, accessibility/screen-reader and non-technical-user
  validation.
- M10 follow-up: RFC-059, keyboard focus and pending-draft UX hardening.

The RFC index is authoritative for exact titles, lifecycle state, and next free
RFC number.
