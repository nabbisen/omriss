# RFC-087: Hide the Document Map

**Project:** omriss — Omriss Editor
**Milestone:** M16 — Readable and operable (0.19.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, UI/UX designer, Rust developer, QA engineer
**Depends on:** RFC-049, RFC-059, RFC-079, RFC-083
**Related RFCs:** RFC-060, RFC-080, RFC-081

---

## 1. Summary

A writer can hide the Document Map to give the Writing Area the window, and
show it again with one visible button. Owner's idea, 2026-11-28: "enable the
author to focus on writing".

**Scope, owner's decision of the same day: the map only.** The toolbar and
the status bar stay. Save, Undo and the unsaved-changes notice stay where
they are, so hiding the map can never hide the state of the user's work.

## 2. Design

1. **One state: the map is shown or hidden.** It is shell view state, the
   same in every view and every format, and it is not part of the document.
   It is remembered between sessions through RFC-083's settings, which also
   land in 0.19.0. Shown is the default.
2. **One visible control, always in the same place.** A toolbar button that
   says what it will do: "Hide Document Map", or "Show Document Map" while it
   is hidden. It is a word, not an icon alone (RFC-080 §5.4 rule 2), and it
   is never hidden itself, so the way back is always on screen (rule 3). The
   button does not move when its label changes (rule 11).
3. **The same command elsewhere.** It is in Quick Actions, and the keyboard
   reference lists it. Its shortcut comes from RFC-081's key study, not
   chosen here. RFC-081's "Go to Document Map" shows a hidden map and then
   focuses its active node.
4. **Hidden means gone for everyone.** The map is removed from the page,
   not only made invisible, so a screen reader does not find a landmark that
   is not there. The button carries the state in its name. RFC-060's landmark
   test covers both states.
5. **Focus.** Hiding the map while focus is inside it moves focus to the
   Writing Area's primary control, through `focus.rs`'s existing fallback
   (RFC-059 §12). Showing the map does not take focus away from the writer.
6. **The text stays readable when it gains the width.** The Writing Area
   column gets a maximum width in `em` (RFC-079), so lines do not grow to
   the full window. The column is centred and its text is aligned to the
   start (RFC-080 §5.4 rule 4).
7. **Nothing else is lost.** The breadcrumb and Up, search, Subsections and
   Quick Actions all move through the document without the map.
8. **One mechanism for narrow windows later.** RFC-080 §5.6 (0.20.0) makes
   the map a panel below a width measured in `em`. It reuses this state and
   this button instead of adding a second way to show the map.

## 3. Non-goals

- **Hiding the toolbar or the status bar** (a "zen mode"). The owner kept
  them, because they show the state of the user's work.
- Resizing the map's width by dragging.
- Hiding per document.

## 4. Validation

- Level 2: the toggle hides and shows the map in every view; the button's
  label in both locales; focus moves out of a hidden map; RFC-060's landmark
  test in both states.
- RFC-079's layout check, with the map hidden: every view × text size ×
  locale. The Writing Area column respects its maximum width.
- Settings: the choice survives a restart (RFC-083).
- GUI: hide, write, search, open a section from the breadcrumb, and show the
  map again with the button, Quick Actions and the shortcut.

## 5. Acceptance criteria

1. One visible, labelled button hides and shows the map, and it is always
   on screen.
2. Save, Undo and the unsaved-changes notice are unaffected.
3. A hidden map is absent from the accessible tree.
4. Focus is never left on nothing when the map hides.
5. The choice is remembered.
