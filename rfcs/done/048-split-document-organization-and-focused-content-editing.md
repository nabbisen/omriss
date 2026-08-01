# RFC-048: Split Document Organization and Focused Content Editing

**Project:** omriss — Omriss Editor
**Milestone:** M10 — UI Role Separation
**Status.** Implemented (v0.16.0) — deferred: keyboard/focus follow-ups to
RFC-059; placement-complete creation controls to RFC-058; full
accessibility/screen-reader validation to RFC-060; non-technical-user
walkthrough validation to RFC-061. §14 open questions 2 and 3 remain open and
are inherited by RFC-053/RFC-054.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** Source-preserving Markdown core, outline tree, focus editing, undo/redo (RFC-001–046); RFC-053 type core
**M10 bundle (reviewed together):** RFC-049, RFC-050, RFC-051
**Related RFCs:** RFC-052

---

## 1. Summary

This RFC changes the primary omriss interaction model from a mixed-role main content area to a clear two-zone layout:

- The **left panel**, named **Document Map**, is where users navigate and organize document structure.
- The **right panel**, named **Writing Area** for Markdown and **Focused Content Area** internally, is where users edit, preview, and save the currently selected content. For future structured formats, the panel title should be the selected item rather than a generic panel name.

The first implementation target remains Markdown. For Markdown files, the Document Map is derived from headings and the Writing Area edits the selected section body.

This RFC is the directional product decision for the M10 UI split. It is not
implementation-authorizing on its own. RFC-048 must be accepted as part of the
M10 bundle with RFC-049, RFC-050, and RFC-051, and that bundle uses the RFC-053
type core as the canonical type source.

This RFC is deliberately written in a format-neutral way so that the same layout can later support JSON, TOML, and possibly YAML through format adapters.

> The Document Map organizes structure. The Writing Area edits the focused content.

## 2. Motivation

The previous Focus Mode allowed the main content area to serve two roles:

1. editing section text;
2. customizing document structure by managing child sections and structural controls.

This mixed-role layout is powerful for expert users, but it increases cognitive load for non-technical users. A user looking at the main canvas may not know whether they are writing, reorganizing, or performing a destructive operation.

The new design makes one role visible in one place:

```text
┌──────────────────────────────┬──────────────────────────────────────┐
│ Document Map                 │ Writing Area                         │
│                              │                                      │
│ Organize structure           │ Edit focused content                 │
│ Select current item          │ Preview focused content              │
│ Add / rename / move / delete │ Save the file                        │
└──────────────────────────────┴──────────────────────────────────────┘
```

For Markdown this reads naturally as:

```text
Left side  = organize sections
Right side = write the selected section
```

For future structured data files it becomes:

```text
Left side  = organize keys, groups, lists, or values
Right side = edit the selected value or group text
```

## 3. Goals

- Make document structure visible and editable in one dedicated place.
- Make the main editing surface calm and focused.
- Remove structural editing controls from the main content canvas.
- Reduce accidental deletion, movement, merging, or heading-level changes.
- Preserve the existing source-preserving document architecture.
- Keep Markdown as the primary and first-class format.
- Avoid Markdown-only UI assumptions that would block future structured plain-text formats.
- Maintain keyboard accessibility and screen-reader clarity.
- Keep advanced structure operations available but hidden until needed.

## 4. Non-goals

- This RFC does not implement JSON, TOML, or YAML support.
- This RFC does not define the document format adapter trait; see RFC-053.
- This RFC does not introduce collaborative editing.
- This RFC does not add rich-text block editing.
- This RFC does not change the canonical document rule: the source text remains canonical.
- This RFC does not require drag-and-drop in the first implementation.
- This RFC does not make the raw source view the primary editing surface.

## 5. Terminology

| Term | Meaning |
|------|---------|
| Source text | The original plain-text file content held as the canonical document. |
| Document Map | The left-side navigation and structure organization panel. |
| Focused node | The selected structural item in the current document. For Markdown this is usually a section. |
| Writing Area | Markdown user-facing label for the right-side section editing area. |
| Focused Content Area | Internal, format-neutral architecture name for the right-side area. Structured formats title the selected item instead of naming the panel. |
| Structure operation | An operation that changes document hierarchy, ordering, names, or deletion. |
| Content operation | An operation that changes the text/value of the focused item without changing the surrounding structure. |

