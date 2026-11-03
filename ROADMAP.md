# Roadmap

The authoritative roadmap, milestone definitions (M0–M15), and theme
breakdown live in the design documents:

- RFC index: [`rfcs/README.md`](rfcs/README.md)
- Lifecycle policy: [`rfcs/done/000-rfc-lifecycle-policy.md`](rfcs/done/000-rfc-lifecycle-policy.md)

Status snapshot (v0.17.0, released 2026-11-03): RFCs 000–067 are implemented;
M0–M12 themes are complete. The desktop MVP runs on
Linux/macOS/Windows (the desktop shell requires platform WebView libraries to
build — see `PLATFORMS.md`). The outline/tree UI uses `dioxus-swdir-tree`.

**M10 (UI role separation) is complete.** RFC-048 through RFC-051 shipped in
v0.16.0 with explicitly deferred residual scope; RFC-062 (crate package name
exchange) shipped in v0.17.0. Each carries its
deferral list in its Status field.

**M11 — Format Adapter Foundation is complete.** RFC-052 (policy) and RFC-053
(the adapter boundary in `omriss-core`) are implemented on `main`.

**M12 is under way. JSON is done** — RFC-054 is implemented on `main` across ten
commits: `.json` files open, navigate, and edit, with byte preservation proven
per value and Markdown behavior unchanged throughout. RFC-063 (a command-line
file argument) was added mid-sequence so rendering could be verified without
synthetic input. Each RFC's Status field records what was deliberately left out.

## 0.17.0 — released 2026-11-03

JSON editing (RFC-052/053/054), a command-line file argument (RFC-063), and the
hardening that made them safe to ship: RFC-064 preview escaping, RFC-065
structural boundary integrity, RFC-066 property-based testing, RFC-067 §3.1's
structure cache. `omriss-core` and `omriss-ui` 0.17.0 are on crates.io under
the names RFC-062 settled; the Windows build ships as MSIX through the
Microsoft Store.

Four small fixes were taken after the feature work was "done" and before the
tag, then the scope was frozen. `RELEASE_CHECKLIST.md` carries the reviewer's
record of what was verified and what was not.

## 0.18.0 — scope

**Theme: close what 0.17.0's own validation found.**

Every item below traces to evidence produced by this release — a smoke run, the
accessibility run, the property suite, the known-limitations audit, or an owner
decision taken during it. Nothing is here because it would be nice to have.
That is the whole scoping rule, and it is what makes the list defensible when
something else asks to join it.

### Core, in dependency order

| # | Work | Why it is in, and why here |
|---|---|---|
| 1 | **RFC-075** component crate extraction | A pure move, cheapest while the tree is quiet right after a release. Everything below then lands in the new crate instead of being moved afterwards |
| 2 | **RFC-064 H3** — wire the CSP | **Its deferral has expired.** §9 deferred it because "0.17.0's Windows smoke run has not yet been performed"; it has been. The policy is verified on Linux, the failure mode is a blank window, and the third platform §9 worried about is macOS — which the artifact matrix no longer claims. Code it early, verify it on the release's own Windows run |
| 3 | **RFC-071** structure-change warning | Blocks RFC-076, and closes the user-surprise half of RFC-066's P3 |
| 4 | **RFC-069** draft lifecycle and session guards | The dirty indicator that stays lit beside "Saved" — seen in every smoke run since Task 011 — and the window-close guard, unbuilt since v0.3.0 |
| 5 | **RFC-059** keyboard focus | §6.2a (a save stops the keyboard working), §6.2b (Ctrl+F never focuses the field), and RFC-060's F-4 (focus restored nowhere after a dialog closes). The highest-frequency defect family in the product |
| 6 | **RFC-067 §3.2** format-aware panes | A malformed `.json` is told "Add headings with # to begin organizing"; a JSON row highlights while the pane shows something else |
| 7 | **RFC-077** search covers heading text | Reclassified from limitation to defect: the application shows a heading and then answers that the text is not there |
| 8 | **RFC-073 §4.1 + §4.3 only** | `Permission denied (os error 13)` reaching the user violates RFC-060 §5. The typed `OpenFailure` needs interpolation, so those two sections travel together; plurals, the screen-reader labels and locale persistence wait for 0.19.0 |
| 9 | **RFC-076** editable plain file text | Owner decision: an editor must let a user edit a plain text file. **After RFC-071**, never before |
| 10 | **RFC-070** + **RFC-066 §3.1b's P4** | The one `#[ignore]`d property, and the rename/join byte invariant that no property currently covers. Both small; both close test debt this release created |
| 11 | **RFC-068** durable file writes | The only item with no observed defect behind it. It is in because it is the save path, and RFC-074's G2 will need it |
| 12 | **RFC-072** Linux artifacts and AUR | Its blocker is gone — `Cargo.lock` is committed and the application ID is decided. Two registered AUR packages are sitting empty |

