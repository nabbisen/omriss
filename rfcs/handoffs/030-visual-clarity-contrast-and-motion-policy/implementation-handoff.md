# RFC-030 — Implementation Handoff (contrast defects)

Related RFC: `../../done/030-visual-clarity-contrast-and-motion-policy.md`

Lifecycle status: inherited from RFC-030.

---

## 1. What fails

WCAG 2 contrast ratio, computed from the tokens in `style.css` (dark `:root`,
light `@media (prefers-color-scheme: light)`). Target: 4.5:1.

| Pair | Theme | Ratio |
|---|---|---|
| `button.primary`: `#0b1320` on `--accent` | light | **3.71** |
| `.btn-danger` (Delete): `#fff` on `--danger` | dark | **2.52** |
| `--muted` on `--panel` | light | **4.09** |
| `--muted` on `--bg` | light | **4.44** |
| `--accent` text on `--panel` | light | **4.27** |

Every other token pair passes (dark `--muted` on `--panel` is the lowest, 4.69).

**The cause of the first two:** the text colour on a coloured button is a
literal (`#0b1320`, `#fff`) while the background is a token that changes with
the theme. A literal right for one theme is wrong for the other. There are
exactly four such literals in `style.css` (lines 45, 46, 578, 632).

## 2. The fix (decided)

1. **Two new tokens, per theme**: `--on-accent` and `--on-danger`, the text
   colour on an accent or danger background.

   | Token | Dark | Light |
   |---|---|---|
   | `--on-accent` | `#0b1320` | `#ffffff` |
   | `--on-danger` | `#0b1320` | `#ffffff` |

   `button.primary` and `.btn-danger` use them. No other text colour literal
   remains.
2. **Light theme, two tokens darkened:** `--muted` `#6b737d` → `#5f6670`;
   `--accent` `#1a6fcc` → `#1866bd`, and `--accent-soft` to the same hue
   (`rgba(24, 102, 189, 0.12)`). Resulting lowest pairs: muted on panel 4.94,
   accent on panel 4.87, white on accent 5.72.
3. **Delete `.skip-link`'s CSS.** Nothing renders that class; it is a fifth
   literal (`#fff` on accent, 2.63 in dark) waiting to be used.

Dark theme's `--muted`, `--accent` and `--danger` do not change.

## 3. The guard

A test that reads `style.css` and fails when:

- any `color:` declaration is not a `var(--…)` (so the literal-on-token bug
  cannot come back);
- any pair in a declared list falls below 4.5:1 in either theme. The list:
  `--text`, `--muted`, `--accent`, `--danger` each on `--bg` and `--panel`;
  `--on-accent` on `--accent`; `--on-danger` on `--danger`. Hover states that
  use `filter: brightness(1.1)` count too: compute them with each channel
  multiplied by 1.1, clamped.

It runs in plain `cargo test`. Put it where that holds — the stylesheet
lives in `crates/app`, which is not a default member.

This is level 1 of the GUI rules' Test Optimization Rules: contrast is a
property of the token values, checkable without a renderer. It is narrower
than the level-2 layout matrix those rules describe (that one is still
undecided); it covers this bug class completely.

## 4. Constraints

- `omriss-core`, `omriss-ui` unchanged. No new dependency — the ratio is a
  ten-line function.
- No layout or size change. Colours only.
