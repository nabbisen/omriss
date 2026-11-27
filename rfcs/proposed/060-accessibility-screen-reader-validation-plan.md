# RFC-060: Accessibility and Screen-Reader Validation Plan

**Project:** omriss — Omriss Editor
**Milestone:** M15 — Accessibility validation (proposed)
**Status.** Proposed — §10, P-1 to P-4, is 0.18.0's item 10: P-2 and P-4 landed (`112c280`); P-3 recorded (§11); P-1 landed in PR 2 (`79d1942`), with constant scripts in PR 3 (`c8a0ac0`). **P-1 to P-4 are done**; the RFC stays Proposed for F-1 and the validation runs.
**Document type:** Follow-up validation RFC
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-051
**Related RFCs:** RFC-027, RFC-028, RFC-029, RFC-030, RFC-059

---

## 1. Summary

This RFC owns the full accessibility and screen-reader validation program that
was originally listed inside the RFC-048/M10 QA package.

RFC-048/M10 keeps narrow smoke checks for obvious role-split regressions. It
does not claim that full assistive-technology validation has passed.

## 2. Motivation

Accessibility and screen-reader validation is broad enough to need its own
plan, evidence rules, severity policy, and acceptance decision. Keeping it as
embedded M10 paperwork risks either blocking the release indefinitely or closing
the pass too shallowly.

This RFC prevents accidental weak closure by making the deferred gate explicit
and source-controlled.

## 3. Scope

In scope:

- screen-reader smoke and workflow validation;
- landmark and accessible-name validation;
- dialog, menu, status, and error announcement checks;
- severity policy for accessibility findings;
- build/source identity requirements for each validation run;
- pass/fail/defer rules.

Out of scope:

- general keyboard-only QA already tracked by RFC-059;
- non-technical-user participant validation, owned by RFC-061;
- full WCAG conformance certification.

## 4. Validation Requirements

Each validation run must record:

- app version and commit/source identity;
- operating system;
- desktop runtime/WebView;
- assistive technology used;
- test fixture or document;
- tester role;
- pass/fail/blocked result for each scenario.

## 5. Required Scenarios

- Header/toolbar exposes useful accessible names.
- Document Map is discoverable as the structure/navigation region.
- Writing Area is discoverable as the primary editing region.
- Quick Actions is discoverable and dismissible.
- Show plain file text is discoverable and described as read-only.
- Row menus expose useful labels and disabled explanations.
- Dialogs announce purpose, field labels, primary actions, and safe cancel
  paths.
- Status/footer messages are exposed through live regions or equivalent
  assistive-technology feedback.
- Errors do not expose byte ranges, node IDs, parser internals, or raw system
  errors.
- Known limitations are announced or documented clearly enough that users are
  not misled.

## 6. Severity Policy

Blocking:

- assistive-technology users cannot open, edit, save, or recover from a common
  dialog;
- destructive action can be triggered without understandable context;
- focus disappears in a way that prevents recovery;
- status/error feedback for failed save or refused navigation is unavailable.

Major:

- important labels are unclear but the workflow remains possible;
- disabled controls lack enough explanation;
- known limitation is not discoverable.

Minor:

- wording polish, announcement order, or redundant labels.

## 7. Acceptance Criteria

- Required scenarios are run and recorded.
- No blocking accessibility finding is open.
- Critical/high accessibility risks are fixed or explicitly accepted by the
  owner with release-note coverage.
- Any deferred findings are tracked in source control.
- Documentation and release notes do not claim full accessibility validation
  before this RFC is complete.

## 8. Relationship To RFC-048/M10

RFC-048/M10 may proceed with only narrow smoke checks if:

- role labels are visible and plain;
- no known new critical/high accessibility issue is open;
- known limitations are documented;
- this RFC remains open as the owner of full accessibility/screen-reader
  validation.

RFC-048/M10 must not mark the full accessibility/screen-reader pass as passed.

---

## 9. First Measurement Run — 2026-10-24

Run at commit `1509aae`, WebKitGTK 2.54.1 / GTK 3.24.52 / at-spi2-core 2.62.1,
**AT-SPI tree only — Orca was not run**. Instruments, 28 captured trees and the
event logs are in `rfcs/handoffs/060-accessibility-screen-reader-validation-plan/`.
This section exists because §7 requires deferred findings to be tracked in
source control, and a decision record outside the repository is not that.

**No §6 Blocking condition was observed.** omriss is exposed to assistive
technology with real structure: every dialog is a named modal with focus
inside, every landmark but two is named, row-menu items carry their disabled
reasons as descriptions, failed-save feedback is assertive and carries its
recovery action, and there is no unnamed `button`, `entry`, `heading` or
`menu item` in any captured state.

