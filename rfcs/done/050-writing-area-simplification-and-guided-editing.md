# RFC-050: Writing Area Simplification and Guided Editing

**Project:** omriss — Omriss Editor
**Milestone:** M10 — UI Role Separation
**Status.** Implemented (v0.16.0) — deferred: §17 "component boundary supports
future structured value editors" is unproven until RFC-053 introduces the
canonical `FocusedContent` / editor-kind types; only the Markdown body editor
exists today. Pending-draft undo/redo clarity is deferred to RFC-059
(`RFC048-QA-020`, `RFC048-QA-021`); screen-reader validation to RFC-060.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048
**Related RFCs:** RFC-049, RFC-051, RFC-053

---

## 1. Summary

This RFC redesigns the right-side main panel so that it has one clear purpose: editing, previewing, and saving the focused content selected from the Document Map.

For Markdown documents, this panel is user-facing as the **Writing Area** and edits the selected section body.

For future structured plain-text documents, the same panel may render a focused value, list, group, or raw text region editor through format-specific content views.

## 2. Design principle

> The right side is where users work on the selected item, not where they rearrange the document.

The interface should feel calm. When users look at the right side, they should understand that this is the safe place to write or edit the selected content.

## 3. Goals

- Reduce clutter in the main content area.
- Remove visible structure-editing controls from the main content area.
- Keep editing, preview, save, undo, redo, and status feedback easy to find.
- Provide guided empty states for new sections or empty values.
- Support Markdown section writing first.
- Keep the panel adaptable for JSON/TOML focused-value editing later.
- Avoid user-facing technical jargon.
- Keep keyboard and screen-reader behavior predictable.

## 4. Non-goals

- This RFC does not introduce rich text editing.
- This RFC does not make preview the default mode.
- This RFC does not add AI writing or rewriting features.
- This RFC does not implement JSON/TOML/YAML editing directly.
- This RFC does not change source-preserving replacement logic.
- This RFC does not remove advanced commands from Quick Actions.

## 5. Markdown Writing Area layout

```text
┌──────────────────────────────────────────────────────────────┐
│ Document › Introduction › Background                         │
├──────────────────────────────────────────────────────────────┤
│ Background                                                    │
│                                                              │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ Write this section here…                                 │ │
│ │                                                          │ │
│ │                                                          │ │
│ └──────────────────────────────────────────────────────────┘ │
│                                                              │
│ [Preview]                                         Saved ✓    │
│                                                              │
│ Smaller sections                                             │
│ [Prior Work] [Key Concepts]                                  │
└──────────────────────────────────────────────────────────────┘
```

The `Smaller sections` area is optional. If included, it must be read-only navigation only. Adding, deleting, renaming, moving, or joining smaller sections must be done from the Document Map.

## 6. Future structured content layout

For structured plain-text files, the panel should keep the same calm layout but use value-appropriate controls.

### 6.1 Selected text value

```text
┌──────────────────────────────────────────────────────────────┐
│ settings › title                                             │
├──────────────────────────────────────────────────────────────┤
│ title                                                        │
│                                                              │
│ Text                                                         │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ My Project                                                │ │
│ └──────────────────────────────────────────────────────────┘ │
│                                                              │
│                                                   Saved ✓    │
└──────────────────────────────────────────────────────────────┘
```

### 6.2 Selected on/off value

```text
┌──────────────────────────────────────────────────────────────┐
│ settings › autosave                                          │
├──────────────────────────────────────────────────────────────┤
│ autosave                                                     │
│                                                              │
│ [ On ] [ Off ]                                               │
│                                                              │
│                                                   Saved ✓    │
└──────────────────────────────────────────────────────────────┘
```

### 6.3 Selected group or list

```text
┌──────────────────────────────────────────────────────────────┐
│ settings                                                     │
├──────────────────────────────────────────────────────────────┤
│ settings                                                     │
│                                                              │
│ This group contains 5 items.                                 │
│ Use the Document Map to add, rename, move, or delete items.  │
│                                                              │
│ [Show plain file text]                                       │
└──────────────────────────────────────────────────────────────┘
```

The exact structured-data UI is defined in RFC-054 and RFC-055. This RFC only reserves the right panel role.

## 7. Controls allowed in the Writing Area

The Writing Area may contain:

- breadcrumb path;
- selected item title;
- focused content editor;
- preview toggle, where useful;
- (focused drafts apply automatically on navigation/save; no separate Done button);
- Save / Save As at toolbar level;
- save status;
- undo / redo at toolbar or shortcut level;
- current focused content word/count information, optional;
- read-only navigation links to child nodes, optional;
- Show plain file text action.

The Writing Area must not contain visible controls for:

- add section/item;
- rename section/item;
- delete section/item;
- move up/down;
- move inside/out;
- join/merge;
- drag structure targets;
- structural toolbars.

## 8. User-facing labels

### 8.1 Markdown labels

| Previous / technical label | New label |
|----------------------------|-----------|
| Body editor | Section text |
| Raw Markdown | Show plain file text |
| Preview pane | Preview |
| Child sections | Smaller sections |
| Current focus | Current section |

### 8.2 Structured-data labels

