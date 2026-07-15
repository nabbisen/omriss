# RFC-049: Document Map Structural Editing Model

**Project:** omriss — Omriss Editor
**Milestone:** M10 — UI Role Separation
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048
**Related RFCs:** RFC-050, RFC-051, RFC-052, RFC-053

---

## 1. Summary

This RFC defines the **Document Map** as the single user interface location for structure navigation and organization in omriss.

For Markdown, the Document Map manages sections derived from headings.

For future structured plain-text formats, the Document Map may manage groups, lists, names, and values derived from JSON, TOML, or YAML adapters.

The Document Map must make structure visible without forcing users to understand file syntax.

## 2. Design principle

> The Document Map is where users shape the document.

The user should not need to inspect raw syntax, heading markers, braces, brackets, indentation, or table notation to understand the structure of the file.

## 3. Goals

- Provide a clear, always-visible map of the current document.
- Make current selection obvious.
- Move all visible structure controls out of the right-side editing area.
- Use plain language for labels and warnings.
- Support Markdown sections first.
- Keep the component model compatible with future structure adapters.
- Prevent accidental destructive changes.
- Keep operations source-preserving through core document commands.
- Support mouse, keyboard, and screen-reader users.

## 4. Non-goals

- This RFC does not implement JSON/TOML/YAML adapters.
- This RFC does not require drag-and-drop in the first implementation.
- This RFC does not expose raw node IDs, byte ranges, parser terms, or heading levels to normal users.
- This RFC does not make the left panel a full schema editor.
- This RFC does not add collaborative outline editing.

## 5. Document Map layout

```text
┌──────────────────────────────┐
│ Document Map                 │
│                              │
│ [+ Add]                      │
│                              │
│ ▾ Introduction          [⋯] │
│   ▸ Background          [⋯] │
│   ▸ Goal                [⋯] │
│ ▸ Methods               [⋯] │
│ ▸ Results               [⋯] │
│                              │
│ Tip: Use this map to arrange │
│ your document.               │
└──────────────────────────────┘
```

For non-Markdown formats, the same panel shape is reused:

```text
┌──────────────────────────────┐
│ Document Map                 │
│                              │
│ [+ Add]                      │
│                              │
│ ▾ package               [⋯] │
│   name                 [⋯] │
│   version              [⋯] │
│ ▾ dependencies          [⋯] │
│   serde                [⋯] │
│   pulldown-cmark       [⋯] │
└──────────────────────────────┘
```

## 6. Node model required by the UI

The Document Map should consume a format-neutral view model. It must not directly depend on Markdown-only section fields.

```rust
pub struct MapNodeView {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub depth: usize,
    pub title: String,
    pub subtitle: Option<String>,
    pub kind: MapNodeKind,
    pub is_current: bool,
    pub has_children: bool,
    pub can_expand: bool,
    pub available_actions: Vec<MapAction>,
}

pub enum MapNodeKind {
    DocumentRoot,
    MarkdownSection,
    Group,
    List,
    Value,
    RawRegion,
    Unsupported,
}

pub enum MapAction {
    AddInside,
    AddAfter,
    Rename,
    Move(MoveDirection), // Up | Down | InsidePrevious | OutOneLevel (RFC-053)
    JoinWithPrevious,
    Delete,
    ShowPlainFileText,
}
```

The exact Rust shape can differ, but the UI must receive enough information to render:

- title;
- depth;
- current selection;
- available actions;
- expansion state;
- plain-language action labels.

Movement uses the single `MoveDirection` vocabulary from RFC-053 (no separate
promote/demote actions). `available_actions` is derived from each node's
`NodeCapabilities` (RFC-053): the row menu shows only what the active adapter
allows, and a `Capability::Disabled { reason }` is rendered with a plain
explanation via the i18n catalog (RFC-043).

## 7. Plain-language action labels

### 7.1 Universal labels

