# Roadmap

The authoritative roadmap, milestone definitions (M0–M15), and theme
breakdown live in the design documents:

- RFC index: [`rfcs/README.md`](rfcs/README.md)
- Lifecycle policy: [`rfcs/done/000-rfc-lifecycle-policy.md`](rfcs/done/000-rfc-lifecycle-policy.md)

Status snapshot (v0.16.0+): RFCs 000–054, RFC-062, and RFC-063 are implemented;
M0–M11
themes are complete. The desktop MVP runs on
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

**Not yet released.** JSON reaching users requires a rendering check on the
actual release build — the owner's remaining step.

**The next theme is not yet chosen.** Remaining roadmap work tracked in
[`rfcs/README.md`](rfcs/README.md):

- M12 remainder: RFC-055 (TOML) and RFC-056 (the YAML feasibility spike).
- M13: RFC-057, multi-document workspace and tabs.
- M14: RFC-058, placement-complete Document Map creation controls.
- M15: RFC-060 and RFC-061, accessibility/screen-reader and non-technical-user
  validation.
- M10 follow-up: RFC-059, keyboard focus and pending-draft UX hardening.

The RFC index is authoritative for exact titles, lifecycle state, and next free
RFC number.
