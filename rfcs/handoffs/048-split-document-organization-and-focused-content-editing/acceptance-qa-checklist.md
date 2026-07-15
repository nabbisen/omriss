# RFC-048 Acceptance / QA Checklist

Use this checklist for the implementation PR series. Mark each item with the
PR number or gate run that proves it.

## Design Authority

- [ ] RFC-048 is implemented only as part of the M10 bundle with RFC-049,
      RFC-050, and RFC-051.
- [ ] RFC-053 type core is accepted, pulled forward, or explicitly mirrored for
      M10 without introducing throwaway names.
- [ ] No implementation uses `FocusedNodeKind` as a public/canonical type.
- [ ] No primary `Done` or `Commit -> Done` workflow remains.
- [ ] Existing spike code was audited against the amended RFCs.

## Layout and Labels

- [ ] Left panel is visibly labelled `Document Map`.
- [ ] Right panel is labelled `Writing Area` for Markdown.
- [ ] Internal right-panel component/boundary uses `FocusedContent` wording or
      another format-neutral equivalent.
- [ ] `Show plain file text` replaces normal-user `Raw Markdown` wording.
- [ ] `Quick Actions` replaces normal-user `Command Palette` wording.
- [ ] English and Japanese i18n catalogs contain the updated labels.
- [ ] Documentation uses the same terminology as the UI.

## Document Map Behavior

- [ ] Selecting a section in the Document Map updates the Writing Area.
- [ ] Current section is highlighted in the Document Map.
- [ ] Add inside is reachable from the Document Map when valid.
- [ ] Add after is reachable from the Document Map when valid.
- [ ] Rename is reachable from the Document Map when valid.
- [ ] Move up/down is reachable from the Document Map when valid.
- [ ] Move inside previous / out one level is reachable when valid.
- [ ] Merge into previous section content is reachable when valid.
- [ ] Delete is reachable when valid.
- [ ] Unsupported actions are hidden or disabled with plain explanations.
- [ ] Destructive actions require confirmation.
- [ ] Confirmation dialogs make canceling the safest/default path.
- [ ] Undo restores structure edits as before.

## Writing Area Behavior

- [ ] Writing Area contains no visible add, rename, delete, move, join, promote,
      demote, drag, or child-management controls.
- [ ] Markdown section body editing still works.
- [ ] Empty sections show helpful plain-language guidance.
- [ ] Optional smaller-section links are read-only navigation only.
- [ ] Preview shows focused Markdown content and does not add structure tools.
- [ ] Save/status affordance remains visible while previewing.
- [ ] Save status uses plain labels: `Saved`, `Not saved yet`, `Saving...`,
      `Saved ✓`, and friendly failure text.

## Draft Lifecycle

- [ ] Typing in a Markdown section updates a local focused draft.
- [ ] Selecting another section applies the valid pending draft first.
- [ ] Ctrl+S applies the valid pending draft before save.
- [ ] Preview applies or safely handles the valid pending draft.
- [ ] Search/navigation applies or safely handles the valid pending draft.
- [ ] No typed content is lost when switching sections.
- [ ] Invalid structured draft blocking is not implemented for Markdown-only
      M10, but the boundary does not prevent RFC-053 `DraftState` behavior.
- [ ] There is one user-visible document dirty state.

## Source Preservation

- [ ] Section body replacement preserves unrelated bytes.
- [ ] Rename preserves body and child sections.
- [ ] Move up/down preserves moved section bytes and unrelated text.
- [ ] Join preserves expected content.
- [ ] Delete removes only the confirmed range.
- [ ] Undo/redo restores exact previous source text for migrated commands.
- [ ] Line endings remain preserved.
- [ ] Existing Markdown source-preservation tests pass after final changes.

## Keyboard and Accessibility

- [ ] Toolbar/header is reachable by keyboard.
- [ ] Document Map is an accessible navigation/structure region.
- [ ] Markdown Writing Area is an accessible main editing region.
- [ ] Status/footer messages are exposed through live regions.
- [ ] Tab order is toolbar -> Document Map -> Writing Area -> status/recovery.
- [ ] Row menus are keyboard reachable.
- [ ] Esc closes open dialogs, menus, palette, or search before changing focus.
- [ ] Focus restores predictably after add.
- [ ] Focus restores predictably after rename.
- [ ] Focus restores predictably after move.
- [ ] Focus restores predictably after delete.
- [ ] Screen-reader labels are plain and useful.
- [ ] Error messages do not expose byte ranges, node IDs, parser internals, or
      raw system errors.

## Manual QA Scenarios

- [ ] Open a Markdown file, select a section, write, save.
- [ ] Add a section from Document Map, write in it, save.
- [ ] Rename a section from Document Map.
- [ ] Move a section up/down.
- [ ] Move a section inside previous and out one level.
- [ ] Join a section with the previous section.
- [ ] Attempt to delete a section with children and cancel.
- [ ] Delete a section with children and undo.
- [ ] Use only keyboard for the primary workflow.
- [ ] Open plain file text and return to editor.
- [ ] Close with unsaved changes and cancel.
- [ ] Confirm no visible structural controls exist in Writing Area.
- [ ] Have at least one non-technical Markdown user complete open, select,
      write, organize, save, and undo without coaching; record result.

## Documentation QA

- [ ] README summary terminology matches the migrated UI.
- [ ] mdbook getting-started docs match the migrated UI.
- [ ] mdbook structural-editing docs place organization in Document Map.
- [ ] mdbook editing/history docs describe apply-on-navigation accurately.
- [ ] keyboard reference lists current shortcuts and labels.
- [ ] architecture docs describe `DocumentMapPanel` /
      `FocusedContentPanel` boundaries.
- [ ] known limitations state that structured plain-text support is future work.
- [ ] CHANGELOG records the UI role separation.

## Required Commands

Run after final implementation changes:

```text
cargo fmt
cargo test -p omriss-core -p omriss-ui
cargo check -p omriss
mdbook build docs
bash scripts/check-rfcs.sh
git diff --check
```

Record exact output or CI links before claiming a gate passed.

## Release Gate

- [ ] No critical or high-severity accessibility issue is open.
- [ ] No Markdown open/edit/save regression is open.
- [ ] Rollback/release-gating decision from RFC-051 is followed.
- [ ] RFC-048 through RFC-056 developer handoffs are regenerated or patched to
      RFC-053 canonical names before the first RFC-048 implementation PR opens.
- [ ] No release archive, tag, or push is created without explicit owner
      approval.