| Technical label | Plain label |
|-----------------|-------------|
| string | Text |
| number | Number |
| boolean | On / Off |
| null | No value |
| object/table | Group |
| array | List |
| property/key | Name |

## 9. Draft lifecycle

omriss uses **apply-on-navigation** with a single user-visible dirty state. There is no primary `Done` action: a focused draft is applied automatically when the user navigates to another item, saves, previews, searches, or blurs the field. Draft validity is tracked by `DraftState` (RFC-053).

For Markdown, draft text remains local while the user types.

```text
User types
  → local draft updates
  → core source text remains unchanged

User changes selection / saves / previews / closes the file
  → validate if needed
  → apply focused replacement
  → rebuild structure index
  → update save state
```

For structured data, the same pattern applies, but validation is format-specific.

```text
User edits focused value
  → local draft updates
  → validation feedback appears gently

User navigates away / saves / previews
  → format adapter validates replacement
  → if valid: adapter applies source-preserving range edit; structure rebuilds
  → if invalid: navigation/save/preview is blocked, focus stays on the field,
    and plain guidance is shown (RFC-053 `DraftState::InvalidUncommitted`)
```

## 10. Empty states

Empty states must guide the user without showing implementation details.

### 10.1 Empty Markdown section

```text
This section is empty.

Start writing here, or use the Document Map to add smaller sections.
```

### 10.2 Empty structured group/list

```text
This group is empty.

Use the Document Map to add an item.
```

### 10.3 Unsupported focused edit

```text
This part can be viewed here, but editing it safely is not supported yet.

[Show plain file text]
```

This is especially important for early YAML support.

## 11. Validation feedback

Validation must be immediate but calm.

Good:

```text
This number is not valid yet.
```

Avoid:

```text
ParseError at byte offset 42.
```

Status and inline validation must not block typing. They do block leaving an invalid structured draft: navigation, preview, and save are held until the value is valid. Markdown body text is always valid and commits on navigation.

## 12. Save feedback

Saving should be visually clear and non-alarming.

States:

| State | Text |
|-------|------|
| no changes | Saved |
| local draft changed | Not saved yet |
| saving | Saving… |
| saved | Saved ✓ |
| save failed | Could not save. Try Save As. |

The right panel may show local draft state. The status bar may show file-level dirty state.

## 13. Preview behavior

For Markdown:

- Preview renders the selected section body.
- Preview must not hide the Save affordance.
- Preview must not introduce structure controls.

For structured data:

- Preview may show the selected item as formatted text.
- Formatting must be clearly labeled as a preview and must not imply the whole file will be reformatted.

## 14. Keyboard behavior

- Ctrl+S saves the file after applying the focused draft if valid.
- Esc returns focus to the Document Map only when no dialog/palette/search is open.
- Ctrl+F opens search.
- Ctrl+P opens Quick Actions.
- Tab moves through right-panel controls in visible order.
- Shift+Tab moves backward.
- Saving and returning to the Document Map must be reachable without leaving the keyboard.

## 15. Accessibility requirements

- The editor input must have a visible label or an accessible name.
- Preview mode must announce that the view changed.
- Validation messages must be associated with the input they describe.
- Save status must use a polite live region.
- Errors must use assertive live regions only when the user attempted an action and it failed.

## 16. Internal design notes

The right panel should render through a format-neutral wrapper.

```rust
pub enum FocusedContentViewModel {
    MarkdownSection(MarkdownSectionVm),
    StructuredValue(StructuredValueVm),
    StructuredGroup(StructuredGroupVm),
    Unsupported(UnsupportedNodeVm),
}
```

Initial implementation only requires `MarkdownSection`.

Future RFCs add:

- `StructuredValue` for JSON values;
- `StructuredGroup` for JSON objects/arrays and TOML tables;
- `Unsupported` for YAML nodes or complex syntax where safe editing is not ready.

## 17. Acceptance criteria

- Writing Area no longer contains visible structural editing controls.
- Markdown section body editing still works.
- Focused drafts apply automatically and safely on navigation and save; invalid structured drafts block until fixed.
- Ctrl+S applies pending draft and saves.
- Preview does not introduce structure controls.
- Empty sections show helpful guidance.
- Validation and save messages use plain language.
- Component boundary can support future structured value editors.
- Screen-reader and keyboard users can operate the panel without relying on pointer gestures.

## 18. Open questions

1. **Resolved (policy; finalize before RFC-054).** Markdown user-facing label: **Writing Area**. Generic architecture name: **Focused Content Area**. For structured-data files, avoid naming the panel; title the selected item instead. Does not block RFC-048–051.
2. **Resolved.** Focused drafts apply automatically on navigation (apply-on-navigation) with a single user-visible dirty state. A Markdown draft always commits; an invalid structured draft blocks navigation/preview/save until fixed (RFC-053 `DraftState`).
3. Should structured values use type-specific controls by default, or raw focused text with validation first?

## 19. Final decision summary

The right panel becomes a calm focused-content editing surface. For Markdown it remains a Writing Area. For future structured formats it can become a focused value/group editor, but it must never become the place where document structure is visibly rearranged.