## 6. Decision

omriss shall adopt a permanent two-zone interaction model:

```text
┌────────────────────────────────────────────────────────────────────┐
│ omriss                                      [Open] [Save] [Preview] │
├───────────────────────┬────────────────────────────────────────────┤
│ Document Map          │ Writing Area                               │
│                       │                                            │
│ [+ Add]               │ Selected item title                        │
│                       │                                            │
│ ▾ Introduction        │ ┌────────────────────────────────────────┐ │
│   ▸ Background        │ │ Focused content editor                 │ │
│   ▸ Goal              │ │                                        │ │
│ ▸ Methods             │ └────────────────────────────────────────┘ │
│ ▸ Results             │                                            │
│                       │ [Preview]                      Saved ✓     │
├───────────────────────┴────────────────────────────────────────────┤
│ Status                                                             │
└────────────────────────────────────────────────────────────────────┘
```

The Document Map owns visible structure-changing controls.

The Writing Area owns visible focused-content controls. Focused drafts apply
automatically on navigation, save, preview, search, or blur; there is no primary
Done control in the M10 design.

Internal architecture name: Focused Content Area.

## 7. Format-neutral rule

The layout must be implemented against format-neutral UI concepts. RFC-053 is
the canonical authority for document format, structure node, capability,
command, movement, draft-state, and focused-content boundary types. RFC-049
defines the Document Map view model derived from those canonical types.

The first implementation may only instantiate Markdown section content, but UI
component names and data flow should not assume that all future nodes are
Markdown headings.

Examples:

| Bad assumption | Preferred neutral wording |
|----------------|---------------------------|
| `HeadingTreePanel` | `DocumentMapPanel` |
| `SectionEditor` as global right panel name | `FocusedContentPanel` internally, `WritingArea` in Markdown UI |
| “Edit section body” hard-coded everywhere | “Edit focused content” internally, “Write this section” in Markdown UI |
| `children` only as Markdown subsections | `child_nodes` in core/UI adapter boundary |

## 8. User-facing wording

The normal user should see plain language.

| Concept | User-facing label |
|---------|-------------------|
| Document structure | Document Map |
| Markdown section body | Section text |
| Current node | Current section / selected item, depending on format |
| Structural editing | Organize |
| Raw source | Button/action label: Show plain file text |
| Focused draft application | Automatic on navigation and save |
| Command palette | Quick Actions |

For Markdown documents, use “section” because it is natural for writers.

For structured data files, avoid jargon where possible:

| Technical term | Plain label |
|----------------|-------------|
| object / table | group |
| array | list |
| key | name |
| value | content |
| boolean | on/off |
| null | No value |

## 9. Behavior requirements

### 9.1 Selection

Selecting an item in the Document Map must update the right panel to show that focused item.

For Markdown:

```text
Select “Background” in Document Map
→ Writing Area shows Background title and section body editor.
```

For future JSON/TOML/YAML:

```text
Select “settings.theme” in Document Map
→ Focused Content Area shows an editor for the selected value.
```

### 9.2 Structure changes

Visible structure changes must originate from the Document Map:

- add;
- rename;
- move;
- delete;
- merge / join where supported;
- move into / move out where supported.

The right panel must not show visible structure-changing controls in the default writing/editing surface.

### 9.3 Content changes

Focused content changes must happen in the right panel:

- Markdown section body editing;
- previewing selected content;
- editing a structured value when future adapters support it;
- validating focused content before applying changes;
- saving the document.

Focused draft application follows RFC-050 §9 and RFC-053 §9.3:

- Markdown section body drafts are valid and apply on navigation, save, preview,
  search, or blur.
- Future structured drafts may block navigation, preview, or save while invalid.
- The UI must not expose a primary Done action for this lifecycle.

### 9.4 Raw source escape hatch

