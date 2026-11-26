# RFC-079: Presentation Foundation — Language, Direction, Scalable CSS, Layout Check

**Project:** omriss — Omriss Editor
**Milestone:** M16 — Readable and operable (0.19.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-075
**Related RFCs:** RFC-030, RFC-043, RFC-060, RFC-073, RFC-080, RFC-081, RFC-082, RFC-083

---

## 1. Summary

The groundwork that RFC-080 (readability), RFC-082 (right-to-left readiness)
and RFC-083 (settings) all stand on, done once:

1. the page declares its **language and direction**;
2. **one pass over `style.css`**: sizes that scale, sides that follow
   direction;
3. a **layout check** — the GUI test rules' level 2 — so "nothing overflows
   at any size, language, palette, window size or direction" is a test
   result, not a screenshot.

Planned in `.git-exclude/specs/omriss-plan-accessibility-keyboard-rtl-draft.md`
and approved by the owner on 2026-11-26.

## 2. Findings (2026-11-26)

| Finding | Where |
|---|---|
| **No `lang` attribute anywhere.** A screen reader reads the Japanese UI with an English voice, or the reverse | no `lang` in `crates/` |
| No `dir` attribute; `Locale` has no direction | `crates/ui/src/i18n.rs` |
| 66 `font-size` declarations in `px` against 4 in `rem`/`em`; 393 `px` values in all. A root size change would scale almost nothing | `crates/app/assets/style.css` (1031 lines) |
| 28 physical `left`/`right` declarations, 0 logical; Document Map indentation is an inline `padding-left` | `style.css`, `crates/components/src/document_map_pane.rs:296` |
| Layout is verified only by real-window captures (level 3): no automated check of overflow, clipping or target size exists | `scripts/gui/` |

## 3. Design

### 3.1 Language and direction on the root

The root element carries `lang` (BCP 47 tag of the active locale) and `dir`
(`ltr` or `rtl`, from `Locale::direction()`, added here and returning `ltr`
for both existing locales). Both change when the locale changes. Nothing else
in the tree sets `dir` except RFC-082's per-text `dir="auto"`.

### 3.2 One CSS pass

- **Type, spacing and control sizes in `rem`**, so the root font size alone
  scales the app (RFC-080's text size). Borders and hairlines stay in `px`.
- **Physical sides to logical ones:** `margin-left` → `margin-inline-start`,
  `left:` → `inset-inline-start:`, `text-align: left` → `start`, and the
  inline indentation to `padding-inline-start`.
- **Colours stay tokens** (RFC-030's guard already enforces it).
- **At the default size the result is identical to today.** Proven by the
  layout check (§3.3) comparing every measured box before and after — not by
  eye.

`style.css` may be split into files by area in the same pass if that makes
the result clearer; one file or several is the implementer's call, recorded in
the PR.

### 3.3 The layout check

**What it proves**, for every view × text size (RFC-080) × locale × palette ×
window size × direction (RFC-082's pseudo-RTL):

- no element overflows its container or the viewport horizontally;
- no text is clipped without the full text being available;
- every interactive target meets RFC-080's minimum size;
- WCAG 1.4.12's text-spacing values break nothing.

**Proposed mechanism:** each view rendered by Dioxus's server-side renderer
(the components crate is renderer-portable, RFC-075), loaded with
`style.css` into **headless Chromium**, measured by an inline script that
writes its result into the page, read back with `--dump-dom`. Chromium is the
engine of WebView2, the Windows WebView; WebKitGTK differences remain a
level-3 spot check.

**Decided first by a spike, bounded by questions, not time.** It answers:

1. Does the server-side render of each view match the desktop's structure?
2. Can the measurement run with nothing but Chromium and the renderer as a
   dev-dependency?
3. Does a deliberately introduced overflow fail it?

It **stops and reports** the moment it needs a browser driver, Node, or any
new runtime dependency. The owner then decides, and the fallback is a
smaller matrix run at level 3 through `scripts/gui/`.

## 4. Non-goals

- No visual change at the default size.
- No new setting — RFC-083 owns settings.
- No RTL locale — RFC-082.

## 5. Validation

- `lang` and `dir` present and correct for each locale, and updated on switch
  — level 2 test.
- The layout check's before/after comparison at the default size: no box
  moved.
- A source test: no `font-size` in `px` and no physical `left`/`right`
  property in `style.css` (allow-list for hairlines, stated with reasons).
- The spike's three answers, recorded in this RFC.

## 6. Acceptance criteria

- A screen reader reads each locale's UI in its own voice (checked once with
  Orca, and on Windows by the owner).
- `style.css` scales from one root size and has no physical sides.
- The layout check runs in CI, or the spike's report and the owner's decision
  on the fallback are recorded here.
