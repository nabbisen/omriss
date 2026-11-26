# RFC-082: Right-to-Left Readiness

**Project:** omriss — Omriss Editor
**Milestone:** M17 — Ready for every direction (0.20.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-079
**Related RFCs:** RFC-043, RFC-073, RFC-080, RFC-081

---

## 1. Summary

**Readiness, not a release of Arabic:** when an RTL language such as Arabic
is added, it is a translation task, with nothing to fix in the code. No RTL
locale ships until a translator is available (owner, 2026-11-26: "accepted
for now").

One part helps users today: document text is shown in its own direction,
whatever the UI language.

RFC-043 recorded RTL as "tracked as a future RFC when an RTL locale is
added"; this is that RFC, written ahead of the locale on purpose.

## 2. Findings (2026-11-26)

| Finding | Where |
|---|---|
| No `dir` on the page; `Locale` has no direction | `crates/ui/src/i18n.rs` |
| 28 physical `left`/`right` CSS declarations; inline `padding-left` indentation | `style.css`, `document_map_pane.rs:296` — addressed by RFC-079 |
| Direction-bearing glyphs in UI text: `← Previous`, `Next →`, breadcrumb `›`, `Alt+←` | i18n catalogues, `breadcrumb.rs` |
| **Document text takes the UI's direction**: an Arabic or Hebrew Markdown file is laid out left-to-right | editor, titles, Document Map rows, breadcrumb, preview, search results |
| RFC-073 §4.2 designs a **two-category** plural selector; Arabic has six (zero, one, two, few, many, other) | RFC-073 §4.2 |

## 3. Design

1. **Direction from the locale** — `Locale::direction()` (RFC-079) through
   the root `dir`.
2. **Document text in its own direction, now:** `dir="auto"` on every element
   that shows document text — the section editor, raw editor, titles,
   Document Map rows, breadcrumb crumbs, preview blocks, search results. An
   Arabic document reads correctly in the English UI. Bytes are unaffected:
   direction is display only.
3. **Mirroring:** glyphs that point come from the direction, not from the
   catalogue; the Document Map's Left and Right (collapse and expand) swap in
   RTL, before reaching `dioxus-swdir-tree`'s key handling; history keys
   (Alt+←/→) keep the convention browsers keep — confirmed by RFC-081's study.
4. **A pseudo-RTL locale, for testing only:** English text with `dir="rtl"`,
   selected by an environment variable and **never offered in the language
   menu**, where a user would take it for a defect. RFC-079's layout check
   and `scripts/gui/` run it.
5. **Plurals:** RFC-073 §4.2 is redesigned **before** it is built: CLDR plural
   categories with a hand-written rule per locale (still no ICU dependency),
   so a six-category language needs only its rule and strings.
6. **Fonts:** system fonts; the stack names Arabic-capable fallbacks. No
   bundled fonts.

## 4. Non-goals

- An Arabic (or any RTL) translation.
- Bidirectional editing beyond what the WebView's text controls do natively.

## 5. Validation

- Pseudo-RTL through the layout check: mirrored, nothing overflowing, no
  physical-side leftovers.
- Fixtures with Arabic and Hebrew headings and bodies: rendered right-to-left
  in the English UI; bytes unchanged after edit and save.
- Plural rules: unit tests per CLDR category for each shipped locale, and a
  test locale with six categories.

## 6. Acceptance criteria

- Adding an RTL locale requires only a catalogue, a plural rule and a
  `direction()` entry — demonstrated by the pseudo-RTL locale.
- RTL document text displays correctly in every UI language.
