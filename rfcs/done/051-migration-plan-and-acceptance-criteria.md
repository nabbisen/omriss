# RFC-051: Migration Plan and Acceptance Criteria

**Project:** omriss — Omriss Editor
**Milestone:** M10 — UI Role Separation
**Status.** Implemented (v0.16.0) — the M10 migration was executed and the QA
gate was run. §12 acceptance checklist is satisfied except the two items the
checklist itself defines as deferred: full accessibility/screen-reader
validation (RFC-060) and full non-technical-user walkthrough validation
(RFC-061), both recorded as *not run, not passed*. Owner accepted the residual
findings `RFC048-QA-010/013/020/021/022` as post-M10 follow-up on 2026-07-15.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-049, RFC-050, RFC-053 type core
**Related RFCs:** RFC-052, RFC-053

---

## 1. Summary

This RFC defines the migration plan for moving omriss to the new two-role GUI model:

- **Document Map:** organize and navigate document structure.
- **Writing Area / Focused Content Area:** edit, preview, and save focused content.

This update also ensures that the migration does not accidentally hard-code Markdown-only assumptions that would block later JSON/TOML/YAML support.

## 2. Migration goal

The migration should improve usability without destabilizing the source-preserving core.

The final user experience should satisfy this rule:

```text
Left side: organize the document.
Right side: work on the selected item.
```

For Markdown users this becomes:

```text
Left side: organize sections.
Right side: write the selected section.
```

## 3. Scope

### 3.1 In scope

- Rename the outline panel to Document Map.
- Split left and right panel responsibilities.
- Move visible structure controls out of the Writing Area.
- Add section action menus to the Document Map.
- Simplify the Writing Area.
- Update labels to plain language.
- Add or update confirmation dialogs.
- Update keyboard navigation.
- Update accessibility labels.
- Update tests and documentation.
- Introduce format-neutral UI naming where it does not increase immediate implementation risk.

### 3.2 Out of scope

- JSON/TOML/YAML implementation work.
- Format adapter trait implementation.
- Drag-and-drop section movement unless already stable.
- Full mobile layout.
- Plugin system.
- Collaborative editing.
- Rich-text editing.
- AI generation features.
- Changing the canonical file format.

## 4. Approval and sequencing prerequisites

RFC-048, RFC-049, RFC-050, and this RFC must be reviewed and accepted as one
M10 bundle before developer handoffs or implementation PRs are authorized.
RFC-048 is only the directional product decision; the sibling RFCs carry the
load-bearing behavior, migration, accessibility, and acceptance detail.

The M10 bundle depends on the RFC-053 type core even though RFC-053 belongs to
M11 as a full adapter architecture. The following RFC-053 concepts are pulled
forward as M10 prerequisites:

- `DocumentFormat`;
- `StructureNodeKind`;
- `NodeCapabilities`, `Capability`, and `CapabilityReason`;
- `StructureCommand` and `MoveDirection`;
- `DraftState`;
- reuse of the existing RFC-006 `NodeId`.

The full JSON/TOML/YAML adapter work remains out of M10 scope.

Any already-written M10/M11-aligned code in the worktree is treated as an
exploratory spike until this bundle is accepted. It is not evidence that the
RFCs are accepted, and it must not be used as the source of truth for handoffs.
Before release authorization, the owner must either accept the reconciled M10
bundle and move the affected RFCs through the lifecycle, or explicitly hold the
code behind a release gate/feature flag until that acceptance happens.

## 5. Migration phases

### Phase 1 — Layout split foundation

Goal: establish the permanent two-zone layout.

Tasks:

- Rename visible `Outline` label to `Document Map`.
- Introduce clear component boundary between `DocumentMapPanel` and `FocusedContentPanel`.
- Keep user-facing right panel label as `Writing Area` for Markdown.
- Ensure current selection is synchronized between panels.
- Ensure the Writing Area receives no visible structural callbacks.
- Keep existing structural operations temporarily available through hidden compatibility commands if needed.
- Rename internal identifiers that would block future formats when low-risk.

Acceptance:

- Selecting a section in the Document Map opens it in the Writing Area.
- Current section is highlighted in the Document Map.
- Writing Area shows section body editor only, with no child management controls.
- Existing Markdown open/edit/save tests still pass.

### Phase 2 — Move structure controls to Document Map

Goal: make the Document Map the only visible place for organization.

Tasks:

- Add `+ Add` to Document Map.
- Add row `⋯` menu.
- Move add/rename/delete/move/join actions into the Document Map.
- Remove structural toolbar from Writing Area.
- Remove editable child-section controls from Writing Area.
- Keep optional read-only smaller-section navigation links in Writing Area.
- Ensure every destructive operation uses confirmation.

Acceptance:

- No visible structural operation remains in the Writing Area.
- Add, rename, move, join, and delete are reachable from the Document Map.
- Delete explains what will be removed.
- Keyboard users can open row menus and activate actions.

### Phase 3 — Guided Writing Area

Goal: make the right side calmer and easier for non-technical users.

Tasks:

- Remove the primary `Commit` / `Done` workflow in favor of apply-on-navigation.
- Replace `Raw Markdown` with `Show plain file text`.
- Replace `Command Palette` with `Quick Actions`.
- Add helpful empty-section text.
- Clarify save state: `Saved`, `Not saved yet`, `Saving…`, `Saved ✓`.
- Ensure Ctrl+S applies pending draft before saving.
- Ensure selection changes apply or safely handle the pending draft.

Acceptance:

- New users can identify where to write without seeing structure tools.
- Save feedback is visible and plain.
- Empty sections guide the user.
- Technical labels are removed from normal UI.

### Phase 4 — Accessibility and safety hardening

Goal: ensure the migration is safe for keyboard and assistive technology users.

Tasks:

- Add or update landmarks.
- Update tree/list semantics for the Document Map.
- Ensure focus restoration after add/delete/rename/move.
- Ensure status messages use live regions.
- Ensure destructive dialogs focus Cancel by default or make Cancel the safest first action.
- Add reduced-motion behavior if not already present.

Acceptance:

- Complete workflow is possible by keyboard.
- Dialog focus never disappears.
- Screen-reader labels are meaningful and plain.
- Errors never expose parser internals.

### Phase 5 — Format-neutral readiness

Goal: prepare the UI layer for RFC-052+ without implementing new formats.

Tasks:

- Avoid hard-coded `section` naming in internal generic components.
- Ensure Document Map rows can carry a generic node kind.
- Ensure Focused Content wrapper can dispatch to a Markdown content view.
- Ensure unsupported future node types can render a safe placeholder.
- Document where future adapters will connect.

Acceptance:

- Markdown behavior is unchanged from the user perspective.
- UI component names and data flow can accept `Group`, `List`, `Value`, or `RawRegion` later.
- No JSON/TOML/YAML parser code is required in this migration.

## 6. Regression risk areas

| Risk | Mitigation |
|------|------------|
| Writing Area loses existing section-edit behavior | Preserve core section body draft lifecycle tests. |
| Structure operation accidentally remains in right panel | Add UI acceptance test / checklist inspection. |
| Focus jumps unpredictably after move/delete | Add focus restoration tests. |
| Non-technical users confused by labels | Apply label audit before release. |
| Future formats blocked by Markdown-only component names | Add Phase 5 format-neutral readiness review. |
| Source preservation broken | Re-run byte preservation tests after every structural operation. |

## 7. Rollback and release-gating decision

M10 changes the primary UI of shipped software. The default rollout policy is:

- develop on a branch or release gate until the accepted M10 bundle exists;
- do not open the first RFC-048 implementation PR until the RFC-048 through
  RFC-056 handoffs are regenerated or patched to current canonical names;
- keep the previous stable release tag as the rollback target;
- if the migrated UI reaches a release candidate before acceptance, ship it only
  behind an explicit feature flag or hold it from release;
- if Markdown open/edit/save, source preservation, or keyboard accessibility
  regresses after release, revert the M10 UI migration in the next patch release
  rather than partially disabling individual controls.

The project may choose a stronger branch-only strategy during implementation,
but it must not silently ship Proposed RFC behavior as accepted design.

## 8. Test plan

### 8.1 Unit tests

- Section body replacement preserves unrelated text.
- Rename preserves body and child sections.
- Move up/down preserves moved section bytes.
- Join section preserves expected content.
- Delete removes only the confirmed section range.
- Undo/redo works across migrated UI commands.

### 8.2 Component tests

- Document Map selection updates Writing Area.
- Row menu shows expected actions.
- Writing Area does not render structure controls.
- Empty section hint appears.
- Save status updates after automatic draft application and Save.
- Unsupported future node placeholder renders safely in test fixture.

