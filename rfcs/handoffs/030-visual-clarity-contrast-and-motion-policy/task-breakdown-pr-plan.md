# RFC-030 — Task Breakdown and PR Plan (contrast defects)

Related RFC: `../../done/030-visual-clarity-contrast-and-motion-policy.md`

Lifecycle status: inherited from RFC-030.

---

## PR 1 — tokens and guard

The two tokens, the two light-theme values, `.skip-link` removed, the guard
test. CHANGELOG `Fixed`.

Show that the guard bites: put `color: #0b1320` back on `button.primary`, and
separately restore the old light `--muted`; each fails the test. Restore.