### 9.1 Findings and their owners

| # | Finding | Disposition |
|---|---|---|
| **F-1** | The Document Map is not a tree to assistive technology: no `role="tree"`/`treeitem`, no accessible name per row, and **no node in any state reports `selected`**. Every row's menu trigger is named identically ("Section actions"), so nine sections give nine indistinguishable controls | §9.2 — the markup comes from `dioxus-swdir-tree`, a separate repository |
| **F-3** | Confirm-delete focuses the destructive button on open | §9.3 — **fixed before 0.17.0** |
| **F-4** | Focus is not restored when a dialog or the palette closes | RFC-059 §6.2 |
| **F-5** | `Permission denied (os error 13)` reaches the user and the AT | RFC-073 §3.2 |
| **F-6** | A successful save can produce no AT event at all; the dirty indicator's name can contradict the status | RFC-059 §6.4, RFC-069 §2.1 |
| **F-7** | A malformed `.json` shows a Markdown instruction and no error | RFC-067 §3.2 |
| **P-1** | The web document's accessible name is `Dioxus app` | §9.4 |
| **P-2** | The footer (`contentinfo`) is unnamed in every state; `main` is unnamed whenever nothing is focused | §9.4 |
| **P-3** | Menu triggers are exposed as `combo box`; the Settings popup declares `role="menu"` but contains a `<select>` | §9.4 |
| **P-4** | The toolbar is named `menu.file` — "File", a menu that does not exist | §9.4 |

### 9.2 F-1 crosses a repository boundary

The Document Map's rows are rendered by **`dioxus-swdir-tree`**, a crate under
the same ownership as omriss (`github.com/nabbisen/dioxus-swdir-tree`,
Apache-2.0). Its `role` identifiers are *glyph* roles in its theme; it emits no
ARIA at all.

So F-1 is not a limitation to work around and not a line omriss can add: it is
a feature in a second repository — `role="tree"`, `treeitem`, `aria-selected`,
`aria-expanded`, `aria-level`, and a per-row accessible name — followed by a
version bump here. That sequencing is the reason this is recorded rather than
scheduled: it needs a release of another crate.

The workflow remains possible without it (the Subsections buttons and the
breadcrumb are named and operable), which is why it is Major and not Blocking.
But `omriss-external-design` describes "a keyboard-operable tree", and to
assistive technology there is no tree.

### 9.3 Dialog defaults are a conformance failure, and were fixed before the tag

`omriss-external-design` §11 states that shipped dialogs "trap focus and
**default to the safest action**", naming Delete Section and File Changed on
Disk among them. Two did not:

| Dialog | Focused on open | If the user presses Enter |
|---|---|---|
| Delete Section | **Delete** | the section and its children are deleted; Undo recovers them |
| File Changed on Disk | **Overwrite** | another program's changes on disk are overwritten; **omriss's Undo does not bring them back** |

The second was not in the measurement run — it was found while verifying the
first — and it is the worse of the two, because the data it destroys was never
in omriss's history.

Neither is §6 Blocking: both dialogs describe their consequence, so the action
is not available "without understandable context". Both are a recorded
decision not being met, which is a different and more fixable thing than a
design question. Dispatched and fixed for 0.17.0.

### 9.4 P-1 … P-4, and a note on the obvious fix for P-1

These are Minor: wording and mapping, not workflow. They are grouped here
because they want one design pass rather than four patches.

For P-1 specifically, the obvious fix does not work. `dioxus-desktop` ships
`<title>Dioxus app</title>` in its index and inserts `custom_head` immediately
before `</head>` — *after* that title. HTML takes the **first** `<title>` in
tree order, so `Config::with_custom_head("<title>Omriss</title>")` leaves the
accessible name unchanged. The document title has to be set at runtime, or the
whole index replaced.

### 9.5 What this run did not establish

Orca was not run, so nothing here was *heard*: every statement about
announcement is inference from the tree an AT receives. Event capture was
intermittent and is reported as observations, not as a log. One platform, one
WebKit build. No participant — that remains RFC-061's.

The next run should either route Orca's speech or state again that it did not,
rather than let "Orca not run" harden into a permanent footnote.

## 10. P-1 to P-4, designed (2026-11-27)

One pass, as §9.4 asked. The rule for all four: **what assistive technology is
told is true, comes from the catalog, and is given by construction rather
than one element at a time.**

