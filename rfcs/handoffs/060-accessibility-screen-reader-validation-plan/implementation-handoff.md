# RFC-060 — Implementation Handoff

Related RFC: `../../proposed/060-accessibility-screen-reader-validation-plan.md`

Lifecycle status: inherited from RFC-060.

---

## 1. Why this can start now

RFC-060 is scheduled for M15 on the assumption that screen-reader validation
needs equipment and a participant. **The first measurement needs neither.**

Orca reads one data structure — the AT-SPI accessible tree — and that tree can
be dumped from a script in seconds. What remains genuinely M15 work is
participant validation (RFC-061's territory) and anything that depends on
hearing announcements rather than reading the tree they are derived from.

## 2. What the first probe established

Run by the architect on 2026-10-22 against a release build, with
`gi.repository.Atspi` — the library Orca itself uses. Full output:
`baseline-accessible-tree.txt`.

**omriss is exposed to assistive technology, and substantially.** It registers
on the a11y bus as `omriss` [application], toolkit `gtk 3.24.52`, and the
WebView's content reaches the tree with real structure:

```text
[landmark] 'Document Map'   [landmark] 'Writing Area'   [landmark] 'Subsections'
[tool bar] 'File'   [button] 'Open…' focusable   [heading] 'On Source Preservation…'
[entry] 'Edit section text' editable,focusable
```

A blank WebView was the failure mode worth fearing. It is not what we have: the
`aria-*` attributes and roles authored in `crates/app/src/components/` survive
into the tree.

**This is reference, not evidence.** The binary predates `e03768c`, one
document state was probed, and a single dump is not a validation run under
RFC-060 §4.

## 3. Four observations to confirm or refute

Do not treat the architect's reading as settled; each needs its own evidence.

| # | Observation | Why it may matter |
|---|---|---|
| **P-1** | `[document web] 'Dioxus app'` | The document's accessible name is a framework default. A screen reader entering the WebView announces **"Dioxus app"** — the name of our toolkit, to a user who has never heard of it. Find where it comes from |
| **P-2** | `[landmark] ''` — the last landmark has no accessible name | Orca announces landmarks by name. An unnamed one is announced as "landmark" and nothing else. Identify the region (probably the footer) |
| **P-3** | `[combo box] 'Settings'`, `[combo box] 'Section actions'` | Authored as menus. If AT-SPI sees `combo box`, the announced interaction model will not match what the control does |
| **P-4** | `[tool bar] 'File'` | The toolbar's accessible name is "File" — a menu that does not exist. Check whether it is deliberate |

Related but **not** part of this run: `breadcrumb.rs:46` carries
`"aria-label": "collapsed sections"`, an untranslated screen-reader-only
string. Recorded in RFC-073 §4.4.

## 4. The instrument

`atspi_probe.py` registers as a read-only AT-SPI client and prints role,
accessible name and the interesting states for every node.

```sh
python3 -I rfcs/handoffs/060-accessibility-screen-reader-validation-plan/atspi_probe.py omriss
```

It is a QA tool, not shipped code: improve it freely, in place.

Launch the application as in Task 015/016 — `GDK_BACKEND=x11 DISPLAY=:1`, an
isolated `XDG_CONFIG_HOME`, **copies** of the fixtures, tracked fixture hashes
re-verified at the end. **Build at current `main` first**; a stale binary makes
the run reference rather than evidence, which is exactly §2's limitation.

### On running Orca itself

Orca's announcements are *derived from this tree*, so a defect in the tree is a
defect in what Orca says. The converse does not hold — a correct tree can still
be announced badly — so an Orca pass is worth doing if its speech can be routed
or silenced, and worth skipping rather than botching. If it is skipped, say so
in the report rather than implying coverage.

**Do not start Orca on the owner's primary session.** It will talk.

## 5. Constraints

1. **Measure; do not fix.** Not one line of `crates/`. A fix made while
   measuring destroys the measurement — the Task 015 rule, for the same reason.
2. Do not edit `RELEASE_CHECKLIST.md` or any RFC. Findings go in the report;
   the architect records them against owning RFCs.
3. Copies of fixtures only; tracked hashes re-verified at the end.
4. Trees and any helper output stay in this handoff directory.

## 6. Out of scope

- **Flatpak.** The sandbox adds two further conditions and no Flatpak build
  exists yet. That is RFC-074 gate G5.
- **Windows / Narrator.** The owner's platform.
- **Keyboard-only navigation defects** already owned by RFC-059 and
  `docs/src/known-limitations.md`. Record something here only if it is specific
  to assistive technology.
- Any WCAG conformance claim — RFC-060 §3 excludes it.

## 7. Escalation

If any RFC-060 §6 **Blocking** condition is observed — an assistive-technology
user cannot open, edit, save or recover from a dialog; a destructive action is
reachable without understandable context; focus disappears unrecoverably;
failed-save feedback is unavailable — stop and report it before finishing the
rest.

That would make accessibility a 0.17.0 question rather than an 0.18.0 one, and
the owner needs to know before the tag, not after.
