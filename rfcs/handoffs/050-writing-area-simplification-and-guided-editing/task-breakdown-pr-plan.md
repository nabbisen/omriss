# RFC-050 Task Breakdown / PR Plan

## PR 1 - Panel Simplification

- Render focused title, editor, preview toggle, status, and optional read-only
  smaller-section links.
- Remove add, rename, delete, move, join, promote, demote, and child-management
  controls from the Writing Area.
- Keep raw-source escape hatch as `Show plain file text`.

Exit criteria:

- The Writing Area has no visible structure controls.
- Markdown section body editing still works.

## PR 2 - Draft Lifecycle

- Keep Markdown focused draft local while typing.
- Apply draft on selection change, save, preview, search/navigation, and safe
  blur.
- Remove primary Done/Commit workflow.
- Ensure Ctrl+S applies draft before save.

Exit criteria:

- Switching sections does not lose typed text.
- Saving captures the latest focused draft.

## PR 3 - Status, Preview, and Empty States

- Add plain empty-section guidance.
- Ensure preview remains focused and does not introduce structure tools.
- Use save statuses from RFC-050.
- Map validation/errors to plain messages.

Exit criteria:

- Status and validation text is clear and localized.
- Preview and edit modes are keyboard reachable.

## PR 4 - Accessibility and Tests

- Add accessible editor label.
- Associate validation/status messages with the editor or live region.
- Add tests/manual QA for draft application and keyboard use.
