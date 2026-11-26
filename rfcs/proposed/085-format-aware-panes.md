# RFC-085: Format-Aware Panes

**Project:** omriss — Omriss Editor
**Milestone:** M13 — defect (0.18.0, item 7: RFC-067 §3.2's deferred scope)
**Status.** Proposed
**Document type:** Defect design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-053, RFC-054, RFC-067
**Related RFCs:** RFC-060, RFC-073, RFC-076, RFC-077, RFC-080

---

## 1. Summary

With a JSON file open, half of omriss reads the file's **Markdown heading
outline**, a parse that means nothing for JSON. The other half reads the JSON
structure. The two halves disagree in front of the user. RFC-067 §3.2 found
this in 0.17.0 and deferred it, because it needed the panes made format-aware
first. This RFC does that, and takes RFC-067 §3.2 from `done/`.

**One source of structure:** every pane reads the document's
`DocumentStructure` (RFC-053), whatever the format. The Markdown outline is
read only by code that edits Markdown.

## 2. Findings (survey by construction, 2026-11-26)

Every non-test reader of `Document::outline()` and of the session methods
built on it, in `omriss-ui`, `omriss-components` and `omriss-app`. One fact
explains most of what is wrong: Markdown, JSON and plain-text node ids come
from the same `NodeId::from_ordinal_path`, so **the root id is the same in
every format**. With a JSON file open, a lookup in the outline misses every
JSON node except the root, and finds the root.

| # | Where | With a JSON file open |
|---|---|---|
| F1 | Opening | no node is focused (`actions.rs:56` skips auto-focus for non-Markdown); the Writing Area shows one card, "Document", while the Document Map highlights the first row (RFC-067 O-3) |
| F2 | Overview card click (`overview_pane.rs:49`) | focuses the root and sets the draft to the **whole file** by `draft.set`, bypassing `draft_sync::sync`. Traced, not run: the draft would differ from the root's raw text by the final newline, so the next navigation may commit a change the user did not make |
| F3 | Enter / ↑ / ↓ in the overview (`current_children`, `outline_items`) | always one card, the root |
| F4 | Breadcrumb (`breadcrumb.rs`) | nothing for a JSON node, not even Up; "Document" for the root |
| F5 | Esc and Up (`zoom_out`) | always back to the overview, never to the JSON parent |
| F6 | Search (`search.rs:94`, `:135`) | one result per hit, every one labelled "Document", choosing one focuses the root; the section scope silently searches the whole file |
| F7 | Statistics (`stats.rs`) | "N words · 0 sections", in English in either locale |
| F8 | A `.json` that does not parse | the Document Map says "Add headings with # to begin organizing" (RFC-060 F-7); the Writing Area offers the "Document" card, which opens an empty view; nothing says the file could not be read |
| F9 | Valid JSON with no members (`{}`, `[]`, a scalar) | the same Markdown hint |

Not defects: the Document Map, a row click, focus by row, Alt+←/→, undo and
redo, and every structural action (all hidden for JSON) already use the
structure.

## 3. Design

### 3.1 One source of structure

Every pane that shows or navigates structure reads it through the session's
`DocumentStructure`, cached by revision (RFC-067 C1), for **every** format,
Markdown included: the overview's cards and its keys, the breadcrumb, Esc and
Up, search, statistics. Then there is one tree and no second one to disagree
with.

The Markdown outline stays where Markdown is edited: committing a section,
structural edits, the structure-change notice (RFC-071), and Markdown-only
labels such as a card's "H2". A source-scanning guard, in the style of the
commit-path guard, lists those places. Any other reader of `.outline()` in
the three application crates fails the test.

Markdown behaviour does not change. Its existing tests pass unmodified.

### 3.2 One way to open a node

A map row, an overview card, a search result and a crumb all open a node
through the same session call followed by `draft_sync::sync`. None sets the
draft itself. This is what F2 lacks.

### 3.3 Each surface, in every format

1. **Opening.** One rule for every format: focus the first top-level node. If
   there is none, show the whole-document view. If the structure cannot be
   read, see §3.4.
2. **The highlighted row is the focused node, and only that.** Nothing is
   highlighted in the whole-document view. The Markdown-only exception in
   `document_map_pane.rs` goes, along with its comment ("for other formats a
   row selection is not a focus").
3. **Overview cards** are the current level's children. A card's label is the
   one the format already uses: "H2" for a Markdown section, the kind word the
   JSON Writing Area shows (`focused_content.json.kind.*`, and the item count
   for a list or group).
4. **Breadcrumb, Esc and Up** follow the structure's parents. A top-level node
   goes up to the whole-document view.
5. **Search** looks at each node's own text: a Markdown section's body as
   today, and a JSON value. A group or list has no text of its own; its
   members are nodes. A result is labelled by its node's path and opens that
   node. The section scope is the focused node and its descendants. Titles
   (headings, JSON keys) are RFC-077's, which lands next on this same
   structure, so it covers JSON keys too.
6. **Statistics** count words over the same text, and nodes by the format's
   own noun: sections for Markdown, values for JSON. Both locales, through
   `t_args`.

### 3.4 A file whose structure cannot be read

Opening stays silent and lossless (`RELEASE_CHECKLIST.md` step 20). What the
user is told becomes true, says what to do, and does not blame them
(RFC-080 §5.4 rule 8):

- The file opens **in plain file text**, the one view that can show it.
  Editing it there is RFC-076's normal path.
- The Document Map says, in place of the Markdown hint: *"This file could not
  be read as JSON, so its structure cannot be shown. Its text is open in Plain
  file text. When it is valid JSON, the structure appears here."* Wording
  follows the format; Japanese has the same meaning.
- Valid JSON with no members (F9): *"This JSON has no keys or items to show."*
- The Markdown empty state and hint are shown for Markdown only.

## 4. Non-goals

- **The error's location** ("line 12, column 4"). Core's `StructureError` has
  none to give. That is a core change with a design of its own, recorded here
  as the next step if users need it.
- **Not building the Markdown outline for JSON** (RFC-067 C2, acceptance 2).
  Once §3.1's guard holds, nothing reads that outline for JSON, so it costs
  time but cannot mislead. Stopping core from building it changes core's
  public `Document` API, so it is decided on a measurement (§5), not here.
- Search for heading and key text: RFC-077.

## 5. Validation

- **F2 first:** a session-level test that opens a JSON file, opens the root
  the way the overview card does today, then navigates, and compares the
  source. The result says whether 0.17.0 could change a JSON file the user
  never typed in. If it could, the CHANGELOG says so under `Fixed`, as
  RFC-084 did.
- §3.1's guard, and its deliberate break.
- Level 2 (Dioxus runtime) per surface, for Markdown, valid JSON, an empty
  JSON container and an unreadable `.json`: the cards, the crumbs, the
  highlighted row, the empty-state text, the view opened.
- Search and statistics on a JSON fixture: each result's path and target.
- GUI, once, `en_US`: open a valid and an unreadable `.json`; the map's
  highlight matches the Writing Area; Esc climbs one level.
- **Measured, for §4:** building the Markdown outline over the 374 KB JSON
  fixture, per commit.

## 6. Acceptance criteria

1. No pane, key or message reads the Markdown outline for a non-Markdown
   document. The guard pins it.
2. The highlighted row and the Writing Area show the same node, in every
   format.
3. No JSON file is ever told to add `#` headings.
4. An unreadable `.json` opens in plain file text, with a message that says
   so and what makes the structure appear.
5. Markdown behaviour unchanged: its existing tests pass unmodified.
