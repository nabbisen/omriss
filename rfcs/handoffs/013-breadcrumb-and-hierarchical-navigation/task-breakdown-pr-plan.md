# RFC-013 — Task Breakdown and PR Plan (breadcrumb layout)

Related RFC: `../../done/013-breadcrumb-and-hierarchical-navigation.md`

Lifecycle status: inherited from RFC-013.

---

## PR 1 — wrapper, truncation, translated label

The three items of the implementation handoff §3. CHANGELOG `Fixed`.

Level 2: a Dioxus runtime test that the crumbs render inside
`.breadcrumb-crumbs` and the up button outside it, and that the "…" label is
the catalog's in both locales.

**Landed** (`c1fdf3f`).

## PR 2 — cutting order (implementation handoff §5)

CSS only. Level 2 cannot see widths; RFC-079's layout check will hold this
later. Now: the same fixture and GUI extents as PR 1, both locales — the root
at its natural width, the ancestors uncut while the current crumb has room to
give, and one capture of the Japanese case.