### 8.3 Accessibility tests

- Tab order is toolbar → Document Map → Writing Area → status/recovery actions.
- Row menu is keyboard accessible.
- Dialog focus trap works.
- Esc closes open overlays before changing document focus.
- Live status messages are emitted.

### 8.4 Manual QA scenarios

1. Open Markdown file, select a section, write, save.
2. Add a section from Document Map, write in it, save.
3. Rename a section from Document Map.
4. Move a section up/down.
5. Attempt to delete a section with children and cancel.
6. Delete a section with children and undo.
7. Use only keyboard for the above.
8. Open raw source view and return to editor.
9. Close with unsaved changes and cancel.
10. Confirm no visible structural controls exist in Writing Area.
11. Confirm the role-split smoke check: normal UI labels make the left/right
    responsibilities visible (`Document Map`, `Writing Area`, `Quick Actions`,
    and `Show plain file text`). Full non-technical-user walkthrough validation
    is deferred to RFC-061 and must not be marked passed by this M10 smoke
    check.

## 9. Documentation updates

Required updates:

- User guide: replace “Outline” with “Document Map” where appropriate.
- User guide: describe left/right roles.
- Keyboard shortcut reference.
- Screenshot/wireframe updates.
- Release notes: explain that section organization moved to Document Map.
- Developer docs: document `DocumentMapPanel` / `FocusedContentPanel` boundaries.
- Future format note: mention that this split prepares omriss for structured plain-text files later.

## 10. Release criteria

The migration is release-ready when:

- all existing source-preservation tests pass;
- all migrated UI workflows pass manual QA;
- role-split smoke checks are recorded;
- no structural controls are visible in the Writing Area;
- Document Map supports the required Markdown structural operations;
- keyboard navigation works for the primary workflow;
- user-facing labels pass the plain-language audit;
- future format readiness checklist passes without implementing new formats;
- no known new critical or high-severity accessibility issue is open;
- full accessibility/screen-reader validation remains tracked by RFC-060 and
  is not claimed as complete by M10;
- full non-technical-user walkthrough validation remains tracked by RFC-061 and
  is not claimed as complete by M10;
- the rollback/release-gating decision in §7 is followed;
- all nine RFC-048–056 developer handoffs are regenerated or patched to RFC-053 canonical names and current `NNN-slug.md` filenames **before the first RFC-048 implementation PR opens**.

## 11. Future work after this migration

After RFC-048–051 are complete and stable, the project may proceed with:

- RFC-052: structured plain-text format expansion policy;
- RFC-053: document format adapter architecture;
- RFC-054: JSON structure view and focus editing;
- RFC-055: TOML structure view and preservation rules;
- RFC-056: YAML feasibility spike.

These must be treated as dependent follow-up work, not as part of the UI split migration itself.

## 12. Acceptance checklist

```text
[ ] Document Map is visible as the left organization panel.
[ ] Writing Area is visible as the right focused editing panel.
[ ] Structure controls are removed from Writing Area.
[ ] Document Map row menus contain structure actions.
[ ] Delete confirmation uses plain language.
[ ] Focused drafts apply on navigation (no primary Done control).
[ ] Show plain file text replaces Raw Markdown in normal UI.
[ ] Quick Actions replaces Command Palette in normal UI.
[ ] Ctrl+S applies valid pending draft before save.
[ ] Current item remains highlighted in Document Map.
[ ] Keyboard-only workflow succeeds.
[ ] Role labels are visible and plain (`Document Map`, `Writing Area`,
    `Quick Actions`, and `Show plain file text`).
[ ] Markdown byte preservation tests pass.
[ ] Internal UI boundary is format-neutral enough for RFC-052+.
[ ] RFC-053 type core is accepted or explicitly pulled forward for M10.
[ ] Rollback/release-gating decision is recorded and followed.
[ ] Full accessibility/screen-reader validation is deferred to RFC-060 and not
    marked passed by M10.
[ ] Full non-technical-user walkthrough validation is deferred to RFC-061 and
    not marked passed by M10.
[ ] RFC-048–056 developer handoffs are regenerated or patched before the first RFC-048 implementation PR.
```

## 13. Final decision summary

The migration should proceed in phases. RFC-048–051 remain the foundation of a simpler Markdown UI, but they are updated to avoid Markdown-only UI assumptions. JSON/TOML/YAML support must follow as new RFCs after the split is stable.
