# RFC-080: Readable for Older and Low-Vision Users

**Project:** omriss — Omriss Editor
**Milestone:** M16 — Readable and operable (0.19.0); §5.6 large-size layout M17 (0.20.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, UI/UX designer, Rust developer, QA engineer
**Depends on:** RFC-079, RFC-083
**Related RFCs:** RFC-030, RFC-059, RFC-060, RFC-061, RFC-081

---

## 1. Summary

Make omriss easy to see and operate for older people and people with low
vision, **without choosing one look for everyone**: follow each person's
system, let them adjust what the research says people need to adjust, keep
today's look as the default, and apply a fixed set of clarity rules that
remove ways to be confused.

Studied from the sources in §9 and approved by the owner on 2026-11-26
(plan: `.git-exclude/specs/omriss-plan-accessibility-keyboard-rtl-draft.md`).

## 2. Who this is for

- By 80, contrast sensitivity is typically 80% lower than at 20; blues and
  greens become harder to tell apart than reds and yellows [WAI-AGE].
  omriss's accent, links and focus ring are all blue.
- Significant vision loss: 16% at 65–74, 46% at 85+; tremor, arthritis and
  Parkinson's make small targets and precise pointing hard [WAI-AGE].
- Older users succeeded on 55% of tasks against 75% for younger users, made
  twice the errors, blamed themselves 90% of the time, and half kept written
  step lists for software they used [NNG].
- The best contrast and brightness differ between people, and for one person
  with fatigue and lighting [LV].

## 3. Findings (2026-11-26)

| Finding | Where |
|---|---|
| Base text 14px; 36 `font-size` declarations at 12px or below; Quick Actions' shortcut hints 11px, grey, italic, monospace | `crates/app/assets/style.css` |
| No way to enlarge text; the saved `font_size` is read by nothing | `crates/app/src/storage/settings.rs` |
| No in-app colour choice; no high-contrast palette; nothing for Windows contrast themes (`forced-colors`) | `style.css:604` |
| Icon-only controls: Search `⌕`, Settings `⚙`, breadcrumb `↑`; unsaved changes shown by a red dot alone | `toolbar.rs`, `breadcrumb.rs` |
| Grey (`--muted`) used 46 times, including for status text, labels and hints | `style.css` |
| Toolbar icon buttons and row menus 24px | `style.css:217`, `:775` |
| **Plain file text: 12px monospace, never wraps** — sideways scrolling on every long line | `style.css:433-443` |
| "Zoom in" names moving into a section, in the UI and the docs | `crates/ui/src/i18n/en.rs:206`, `ja.rs:210`, `docs/src/keyboard-reference.md` |

## 4. Principle

**No look is best for everyone** (owner, 2026-11-26). Therefore:

1. **The person's system comes first** — colour scheme, contrast preference,
   Windows contrast themes, reduced motion, the OS text size.
2. **What people need to adjust is adjustable** (§5.1), each choice separate.
3. **Today's look stays the default.**
4. **No combination of settings can make the app unusable**: choices are
   bounded, and every combination is in RFC-079's layout check.
5. **Clarity is not a setting** (§5.4): those rules hold in every palette and
   size.

## 5. Design

### 5.1 User choices

Few on purpose: options that are hard to set are not used [COGA 4.7.4]; more
than about five main choices overload [COGA 4.6.3]. Presented by RFC-083.

| Choice | Values | Why |
|---|---|---|
| Text size | 100 / 125 / 150 / 175 / 200% — text, controls and spacing scale together | the most common need [WAI-AGE, NNG]; WCAG 1.4.4 |
| Colours | Same as my computer / Light / Dark / Light, high contrast / Dark, high contrast | light vs dark is personal; high contrast is not the same as dark [LV, MS] |
| Font for writing | Sans-serif / Serif / Monospace — system fonts, none bundled | readers need to choose the face [LV 3.3.2] |
| Line spacing | Normal / Relaxed / Loose | [LV 3.4.1; WAI-AGE G188] |
| Wrap long lines | on by default | one direction of scrolling [LV 3.2.1; WCAG 1.4.10] — §5.5 |

### 5.2 Following the system

- `prefers-color-scheme`, `prefers-contrast: more`, `prefers-reduced-motion`
  (already honoured), `forced-colors`.
- **The OS text size** (Windows: 100–225% [MS]) must be honoured, never
  overridden. Whether WebView2 and WebKitGTK pass it to the page is
  **unknown and measured first**. If they do, the in-app size multiplies it
  and the layout check covers the product.

### 5.3 Palettes and Windows contrast themes

- **Palettes are data**: a set of colour tokens per palette, each checked by
  RFC-030's contrast guard at its tier — 4.5:1 standard, **7:1** high
  contrast (WCAG 1.4.6). Four to start. Another palette later is a reviewed
  data change, added when a finding asks for it.
- **Windows contrast themes are not a palette.** Under `forced-colors:
  active`, the app uses the system colours (`Canvas`, `CanvasText`,
  `ButtonFace`, `ButtonText`, `Highlight`, `HighlightText`, `LinkText`;
  `GrayText` for disabled controls only) and stops drawing its own [MS].
  Tested in all four built-in Windows themes.

### 5.4 Clarity rules — every palette, every size

1. **Nothing by colour alone** [WCAG 1.4.1; GNOME; WAI-AGE G14]. The unsaved
   marker carries a word. This also settles RFC-059 §6.4: the draft not yet
   applied and the document not yet saved are named differently.
2. **Visible labels**; an icon only beside a label and with one meaning
   [COGA 4.2.7, 4.5.6]. Search, Settings and the breadcrumb's Up get words.
   A tooltip is never the only place a name appears.
3. **Nothing important behind hover or a hidden menu** [COGA 4.3.2, 4.3.4;
   NNG].
4. **Undecorated text:** no informational text in italic or all caps [WAI-AGE;
   LV 3.3.3–3.3.4]; long text never justified or centred [LV 3.4.4]; nothing
   informational below 0.875 × the base size.
5. **Grey is for de-emphasis only** [LV 2.4.3; MS]. Every use of `--muted` is
   reviewed; status text, labels and hints use the full text colour.
6. **Focus that cannot be missed and does not depend on blue:** the size WCAG
   2.4.13 describes, offset from the control, with a second contrasting edge.
7. **Targets** at least 32px at 100% and growing with text size (44px from
   about 137%, WCAG 2.5.5), with **space between adjacent targets** [NNG].
8. **Feedback near the user's point of regard** [LV 3.6.2]. Routine
   confirmations stay in the status bar (announced, `aria-live`). Anything
   that needs action — a failed save, a structure-change warning — appears
   beside the work it concerns and stays until dismissed or acted on.
   **Messages never blame the user** and say how to fix the problem [NNG;
   COGA 4.5.10]. Known cases: a failed save; a save the disk did not confirm
   (RFC-068 §9); a structure-change warning.
9. **Keep the user's place** when text size, colours or spacing change, and
   when a view is left and re-entered (plain file text, preview): the caret
   returns where it was and the focused section stays in view [LV 3.6.1].
   Found on Windows, 2026-11-26: leaving plain file text puts the caret at
   the end of the section. Likewise, reopening the search panel keeps the
   last query, so stepping through results does not mean retyping it.
   Committing an edit keeps the caret where the user left it (RFC-084).
10. **Words mean one thing** [COGA 4.4.1]. Owner decision 2026-11-26: the
    word **"zoom" is used for nothing**. Moving into and out of a section is
    "Open section" / "Up one level"; size is "Text size". The UI vocabulary
    is audited in both languages for terms only this application uses.
11. **Stability.** Controls stay where they are; additions go where users
    already look [NNG; COGA 4.2.2].

### 5.5 Plain file text

The plain file text view follows the text size, uses RFC-083's font and line
spacing, and **wraps by default**. Wrapping can be turned off for files whose
layout matters (wide JSON) — the need is real, so `line_wrap` becomes a
working setting rather than being deleted.

### 5.6 Layout at large sizes (M17, 0.20.0)

At 200% the two panes do not fit a 1080px window. Below a width measured in
`em`, the Document Map becomes a panel opened by a visible, labelled button —
never a hidden gesture — instead of squeezing the Writing Area. No horizontal
scrolling (WCAG 1.4.10).

## 6. Non-goals

- A free colour picker, or font sizes beyond 200%: unbounded choices could
  make the app unusable (§4.4).
- Bundled fonts.
- A "simple mode" that hides features [COGA 4.9.3] — not asked for by any
  finding.

## 7. Validation

- Contrast guard: every palette, at its tier.
- RFC-079's layout check: every view × size × locale × palette × window size,
  plus WCAG 1.4.12 spacing.
- `forced-colors`: the four Windows contrast themes — owner.
- OS text size on Windows and GNOME — measured, then covered by the check.
- Focus visible in every palette — layout check measures ring size and
  contrast.
- **Real users** [COGA §1]: RFC-061 gains older and low-vision participants;
  the owner is arranging them. Their findings become layout-check rules.

## 8. Acceptance criteria

- WCAG 2.2 AA in every combination; AAA contrast in the high-contrast
  palettes; AAA target size from about 137%.
- Every §5.4 rule holds, each with a test or a recorded check.
- No horizontal scrolling at 200% in plain file text with wrapping on.
- The word "zoom" appears nowhere in the UI or the user documentation.

## 9. Sources (read 2026-11-26)

- **[LV]** W3C, *Accessibility Requirements for People with Low Vision*
  (First Public Working Draft, 2016), https://www.w3.org/TR/low-vision-needs/
- **[WAI-AGE]** W3C WAI, *Older Users and Web Accessibility*;
  *Developing Websites for Older People*, https://www.w3.org/WAI/older-users/
- **[COGA]** W3C, *Making Content Usable for People with Cognitive and
  Learning Disabilities* (2021), https://www.w3.org/TR/coga-usable/
- **[MS]** Microsoft Learn, *Text scaling*; *Contrast themes*; *Accessible
  text requirements*
- **[NNG]** Nielsen Norman Group, *Usability for Older Adults* (2019);
  *Usability for Senior Citizens: Improved, But Still Lacking* (2013)
- **[GNOME]** GNOME Human Interface Guidelines: accessibility, UI styling
- WCAG 2.2, criteria as named

Limits: [LV] is a 2016 draft; [NNG]'s figures come from websites. Apple's
guidelines could not be read (JavaScript-only pages).