| Internal operation | Default label |
|--------------------|---------------|
| select node | Open |
| add inside | Add inside |
| add after | Add after |
| rename | Rename |
| move up | Move up |
| move down | Move down |
| delete | Delete |
| show source | Show plain file text |

### 7.2 Markdown-specific labels

| Internal operation | User-facing label |
|--------------------|-------------------|
| promote heading | Move out one level |
| demote heading | Move inside previous section |
| merge section up | Merge into previous section content |

The UI must not expose “promote”, “demote”, “H1”, “H2”, or “heading level” as normal labels unless advanced details are explicitly shown.

### 7.3 Structured-data labels

| Internal term | User-facing label |
|---------------|-------------------|
| object/table | group |
| array | list |
| key | name |
| value | content |
| boolean | on/off |
| null | No value |

## 8. Visible controls

### 8.1 Always visible

The Document Map always shows:

- panel title: `Document Map`;
- current document tree;
- highlighted current item;
- one primary add button when adding is supported;
- row menus for available operations;
- optional plain tip for first-time users.

### 8.2 Hidden until needed

The following actions should be behind a row menu or equivalent disclosure:

- rename;
- move up/down;
- move inside/out;
- join/merge;
- delete;
- advanced format-specific actions.

### 8.3 Disabled vs hidden

Prefer hiding unsupported actions over showing disabled actions. If a disabled action is shown, the UI must explain why in plain language.

Example:

```text
Move up
Unavailable because this is already the first item.
```

## 9. Required Markdown behavior

For Markdown documents, the Document Map must support:

- selecting a section;
- adding a section inside the current section;
- adding a section after the current section;
- renaming a section title;
- moving a section up/down among siblings;
- moving a section out one level, when valid;
- moving a section inside the previous section, when valid;
- joining with the section above, when valid;
- deleting a section with confirmation.

All operations must preserve unrelated source text bytes.

## 10. Future structured-data behavior

For JSON/TOML/YAML, actions are provided by the active document format adapter. The Document Map must not invent actions that the adapter does not support.

Example support policy:

| Format | Add | Rename | Move | Delete | Edit value |
|--------|-----|--------|------|--------|------------|
| Markdown | Yes | Yes | Yes | Yes | In Writing Area |
| JSON | Later staged | Object keys only | Array items maybe | Yes with validation | RFC-054 |
| TOML | Later staged | Table/key names only | Conservative | Conservative | RFC-055 |
| YAML | Feasibility only | Not promised | Not promised | Not promised | RFC-056 |

## 11. Confirmation dialogs

Destructive actions must open a confirmation dialog.

### 11.1 Delete Markdown section

```text
Delete this section?

“Background” and its 2 smaller sections will be removed.

[Cancel] [Delete]
```

### 11.2 Delete structured value/group

```text
Delete this item?

“theme” will be removed from this file.

[Cancel] [Delete]
```

If the item contains children:

```text
Delete this group?

“settings” and its 5 items will be removed.

[Cancel] [Delete]
```

## 12. Keyboard and focus behavior

- Arrow keys move through visible map rows.
- Enter opens the selected row in the right panel.
- Space toggles expansion when the row can expand.
- Context menu key or a visible `⋯` button opens the row menu.
- Esc closes row menus before leaving the panel.
- After deleting a row, focus moves to the nearest safe remaining row.
- After adding a row, focus moves to the newly created row and the right panel opens it.

## 13. Accessibility requirements

The Document Map must provide:

- accessible panel label;
- accessible tree/list structure;
- current item indication;
- expansion state;
- action menu labels;
- dialog labels and descriptions;
- live feedback after successful and failed operations.

Screen-reader phrasing should be plain:

```text
Background, selected, section, contains 2 smaller sections.
```

For structured data:

```text
settings, selected, group, contains 5 items.
```

## 14. Error handling

Errors must be reported without technical internals.

| Cause | Message |
|-------|---------|
| stale selected node | “That item changed. Please choose it again.” |
| invalid move | “This item cannot be moved there.” |
| unsupported action | “This file type does not support that change yet.” |
| parser rejected result | “This change would make the file invalid.” |
| source-preservation failure | “This change could not be made safely.” |

