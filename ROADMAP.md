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

Every item traces to evidence this release produced — a smoke run, the
accessibility run, the property suite, the known-limitations audit, or an owner
decision taken during it. Nothing is here because it would be nice to have.
That rule is the scope's only defence when something else asks to join.

### Core

| # | Work | Why it is in, and why here |
|---|---|---|
| 1 | **RFC-075** component crate extraction | A pure move, cheapest while the tree is quiet right after a release. Everything below then lands in the new crate rather than being moved after it |
| 2 | ~~**RFC-064 H3** — wire the CSP~~ — **withdrawn, and not replaced** | Attempted 2026-11-04 and withdrawn (RFC-064 §9c): the policy leaves the window rendering, pixel-identical, and accepting no input, and `document::eval` is `eval`, so a third `unsafe-*` would be needed to keep every focus behaviour Task 019 and RFC-060 established. **This slot is empty on purpose.** The gap it was for is §9a's tracking pixel, now RFC-064 §9d's H4 and scheduled to 0.19.0 — the quick version of H4 was rejected for misleading a reader whose own image silently vanished |
| 3 | **RFC-071** structure-change warning | Blocks RFC-076, and closes the user-surprise half of RFC-066's P3. **Carries RFC-073 §4.1 (`t_args`) as its first PR** — its messages count things, `t()` cannot interpolate, and scheduling interpolation at item 9 was an ordering error |
| 4 | **RFC-076** editable plain file text | **Carries RFC-069 §2.1 as its first PR** — `commit_or_block` never resyncs the draft, so every navigation after a commit re-commits and pushes an undo record, and this feature's whole safety story is that undo works. **Moved up deliberately.** It replaces an entire document in one commit — the most dangerous thing in the backlog — so it takes the *most* exercise before the release ends, not the least. Its only hard dependency is RFC-071 |
| 5 | **RFC-069** draft lifecycle and session guards | The dirty indicator that stays lit beside "Saved", seen in every smoke run since Task 011, and the window-close guard unbuilt since v0.3.0 |
| 6 | **RFC-059** keyboard focus | §6.2a (a save stops the keyboard working), §6.2b (Ctrl+F never focuses its field), RFC-060's F-4 (focus restored nowhere after a dialog closes). The highest-frequency defect family in the product |
| 7 | **RFC-067 §3.2** format-aware panes | A malformed `.json` is told "Add headings with # to begin organizing"; a JSON row highlights while the pane shows something else |
| 8 | **RFC-077** search covers heading text | The application shows a heading, then answers that the text is not there |
| 9 | **RFC-073 §4.3** (§4.1 lands earlier — see item 3) | `Permission denied (os error 13)` reaching the user violates RFC-060 §5. The typed `OpenFailure` needs interpolation, so those two travel together. **Lifecycle: RFC-073 then moves to `done/` with §4.2, §4.4 and §4.5 recorded as deferred in its Status**, the way RFC-067 handled §3.2 — otherwise it is a proposed RFC that is half built, which the folder-is-state rule has no answer for |
| 10 | **RFC-060 P-1 and P-2** | A screen reader entering the editor announces the document as **"Dioxus app"**, and two landmarks have no name at all. Both are small, both tell the user something untrue, and neither has the cross-repository problem that keeps F-1 out |
| 11 | **RFC-070** + **RFC-066 §3.1b's P4** | The one `#[ignore]`d property, and the rename/join byte invariant no property covers. Both small; both close test debt this release created |

### Stretch — taken only if the core lands early

**RFC-072** (Linux artifacts and AUR), **RFC-068** (durable file writes),
**RFC-078** (setext H1/H2 level changes).

These are not core with an asterisk. Eleven items is at capacity for one
maintainer and one implementer, and a plan that lists fourteen while privately
expecting to drop three is not a plan — it is an overrun with a cover story.
None of the three has an observed user-facing defect behind it, which is what
makes them the right three to hold back.

If the core finishes early, take them in that order: RFC-072 first, because two
registered AUR packages are sitting empty and its blocker is gone.

### Where the owner is needed, and when

0.17.0's pace was set by owner availability, not by code. Two points need it,
and they must not be the same point:

1. **Mid-release: a Windows check of the CSP alone.** Item 2's failure mode is
   a blank window. Discovering that during the final run would force a code
   change after the freeze, which is the one thing the freeze exists to stop.
2. **End: the reduced run (A–I)** against the release candidate.

### The scope freeze, and its one exception

The freeze applies from the start this time, not from the moment someone
notices the release is late. 0.17.0's freeze worked because it existed before
it was needed.

**One exception, stated now so it is not negotiated later:** if the CSP fails
verification, *unwiring* it is permitted at any point. Removing something
unverified is not the same act as adding something unverified, and only the
second is what the freeze forbids. Shipping the CSP unverified is never the
answer to a late release.

### Out of scope, with reasons

| Deferred | Reason |
|---|---|
| **RFC-074** Flatpak | Five gates (G1–G5) unproven and Flathub's review is an external queue. It follows RFC-072's assets rather than racing them |
| **RFC-055 / RFC-056** TOML, YAML | A third format before the second is polished repeats 0.17.0's sequence in a worse order |
| **RFC-057** tabs, **RFC-058** creation controls | Capability work, and this is not a capability release |
| **RFC-073** §4.2, §4.4, §4.5 | Plurals, screen-reader labels and locale persistence deserve a theme, not a squeeze |
| **RFC-060 F-1**, P-3, P-4 | F-1's Document Map tree semantics live in **`dioxus-swdir-tree`**, a different repository on its own cadence; scheduling it here would hide that dependency rather than manage it. P-3 and P-4 need a design pass the release has no room for |
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
- **Privacy:** RFC-064 §9d's **H4** — remote images in the preview render as a
  placeholder until the reader asks for them, rather than being fetched
  silently. 0.19.0. The one-line version of this (drop them outright) was
  rejected on 2026-11-04: it would have left a user whose own document
  contained an image staring at nothing, with no statement that anything had
  been withheld.

The RFC index is authoritative for exact titles, lifecycle state, and the next
free RFC number.
