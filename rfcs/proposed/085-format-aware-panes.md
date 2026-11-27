# RFC-085: Format-Aware Panes

**Project:** omriss — Omriss Editor
**Milestone:** M13 — defect (0.18.0, item 7: RFC-067 §3.2's deferred scope)
**Status.** Proposed — PR 1–3 on `main` for 0.18.0 (`da66f4a`, `362462d`, `173d561`); PR 4 measured; PR 5 (`c007cc7`); PR 6 (`0f8bd66`). **Open:** PR 7 (§9); then `done/`.
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

## 7. Decisions on PR 1–4 (2026-11-27)

**F2 was real.** 0.17.0 added a newline (a CRLF in a CRLF file) to a JSON file
that ends in one, after the "Document" card was clicked and the user went
anywhere else. Undo restored it. It is fixed, and the CHANGELOG tells users.

Accepted as built:

1. **"Current Section" search includes the sections under the focused one, in
   Markdown too.** §3.3 item 5's rule. A section contains its subsections, so
   searching it should find them. This is the one deliberate change to Markdown
   behaviour, and the CHANGELOG says so.
2. A document with nothing under its root opens on the root. One rule, and
   what a Markdown file without headings has always done.
3. **Every format whose structure cannot be shown opens in plain file text:**
   unreadable JSON, TOML and YAML (no reader yet) and plain text. Each has its
   own sentence.
4. The guard covers the session's snapshot methods as well as `.outline()`.
   They read the same outline, so the wider list is the right one.
5. **Not building the Markdown outline for JSON (§4) is closed, not adopted.**
   Measured: 0.6–0.9 ms per build on a 374 KiB JSON file, under a fifth of a
   committed edit. Changing core's public `Document` API is not worth that.
   The guard makes the outline unreadable for JSON, so it cannot mislead.
   RFC-067's acceptance criterion 2 is answered by this measurement.

### PR 5

1. **No dead end beside an unreadable file.** "Back to editor" from plain file
   text still leads to one empty "Document" card, which opens an empty view
   (F8's remainder). Outside plain file text, the Writing Area of a file whose
   structure cannot be shown shows the Document Map's sentence (the same
   catalog key, not a copy) and a button to plain file text. It never shows an
   empty card or an empty view. The button stays where it is (RFC-080 §5.4
   rule 11).
2. **Measure the whole-document view of a large list.** A root array of 8,000
   items now renders 8,000 overview cards when the user climbs to the whole
   document. Measure that render and the Document Map's on the 374 KiB fixture,
   in a release build, and report. The architect decides from the numbers.
3. `overview.card.subsections_one`: "1 subsection", like `items_one`. In both
   locales.

## 8. Decisions on PR 5 (2026-11-27)

PR 5 is accepted. The editor view beside an unreadable file shows the
Document Map's sentence and a button to plain file text, and never an empty
card. Its measurement found two things.

### Findings

| # | Finding | Measured (8,000-item root array, 374–391 KiB, release) |
|---|---|---|
| P1 | **The sentence is untrue in the view PR 5 added.** "Its text is open in Plain file text" is shown beside a Writing Area where plain file text is *not* open | screenshot `pr5-unreadable-back-to-editor.png` |
| P2 | **Search and statistics are quadratic on JSON.** Each value's text is read through `focused_content`, which finds its node by a linear scan. Search runs this on every keystroke | 95 ms per call (objects), 18 ms (strings) |
| P3 | Overview cards for a list of scalars: one `focused_content` per card, for its kind label | 13.8 ms per render |
| P4 | **The Document Map rebuilds its whole tree on every focus change.** This predates RFC-085 and has been in the map since 0.17.0 | 0.4–1.2 s per row click |
| P5 | 8,000 overview cards when the user climbs to the whole document | 0.6–1.0 s, once |

### Design

1. **P1 — a sentence that is true wherever it is shown.** It says where the
   text *can* be seen, not that it is open: *"This file could not be read as
   JSON, so its structure cannot be shown. You can view and edit its text in
   Plain file text. When it is valid JSON, its structure is shown."* The TOML,
   YAML and plain-text sentences change the same way. Japanese keeps the same
   meaning.
2. **P2, P3 — a node's own text is the format's to say, and is read by node,
   not looked up by id.** `DocumentFormatAdapter` gains a required method that
   gives a node's own text from the node itself. It is §3.3 item 5's rule,
   moved to where the formats live: a Markdown section's body, a JSON value
   (decoded), none for a group or list, and the region for plain text. It
   returns whether the text is the source byte for byte. JSON's
   `focused_content` uses the same decoding, so there is one decoder path.
   `node_text.rs`'s per-format match becomes a call. It is required, with no
   default, so every format states its own text. This is a public API addition
   to `omriss-core`, stated in the CHANGELOG under `Added`.
3. **Node texts are cached by revision**, beside the structure (RFC-067 C1).
   Search per keystroke and the cards read the cache and decode nothing.
4. **P4 and P5 are not this RFC's.** Large structures in the Document Map and
   the overview — the map's rebuild per focus change, and the cards — need a
   design of their own. The owner decides its release.

## 9. Decisions on PR 6 (2026-11-27)

Accepted. Search on a 374 KiB JSON file takes about 2 ms per keystroke instead
of 95 ms. The cards of an 8,000-scalar list take 0.3 ms instead of 14.6 ms.
The sentence is true in both views.

- **`OwnText.value_kind` is accepted.** The kind comes out of the same decode.
  Without it a card would decode a second time, or the UI would keep a second
  copy of `classify`.
- **PR 7: the method is named for what it returns.** `node_text` returns an
  `OwnText`, from the `own_text` module, and "node text" reads as if it
  included the children. The method becomes `own_text`. It is unreleased, so
  the rename costs nothing now and a breaking change after 0.18.0. The
  CHANGELOG's `Added` entry follows.