## 15. Internal design notes

The Document Map should send high-level commands rather than edit source text directly.

```rust
pub enum DocumentMapCommand {
    Select(NodeId),
    AddInside(NodeId, NewItemSpec),
    AddAfter(NodeId, NewItemSpec),
    Rename(NodeId, String),
    Move(NodeId, MoveDirection),
    JoinWithPrevious(NodeId),
    Delete(NodeId),
}
```

The application session routes commands to the active document format adapter or Markdown core operation.

`MapAction` and `DocumentMapCommand` are presentation/session wrappers only.
They are not alternative canonical mutation commands; each maps to the
`StructureCommand` defined in RFC-053:

| Document Map event | Canonical mutation command (RFC-053) |
|---|---|
| `MapAction::AddInside` | `StructureCommand::AddInside` |
| `MapAction::AddAfter` | `StructureCommand::AddAfter` |
| `MapAction::Rename` | `StructureCommand::Rename` |
| `MapAction::Move(direction)` | `StructureCommand::Move { direction }` |
| `MapAction::JoinWithPrevious` | `StructureCommand::JoinWithPrevious` |
| `MapAction::Delete` | `StructureCommand::Delete` |
| `MapAction::ShowPlainFileText` | view/session action, not a structure mutation |

The Document Map must not know how to rewrite Markdown headings, JSON values, TOML tables, or YAML indentation.

## 16. Component reconciliation (Document Map vs ItemTreeView)

The Document Map is a **new product component**, not a renamed outline panel.
The shipped outline sidebar uses `dioxus-swdir-tree::ItemTreeView`. The Document
Map may reuse that component's keyboard navigation, expand/collapse handling,
and ARIA tree semantics, but it must not inherit file-tree assumptions
(directory/file kinds, string-only ids, no per-row action menu, no per-action
capability state).

Crate boundaries (RFC-001):

- `omriss-core` owns the canonical structure and capability types from
  RFC-053: `StructureCommand`, `MoveDirection`, `NodeCapabilities`,
  `Capability`, `CapabilityReason`, `DocumentFormat`, and `NodeId`. The UI
  consumes these; it does not own or redefine them.
- `omriss-ui` owns the Dioxus-free Document Map view model (`MapNodeView`),
  selection state, expanded/collapsed state, row-menu state, and the mapping
  from `CapabilityReason` to i18n keys (RFC-043) — no Dioxus.
- the `omriss` app package owns the rendered Dioxus component, the `⋯` row menus, the
  confirmation dialogs, and any `dioxus-swdir-tree` integration.

Whether to extend `ItemTreeView` (a first-party crate) with action-slot and
capability support, or to build a dedicated component that lifts only the
reusable Dioxus-free logic into `omriss-ui`, is settled in this RFC's first PR;
the requirement is only that the Document Map is a new component with its own
capability-gated row menus.

## 17. Acceptance criteria

- Structure actions are no longer visible in the Writing Area.
- Document Map supports Markdown selection, add, rename, move, join, and delete flows.
- Unsupported actions are hidden or explained.
- Deletion always requires confirmation.
- Current item remains visually clear after every operation.
- Keyboard users can perform the same operations as pointer users.
- Screen-reader labels do not expose internal IDs or parser details.
- Component boundary can accept future `MapNodeKind` values without redesign.

## 18. Open questions

1. Should drag-and-drop be added as a later enhancement after menu-based movement is stable?
2. Should the Document Map show type badges for structured data files, or would that add too much visual noise?
3. Should “Add” default to “Add after” or open a small chooser between “inside” and “after”?

## 19. Final decision summary

The Document Map becomes the only visible structure organization surface. It starts with Markdown sections, but its UI model is explicitly format-neutral so future JSON/TOML/YAML structure adapters can reuse the same left-panel interaction model.
