# RFC-058: Document Map Placement-Complete Creation Controls

**Project:** omriss — Omriss Editor
**Milestone:** M14 — Document Map Creation Placement (proposed)
**Status.** Proposed
**Document type:** Follow-up RFC design seed
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-049, RFC-053
**Related RFCs:** RFC-050, RFC-051

---

## 1. Summary

This RFC records the follow-up design direction for replacing the temporary
top-level creation shortcut in the Document Map with a placement-complete
creation model:

```text
+ Before   + Inside   + After
```

The user-facing model is:

- create a sibling before the selected item;
- create a child inside the selected item;
- create a sibling after the selected item.

Top-level creation is not a separate normal editing mode. With existing
top-level sections, the user selects a top-level section and creates a sibling
before or after it. Empty/no-section documents still need a first-section
affordance, such as `+ Section`.

## 2. Motivation

The RFC-048 / M10 Document Map QA flow exposed that creation controls must be
discoverable and should answer the user's real question: "where should the new
section go?"

The current accepted M10 implementation uses:

```text
+ Top   + Inside   + After
```

That implementation is acceptable for M10 stabilization, but `+ Top` is a
level-specific shortcut. It privileges top-level creation instead of expressing
the general placement relation around the selected item.

The long-term model should be placement-complete:

- `+ Before`
- `+ Inside`
- `+ After`

This is easier to generalize across Markdown sections and future structured
Document Map nodes.

## 3. Design Status

This RFC is a tracking/design seed, not an implementation authorization.

The architect design review in
`.git-exclude/reviewed/rfc-048-document-map-placement-complete-creation-review.md`
accepted the placement-complete model directionally, but deferred
implementation until RFC-049 and RFC-053 are amended.

M10 manual QA should continue on the accepted `+ Top` / `+ Inside` /
`+ After` strip unless the owner explicitly expands M10 scope.

## 4. Goals

- Replace permanent top-level-specific creation with placement-relative
  creation.
- Add `AddBefore` to the Document Map action vocabulary.
- Add an explicit `can_add_before` capability.
- Add a canonical `StructureCommand::AddBefore`.
- Keep creation actions out of row context menus.
- Preserve source text and undo behavior for Add Before.
- Define empty/no-section and no-selection behavior.
- Keep the model compatible with JSON/TOML/YAML adapters.

## 5. Non-goals

- This RFC does not require replacing the accepted M10 strip before M10 manual
  QA.
- This RFC does not add drag-and-drop.
- This RFC does not expose heading levels such as H1/H2 in normal labels.
- This RFC does not require every future format adapter to support Add Before.

## 6. User-Facing Model

### 6.1 Selected-section state

When a section or structural node is selected, the Document Map creation strip
should expose:

```text
+ Before   + Inside   + After
```

For Markdown:

- `+ Before` creates a sibling section before the selected section.
- `+ Inside` creates a child section at the end of the selected section.
- `+ After` creates a sibling section after the selected section's full
  subtree.

For future structured formats, adapters decide which placement actions are
supported for each node.

### 6.2 Empty/no-section state

When the document has no selectable section or structural node, the Document
Map should show a first-item affordance such as:

```text
+ Section
```

For Markdown, this creates the first top-level section.

### 6.3 Existing document with no selected row

The application should either:

- guarantee a selected row before showing placement controls; or
- show no placement controls and a plain prompt such as "Select a section to
  add before, inside, or after."

This state must be defined before implementation.

## 7. Required RFC Amendments

Before implementation, RFC-049 should be amended to include:

- `MapAction::AddBefore`;
- `DocumentMapCommand::AddBefore`;
- event-to-command mapping from Add Before to the canonical RFC-053 command;
- selected, empty, and no-selection control placement.

Before implementation, RFC-053 should be amended to include:

- `NodeCapabilities::can_add_before`;
- `StructureCommand::AddBefore { target: NodeId, spec: NewNodeSpec }`;
- adapter guidance for hiding/disabling Add Before independently of Add After.

RFC-048/RFC-051 handoff and QA material should be updated if this is pulled
into an active implementation milestone.

## 8. Source Preservation Requirements

Add Before must be implemented through the same source-preserving architecture
as Add Inside and Add After.

Required behavior:

- unrelated bytes remain untouched;
- selected section subtree remains intact;
- following siblings remain siblings;
- line endings and no-trailing-newline behavior are preserved according to the
  active file profile;
- undo/redo restores byte-exact source.

## 9. Test Requirements

Implementation must include tests for:

- Add Before first top-level section;
- Add Before middle top-level section;
- Add Before nested child section;
- Add Before a section with children;
- Add Before while a focused-body draft is dirty;
- Add Before in no-trailing-newline source;
- undo/redo source restoration;
- capability gating when Add Before is unsupported.

## 10. Manual QA Requirements

Manual QA must cover:

- first-section creation in an empty document via `+ Section`;
- another top-level section before the first top-level section;
- another top-level section after a selected top-level section;
- child creation via `+ Inside`;
- sibling-after-subtree creation via `+ After`;
- sibling-before-subtree creation via `+ Before`;
- keyboard-only traversal and activation;
- screen-reader labels for Before, Inside, and After;
- Japanese and narrow-panel visual checks.

## 11. Acceptance Criteria

- RFC-049 and RFC-053 amendments are reviewed and accepted.
- Document Map exposes `+ Before`, `+ Inside`, and `+ After` only when
  supported by the selected node capabilities.
- Empty/no-section behavior is explicit.
- Add Before is source-preserving, undoable, and tested.
- Row context menus remain focused on row mutation/organization actions.
- Documentation and i18n match the accepted labels.

## 12. Open Questions

1. Should placement-complete creation ship in M11/M12 with adapter work, or in
   a separate M14 polish milestone?
2. Should the command palette mirror Add Before immediately, or only after the
   Document Map UI lands?
3. Should no-selection state auto-select the first top-level section, or show
   an explicit prompt?
