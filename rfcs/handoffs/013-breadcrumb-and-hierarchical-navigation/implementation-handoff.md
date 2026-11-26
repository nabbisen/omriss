# RFC-013 — Implementation Handoff (breadcrumb layout)

Related RFC: `../../done/013-breadcrumb-and-hierarchical-navigation.md`

Lifecycle status: inherited from RFC-013.

---

## 1. Why the path is spread out

`style.css:338` gives `.breadcrumb` `justify-content: space-between`, meant to
push the "up" button to the right edge, and defines `.breadcrumb-crumbs`
(`flex: 1`, line 347) to hold the path on the left. **`breadcrumb.rs` never
renders that wrapper.** Every crumb, separator and the up button are direct
children of the `nav`, so `space-between` spreads all of them across the width.

## 2. What already handles long paths, and what does not

- **Many levels:** more than four segments collapse to
  `first › … › parent › current` (`MAX_VISIBLE`, `breadcrumb.rs:8`). RFC-013's
  "long path collapses without hiding current section" holds.
- **Long titles:** nothing. The `nav` wraps (`flex-wrap: wrap`), so one long
  heading pushes the path onto two or three lines and moves the editor down.
- **The "…" is in English for screen readers:** `aria-label: "collapsed
  sections"` is a literal, not a catalog key.

## 3. The fix (decided)

1. **Render the wrapper.** The crumbs go inside `div.breadcrumb-crumbs`; the
   up button stays outside it, at the right.
2. **One line, truncated.** The crumbs do not wrap. Each ancestor crumb has a
   maximum width (`12em`) and ends in an ellipsis when cut; the current
   section takes the remaining width and is cut last (`min-width: 0`,
   ellipsis). Every crumb whose text can be cut carries its full title as
   `title`, so hovering shows it; the accessible name stays the full text
   (CSS truncation does not change it).
3. **Translate the "…" label:** a catalog key, English and Japanese.

Not changed: the four-segment collapse, the "…" staying non-interactive (the
up button and the Document Map reach every ancestor), and what clicking a
crumb does.

## 4. Constraints

- `omriss-core` unchanged; `omriss-ui` only for the catalog key.
- No new dependency.