The application must retain a source-view escape hatch. User-facing label:

```text
Show plain file text
```

The raw source view must be reachable without implying that users need to understand Markdown, JSON, TOML, or YAML syntax.

## 10. Accessibility requirements

- The Document Map must be an accessible navigation region.
- The right panel must be an accessible main editing region.
- Focus must move predictably after selection, add, rename, delete, and save operations.
- Structural menus must be keyboard-reachable.
- Dangerous operations must use confirmation dialogs with `Cancel` as the safest default.
- Status messages must use live regions and plain language.

Expected landmarks:

```text
<header> app toolbar
<aside aria-label="Document Map"> structure navigation
<main aria-label="Writing Area"> focused content editing
<footer> status and save feedback
```

`aria-label="Writing Area"` is the Markdown label. Future structured-data views
should use the selected item title or another plain accessible name rather than
forcing the Markdown panel name onto non-Markdown content.

The implementation may vary exact tags based on Dioxus rendering constraints, but the accessibility tree must preserve these roles.

## 11. Error and safety requirements

No visible message should expose parser internals, byte ranges, node IDs, or system errors.

Examples:

| Internal cause | User-facing message |
|----------------|---------------------|
| stale node ID | “That item changed. Please choose it again.” |
| invalid edit range | “This part could not be changed safely.” |
| invalid JSON after edit | “This text does not look valid yet.” |
| save I/O failure | “Could not save the file. Try Save As.” |

## 12. Implementation notes

The UI should be split into at least these conceptual components:

```text
AppShell
  Toolbar
  DocumentMapPanel
  FocusedContentPanel
  StatusBar
  DialogHost
```

`FocusedContentPanel` should delegate to format-specific content views:

```text
FocusedContentPanel
  MarkdownSectionContentView
  JsonNodeContentView      future RFC-054
  TomlNodeContentView      future RFC-055
  YamlNodeContentView      future RFC-056 only if accepted
```

In the first migration, only `MarkdownSectionContentView` is required.

Canonical cross-RFC ownership:

- RFC-049 owns Document Map actions, row menus, and structure-control placement.
- RFC-050 owns Writing Area behavior, labels, and apply-on-navigation draft
  lifecycle.
- RFC-051 owns migration sequencing, rollback/flag decision, tests, and
  release criteria.
- RFC-053 owns canonical format, node-kind, capability, command, movement,
  draft-state, and focused-content boundary names.

## 13. Acceptance criteria

- Main area no longer shows section move/delete/promote/demote controls.
- Document Map is the only visible place for structural operations.
- Writing Area only shows focused content editing, preview, save/status, and
  related focused-content controls; RFC-050 §7 is the exact allowed/forbidden
  control list.
- Selecting a section in the Document Map updates the Writing Area.
- Current focused item is clearly highlighted in the Document Map.
- Existing Markdown source-preservation tests still pass.
- UI component names and boundaries do not prevent future JSON/TOML/YAML nodes.
- User-facing labels avoid unnecessary technical terms.
- Keyboard navigation works across toolbar, Document Map, Writing Area, dialogs, and status recovery actions.

These are high-level product acceptance goals. RFC-051 §8 and §12 are the
binding migration test plan and acceptance checklist for M10. The Markdown
source-preservation regression anchor is mandatory and blocking.

## 14. Open questions

1. **Resolved in RFC-050 §18.** Markdown user-facing label: “Writing Area”.
   Internal architecture name: “Focused Content Area”. Structured-data files
   should title the selected item instead of naming the panel.
2. Should “Show plain file text” be read-only initially for structured formats?
3. Should future non-Markdown support be hidden behind an experimental setting until stable?

## 15. Final decision summary

The UI split is the M10 directional foundation:

```text
Document Map = structure selection and organization.
Writing Area / Focused Content Area = focused content editing, preview, and save/status.
```

Markdown remains the primary product experience, but the layout must not
hard-code Markdown assumptions that would block structured plain-text support
later. Implementation and developer handoff require the accepted M10 bundle
(RFC-048 through RFC-051) plus the RFC-053 type core.