### The cut line, declared now

If the release runs long, drop in this order: **RFC-072, then RFC-068, then
RFC-078** (setext H1/H2, otherwise a stretch item). None of the three has an
observed user-facing defect behind it, which is exactly why they are the ones
to lose.

**RFC-064 H3 is not on that list and must not join it.** If its verification
fails on Windows, the answer is to leave the CSP unwired and say so in the
RFC — not to ship it unverified because the release is late.

Declaring the order now is the point. 0.17.0's scope freeze worked because it
existed before it was needed.

### Out of scope, with reasons

| Deferred | Reason |
|---|---|
| **RFC-074** Flatpak | Five gates (G1–G5) are unproven and Flathub's review is an external queue. It should follow RFC-072's assets rather than race them |
| **RFC-055 / RFC-056** TOML, YAML | A third format before the second one is polished repeats 0.17.0's sequence in a worse order |
| **RFC-057** tabs, **RFC-058** creation controls | Capability work, and this release is not a capability release |
| **RFC-073** §4.2, §4.4, §4.5 | Plurals, screen-reader labels and locale persistence deserve a themed release, not a squeeze |
| **RFC-060** remediation beyond F-4/F-5 | §9.2's Document Map tree semantics live in **`dioxus-swdir-tree`**, a different repository on a different release cadence. Scheduling it here would hide that |
| **RFC-061** | Needs participants, not code |

### What this release is, named honestly

**0.18.0 is a polish release.** RFC-076 is its only new capability. Everything
else makes what already exists behave the way the documentation already claims.
That is the right shape after a feature release whose own validation produced
this much — but it should be called what it is, so nobody opens the changelog
expecting a headline.

## After 0.18.0

Only work that 0.18.0 deliberately leaves alone. Anything already in its scope
table is not repeated here.

- **Distribution:** RFC-074 (Flatpak and Flathub), after RFC-072's assets
  exist. Its G1–G5 gates and Flathub's review queue are why it does not travel
  with RFC-072.
- **Microsoft Store publishing automation**, deferred by owner decision on
  2026-11-03 after 0.17.0's `publish` job failed on credentials that were
  never configured. Azure Entra and Partner Center need more preparation than
  a release can absorb mid-flight. Until it is taken up, `release.yml` builds
  and uploads the MSIX and **skips** publishing unless the repository variable
  `STORE_PUBLISH` is `true`; submission is a manual step on the checklist.
  Worth noting for whoever picks it up: 0.17.0's MSIX is a complete, verified
  artifact, and the failure was never in the packaging.
- **Formats:** RFC-055 (TOML) and RFC-056 (the YAML feasibility spike) — M12's
  remainder, deliberately behind the polish of the format already shipped.
- **Capability:** RFC-057 (multi-document workspace and tabs) and RFC-058
  (placement-complete Document Map creation controls). A recent-files
  reachability RFC would belong here too if the owner takes it up.
- **Localization:** RFC-073 §4.2, §4.4 and §4.5 — plurals, the screen-reader
  labels and locale persistence. §4.1 and §4.3 are in 0.18.0 because a raw OS
  error reaching the user is a conformance failure; the rest is a theme of its
  own.
- **Accessibility:** RFC-060's remaining findings and RFC-061. **RFC-060 §9.2
  crosses a repository boundary** — the Document Map's tree semantics are a
  feature of `dioxus-swdir-tree`, under the same ownership but on its own
  release cadence. Scheduling it inside an omriss release would hide that
  dependency rather than manage it.
- **Portability:** a web host built on RFC-075's extraction, which RFC-075 §7
  gates behind a threat-model audit RFC-064's assumptions do not survive.

The RFC index is authoritative for exact titles, lifecycle state, and the next
free RFC number.
