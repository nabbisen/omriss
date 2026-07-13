# RFC-049 Acceptance / QA Checklist

## Document Map

- [ ] Document Map is visible as the left organization panel.
- [ ] Current section is highlighted.
- [ ] Selecting a row updates the Writing Area.
- [ ] Add inside is available when valid.
- [ ] Add after is available when valid.
- [ ] Rename is available when valid.
- [ ] Move up/down is available when valid.
- [ ] Move inside previous / out one level is available when valid.
- [ ] Merge into previous section content is available when valid.
- [ ] Delete is available when valid.
- [ ] Show plain file text is available where appropriate.

## Safety

- [ ] Destructive operations use confirmation.
- [ ] Cancel is the safest/default path.
- [ ] Unsupported actions are hidden or explained.
- [ ] No node IDs, byte ranges, heading levels, or parser terms appear in normal
      UI.
- [ ] Undo works after structure edits.

## Accessibility

- [ ] Document Map has an accessible label.
- [ ] Row actions are keyboard reachable.
- [ ] Focus restores predictably after add, rename, move, and delete.
- [ ] Disabled action explanations are plain and localized.

## Required Commands

```text
cargo test -p omriss
cargo test -p omriss-ui
cargo check -p omriss-app
bash scripts/check-rfcs.sh
git diff --check
```
