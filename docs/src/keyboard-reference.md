# Keyboard Reference

Omriss is fully usable with a keyboard. This page lists every shortcut.

*On macOS, **Cmd** is used where the table shows **Ctrl**.*

## File Operations

| Keys | Action |
|------|--------|
| Ctrl+O | Open a Markdown file |
| Ctrl+S | Save (commits any pending body edit first) |
| Ctrl+Shift+S | Save As — prompts for a file path |

## Navigation

| Keys | Action |
|------|--------|
| **↑** / **↓** | Move selection in the Document Map or overview |
| **Enter** | Zoom into the selected heading (when in overview) |
| **Esc** | Dismiss search/palette · zoom out one level (commits pending edit) |
| Alt+← | Back — return to the previous focus location (commits pending edit) |
| Alt+→ | Forward — re-enter the next focus location (commits pending edit) |

## Writing Area: Sibling and Depth Navigation

The navigation bar beneath the section title offers four buttons:

| Button | Action |
|--------|--------|
| ← Previous | Prev sibling (same parent, one earlier in source) |
| ↑ Parent | Parent section (or overview if at top level) |
| ↓ First Child | First child section |
| Next → | Next sibling (same parent, one later in source) |

Disabled buttons are shown but not interactive; they indicate the edge of the structure.

## Writing Area: Preview

Below the section text editor, the footer contains:

| Control | Action |
|---------|--------|
| **Preview** button | Toggle rendered Markdown preview (same as Ctrl+Shift+P) |

Structure actions are in the Document Map, not the Writing Area. Top-level
creation uses `+ Top`; selected-section creation uses `+ Inside` and
`+ After`.
See [Structural Editing](structural-editing.md) for details.

## Editing

| Keys | Action |
|------|--------|
| Ctrl+Z | Undo the most recent committed section edit |
| Ctrl+Y | Redo the most recently undone edit |
| Ctrl+Shift+Z | Redo (alternative binding) |

> **Note — undo and your text editor:**  
> While the Writing Area holds uncommitted text, **Ctrl+Z** applies the
> textarea's own undo. Once you save or move away, document-level undo restores
> the applied section text in one step.

## Search and Commands

| Keys | Action |
|------|--------|
| Ctrl+F | Open / close the search panel (commits pending edit before opening) |
| Ctrl+P | Open / close Quick Actions |
| Ctrl+\` | Toggle the read-only plain file text view (commits pending edit before opening) |
| Ctrl+Shift+P | Toggle the Markdown preview pane (RFC-045) |

**Search panel** — slide-in panel on the right; type to search case-insensitively
across the whole document or only the current section. Click a result to focus
its section.

**Quick Actions** — floating overlay; type to filter all actions by name.
Click or press Enter on a result to run the command.

## Tab and Focus Order

Pressing **Tab** moves through the toolbar buttons, the Document Map items,
and the main pane content in document order. All interactive elements are
reachable without a mouse.
