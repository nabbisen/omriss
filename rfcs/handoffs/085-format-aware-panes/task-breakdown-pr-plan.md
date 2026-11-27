# RFC-085 — Task Breakdown and PR Plan

Related RFC: `../../proposed/085-format-aware-panes.md`

Lifecycle status: inherited from RFC-085.

In order.

## PR 1 — one way to open a node (§3.2), and F2 measured

- First, the test of RFC-085 §5's F2: open a JSON file, do what the overview
  card does today, navigate, compare the source byte for byte. Run it on the
  current code and **record the result before fixing**. If the source
  changes, the CHANGELOG `Fixed` entry says what 0.17.0 did and to which
  files, and the report says so at the top.
- Then the map row, the overview card, a search result and a crumb all open a
  node through one session call followed by `draft_sync::sync`. A guard that no
  component sets the draft outside `draft_sync`, if the existing guards do
  not already cover it. Say which.

## PR 2 — one source of structure (§3.1, §3.3 items 3–6)

- The overview's cards and keys, the breadcrumb, Esc and Up, search and
  statistics read `DocumentStructure` in every format.
- Statistics through `t_args`, both locales; JSON counts values.
- The guard, and its deliberate break.
- The dead code of handoff §2.7 deleted.

## PR 3 — opening and empty states (§3.3 items 1–2, §3.4)

- One opening rule for every format; the highlighted row is the focused node.
- An unreadable non-Markdown file opens in plain file text.
- The Document Map's empty states per format and per case (unreadable, no
  members); the Markdown text for Markdown only.
- Docs, and the GUI run (checklist).

**PR 1–3 landed** (`da66f4a`, `362462d`, `173d561`).

## PR 4 — the measurement for RFC-085 §4

Report only, no code: building the Markdown outline over the 374 KB JSON
fixture, per commit, in a release build, the same way RFC-067 §2 measured.
The architect decides from it.

**Done**: 0.6–0.9 ms per build; §4 closed (RFC-085 §7.5).

## PR 5 — RFC-085 §7

1. The Writing Area beside an unreadable file: the map's sentence (same key)
   and a button to plain file text; never an empty card or view. Level 2, and
   in the GUI: open the unreadable `.json`, choose "Back to editor".
2. The measurement of the whole-document view and the Document Map on the
   8,000-item array, release build. Report only.
3. `overview.card.subsections_one`, both locales.

**PR 5 landed** (`c007cc7`).

## PR 6 — RFC-085 §8

1. The sentences of §8.1, in both locales: map and Writing Area, all formats.
2. The adapter's own-text method (§8.2): required, implemented by every
   adapter. JSON's `focused_content` shares its decoding. `node_text.rs`
   calls it. Core's existing tests pass unmodified. The CHANGELOG has an
   `Added` entry for core.
3. Node texts cached by revision with the structure (§8.3). A test shows that a
   draft does not rebuild them and a commit does.
4. Measure again on PR 5's two fixtures, in the same way: search per call,
   statistics, `current_children` for 8,000 scalars. Before and after.
