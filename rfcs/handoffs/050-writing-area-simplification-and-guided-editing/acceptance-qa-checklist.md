# RFC-050 Acceptance / QA Checklist

## Writing Area

- [ ] Right panel is labelled `Writing Area` for Markdown.
- [ ] Internal boundary is format-neutral.
- [ ] Selected item title is visible.
- [ ] Markdown section text editor is usable.
- [ ] Preview is available and focused on selected content.
- [ ] Empty section guidance is plain.
- [ ] Optional smaller-section links are read-only.
- [ ] Show plain file text is available.

## Prohibited Controls

- [ ] No add control in Writing Area.
- [ ] No rename control in Writing Area.
- [ ] No delete control in Writing Area.
- [ ] No move/promote/demote control in Writing Area.
- [ ] No join/merge control in Writing Area.
- [ ] No child-section management control in Writing Area.
- [ ] No primary Done/Commit button.

## Draft Lifecycle

- [ ] Typing updates a local focused draft.
- [ ] Selection change applies the draft.
- [ ] Ctrl+S applies the draft before save.
- [ ] Preview applies or safely handles the draft.
- [ ] Search/navigation applies or safely handles the draft.
- [ ] No content is lost when changing sections.
- [ ] One user-visible dirty state is shown.

## Accessibility

- [ ] Editor has visible label or accessible name.
- [ ] Preview mode announces the view change.
- [ ] Status uses polite live region.
- [ ] Action failures use assertive live region only when appropriate.
- [ ] Keyboard-only workflow succeeds.

## Required Commands

```text
cargo test -p omriss-core
cargo test -p omriss-ui
cargo check -p omriss
git diff --check
```