1. **P-1, the document's name.** One function gives the title, and it names
   both the window and the web document. The title is "Omriss" with no file
   open, and "notes.md — Omriss" with one: the file's name, not its path,
   because the status bar shows the path. Unsaved changes are not shown in the
   title; how they are named is RFC-080 §5.4 rule 1. Dioxus's
   `document::Title` is the expected means, because it sets the title at
   runtime and so avoids §9.4's first-`<title>` trap. **Measure before
   relying on it:** confirm with AT-SPI that the document's name changes, and
   check what it does to the window title, which `main.rs` currently sets to a
   fixed "Omriss". If one mechanism cannot reach both, stop and report. Two
   titles that disagree are the fault this fixes.
2. **P-2, landmarks.** Every landmark omriss renders has a name from the
   catalog, in every view. The main region is always "Writing Area": in the
   welcome view, the overview, plain file text and the unreadable view, as it
   already is in focus. The section's own name is its heading, inside. The
   footer is "Status bar". A level-2 test mounts each view and checks that
   every `main`, `header`, `footer`, `nav` and `aside`, and every
   `role="region"` and `role="toolbar"`, has a name. A landmark added later
   without one fails it.
3. **P-3, menu triggers.** The settings popup's `role="menu"` was removed by
   RFC-059 PR 3. Measure the row menu trigger ("…") again with AT-SPI. If it
   still reports `combo box`, find out whether omriss's markup causes that or
   WebKitGTK maps every `aria-haspopup` button that way. The markup is fixed
   only if omriss's markup is the cause. A platform mapping is recorded here,
   not fought.
4. **P-4, the toolbar's name.** "Toolbar" / 「ツールバー」, from its own key.
   It stops borrowing `menu.file` ("File", a menu that does not exist).

**Validation.** The level-2 landmark test, with a deliberate break. AT-SPI in
the GUI, `en_US` and `ja_JP`: the document's name with and without a file;
every landmark's name in the welcome, overview, focus and plain-file-text
views; P-3's measurement.

## 11. Decisions on PR 1 (2026-11-27)

P-2 and P-4 are accepted. Every landmark in every view has a name, in both
locales, and a level-2 test pins it. The toolbar is "Toolbar", and the dead
`menu.file` key is gone.

**P-1: the dev team stopped correctly. The design is completed here.**
Measured: `document::Title` sets the native window's title and never
`document.title`. No single framework call names both. §10.1's intent was one
**source** of the title, so that the two never disagree, not one call.
Therefore:

1. **One title, in one place.** A function gives it ("Omriss", or
   "notes.md — Omriss"), from one constant for the product name. One component
   applies it to both targets, from the same string, in the same render: the
   window through `document::Title`, the page through `document.title`. No
   other code sets either.
2. **Never "Dioxus app", not even at start.** The page's first name comes from
   the same constant: `custom_head` sets `document.title` as the page is
   parsed, before any effect runs. `main.rs`'s fixed window title uses the same
   constant.
3. A source-scanning guard: no `document.title` and no `document::Title`
   outside that component, except the start-up line of item 2.

**P-3 is the platform.** On WebKitGTK 2.54.1, any `aria-haspopup` other than
`false` is exposed as a combo box, as a bare page with no omriss markup shows.
omriss's markup is correct ARIA: it says a menu opens, and that is true. It
stays. A Linux screen-reader user will hear "combo box", so
`known-limitations.md` says so, names the platform as the cause, and says the
button opens a menu.

## 12. Decisions on PR 2 (2026-11-27)

Accepted. Measured by a poller started before launch, in both locales:
- the page is "Omriss" from its first name;
- with a file open, the window and the page both read "notes.md — Omriss";
- "Dioxus app" is never seen.

The window takes the new title about 0.05 s before the page. That is accepted:
both names come from one string, in one render.

Also accepted:
- `welcome.rs` takes its heading from `PRODUCT_NAME`;
- the unused `app.title` key is removed.

**PR 3. JavaScript source is never built from data.** The page title is the
first `document::eval` whose source contains text from outside the program: a
file's name, which anyone who can name a file chooses. Its escaping is correct
today, but that correctness depends on a hand-written escaper at a security
boundary. The rule, by construction:

1. **Every script omriss runs in the page is a constant.** Data reaches a
   script as data, through the eval channel (`send` and `dioxus.recv()`), and
   is never spliced into source. The title component sends the title string.
   `js_string` and the code-building go.
2. **The start-up script is a constant too**, made from `PRODUCT_NAME` at
   compile time, or written out with a test that it equals the constant.
3. **A guard:** in the UI layer's sources, every `document::eval(` argument is
   a string literal or a `const`. A `format!`, a variable or a `&String` fails
   it. `focus.rs`'s calls already satisfy it.
