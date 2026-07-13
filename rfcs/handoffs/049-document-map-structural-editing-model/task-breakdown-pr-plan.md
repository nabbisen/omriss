# RFC-049 Task Breakdown / PR Plan

## PR 1 - Map View Model and Selection

- Render Document Map rows from current Markdown structure.
- Include title, depth, current selection, child state, and available actions.
- Keep `NodeId` internal and invisible.
- Highlight the current item.

Exit criteria:

- Selecting a row updates focused content.
- Current item is visually and accessibly identified.

## PR 2 - Row Actions

- Add row menu or equivalent action disclosure.
- Route add inside, add after, rename, move, join, delete, and show plain file
  text through the Document Map.
- Use `MoveDirection` terms from RFC-053 for internal routing.

Exit criteria:

- All required Markdown structure actions are reachable from the Document Map.
- No visible structure control remains in the Writing Area.

## PR 3 - Safety and Capability Gating

- Hide unsupported actions or show disabled actions with plain reasons.
- Add confirmations for destructive actions.
- Ensure undo remains available after structure edits.
- Keep keyboard access to row menus.

Exit criteria:

- Delete with children can be canceled.
- Invalid moves are unavailable or explained.
- Keyboard-only row action workflow works.

## PR 4 - Tests and Docs

- Add or update unit/component tests for row action availability and routing.
- Update docs that describe where organization happens.
- Run final gates from the acceptance checklist.
