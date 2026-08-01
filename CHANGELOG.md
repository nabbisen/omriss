# Changelog

All notable changes to this project are documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- **RFC-052 product-scope documentation** — README and the mdBook
  introduction now describe omriss as a structured plain-text editor that
  starts with Markdown and is growing to support other plain-text formats,
  alongside a new `docs/src/file-formats.md` page recording the current,
  honest per-format status (Markdown fully supported; JSON and TOML planned;
  YAML under investigation) and the existing `.md`/`.markdown`/`.mdown`/`.txt`
  file-opening behavior. This is a documentation and policy change only: no
  format support is added, no file-dialog filter changed, and no code under
  `crates/` was touched.
- **Internal module split and clippy gate** — `crates/core/src/doc/structural.rs`
  and `crates/app/src/components/document_map_pane.rs` were split into smaller
  submodules to satisfy the project's file-size rule, and `index/index.rs` was
  renamed to `index/builder.rs` to fix a `clippy::module_inception` warning.
  Three `collapsible_if` warnings were also fixed using let-chains. The
  workspace now passes `cargo clippy --workspace --all-targets -- -D warnings`
  cleanly. This is a pure code-motion and lint-fix refactor with no
  user-facing change: all public APIs, `rsx!` markup, and behavior are
  unchanged, and the full test suite (239 tests) passes unmodified.
- **RFC-062 crate package name exchange** — the reusable document engine package
  is now `omriss-core`, while the desktop app package is now `omriss`. This
  makes `cargo run -p omriss` run the app. Library users should migrate Rust
  imports from `omriss::...` to `omriss_core::...`. The app package is marked
  `publish = false`; `omriss-core` is the publishable library package.

## [0.16.0] - 2026-07-15

### Changed

- **RFC-048 user-facing wording pass** — README, mdBook user pages, architecture
  notes, and GUI labels now consistently describe the left panel as the
  Document Map and the right panel as the Writing Area / Focused Content
  surface. Old "Command Palette" and raw-source labels were replaced with
  "Quick Actions" and "Plain File Text" wording in the English and Japanese
  catalogs.
- **RFC-048 re-review polish** — the RFC metadata now separates dependency
  and M10-bundle relationships, the wireframe uses the Markdown-facing
  `Writing Area` label, and the raw-source action consistently reads
  `Show plain file text`.
- **Header Search and Settings** — Search is now pointer-discoverable in the
  header, Settings contains the language selector, and the file path is shown
  in the footer only.

### Fixed

- **Focused drafts now apply before leaving the Writing Area** — toolbar and
  keyboard Back/Forward, search navigation, plain-file-text view, preview, and
  Open/New guards now account for local uncommitted text before changing view
  or document state.
- **Document Map action coverage** — row actions now include Add section after
  and Rename, backed by source-preserving section operations and undo history.

### Known Limitations

- **Quick Actions reverse Tab traversal deferred** — Quick Actions supports
  search, arrow-key command selection, Enter execution, and Escape/Ctrl+P
  dismissal, but Shift+Tab focus traversal inside the panel remains a known
  keyboard limitation tracked under RFC-059.

### Removed

- **Obsolete pre-RFC-048 app components** — removed the unexported legacy
  `FocusEditor` and `OutlinePane` source files so the tree no longer carries
  the old right-side structural-toolbar implementation.

## [0.15.6] - 2026-06-25

### Changed

- **"Show file text" moved from `⋯` row menu to the left panel footer** —
  replaces the "Up one level" button at the bottom of the Document Map. Always
  visible regardless of focus state. Styled as a dashed footer button.

- **"Up one level" moved to the breadcrumb right edge as an ↑ icon button** —
  removed from the Document Map panel; now lives in the breadcrumb bar of the
  right panel, flush right. Commits any pending draft before navigating up,
  then syncs the draft to the new focus.

- **Row click selects the section** — clicking anywhere on a `dx-swdir-row`
  now selects it and focuses the session, not just clicking the label span.
  The label has `pointer-events: none` so clicks pass through to the row div.
  The `+` and `⋯` buttons still stop propagation so they don't also trigger
  a row selection.

- **Action buttons always in a horizontal group** — `dx-swdir-row` is now
  explicitly `display: flex; align-items: center` so the caret, icon, label,
  and action buttons always sit in a single horizontal line. Removed the stray
  `background` declaration that was left from an earlier edit.

- **Auto-focus first section on file open** — after loading a file the first
  top-level section is immediately focused so the editor is ready without an
  extra click. On "New" (blank document) the outline panel is shown ready and
  the user clicks `+ Add section` to create the first H1; no dialog opens
  automatically.

- **Synthetic root row no longer rendered in the Document Map** — the document
  root (no title, no actions) was previously drawn as a blank first row, which
  shifted every real section row down one visual position. The `+`/`⋯` buttons
  then aligned to the wrong row, so clicking H1's `+` actually targeted H2 and
  added an H3. The root is now filtered out before rendering; H1 sections start
  at indent 0 and click targets line up with what the user sees.

### Fixed

- **H1 section showed no `⋯` menu** — the `⋯` button now correctly stops
  propagation so the `aside`'s `close_menu` handler does not fire in the same
  event bubble and instantly dismiss the menu that was just opened. Combined
  with row-level selection (the row is focused before `⋯` is clicked),
  `find_node` reliably finds the correct node to render `NodeRowMenu`.

- **"Add section" on an H1 added an H3 instead of an H2** — caused by the blank
  root row shifting click targets (see above). With the root row suppressed,
  the `+` button on an H1 row focuses the real H1, and the child level is
  computed as H2.

- **"New" no longer auto-opens the Add Section dialog** — a New document now
  shows the blank editor with the outline panel ready; the dialog opens only
  when the user explicitly clicks `+ Add section`.

## [0.15.5] - 2026-06-24

### Changed

- **MSRV raised to Rust 1.88** — the dependency tree requires it:
  `dioxus-swdir-tree-core 0.9.1` uses let-chains (stabilized in 1.88) and
  `rfd 0.17.2` (a transitive dependency of `dioxus-desktop 0.7.9`) only
  compiles on 1.88+. The previous `rust-version = "1.87"` could not build the
  app crate. Updated `Cargo.toml` and `README.md` accordingly.

- **Language switcher moved to status bar footer** — removed from the toolbar
  header; added to the right end of the `StatusBar` footer. Styled as
  `.statusbar-locale`: `background: transparent`, `color: var(--muted)`,
  `border: none`, `font-size: 11px`. Using CSS variables means it renders
  correctly on both light and dark OS themes without the contrast issues that
  appeared in the toolbar's OS-native `<select>`.

- **Outline item icon changed from `§` to `#`** — `#` is the Markdown heading
  marker, directly communicating "section" in this editor's context. Bold 11px,
  no font file required.

### Fixed

- **Outline tree item could not be collapsed** — the `use_effect` in
  `DocumentMapPane` re-expanded ancestors of the focused node on every session
  change, immediately undoing any manual collapse. Fixed by gating the expand
  pass on `item_tree.is_expanded(focused_sw).is_none()`: a `None` result means
  the node is new to the tree and needs its ancestors opened; an existing node
  is left alone so the user's expand/collapse state is respected.

## [0.15.4] - 2026-06-24

### Changed

- **Selected outline item is now visually prominent** — the widget's default
  selection highlight (`rgba(66,133,244,0.18)`) was too faint on the dark
  panel background. Overridden with `var(--accent-soft)` fill plus a solid
  `1px var(--accent)` inset outline, matching the app's existing focus-ring
  language.

- **Outline icons replaced** — `dioxus-swdir-tree` renders 📁/📂/📄 emoji by
  default (the `UnicodeTheme`), which carry file-system semantics inappropriate
  for a section outline. The emoji are suppressed via `font-size: 0` on
  `.dx-swdir-icon` and replaced with a neutral `§` glyph injected via CSS
  `::before`. The caret (`▸`/`▾`) already communicates branch vs leaf, so a
  single consistent icon is sufficient. No Rust changes required.

### Fixed

- **"Add section" dialog input not focused** — `autofocus: true` only fires on
  initial page load in a WebView; dynamically-mounted elements are ignored by
  the browser. Replaced with a `use_effect` call to `document::eval` that
  programmatically focuses the `.split-dialog-input` element after mount.

## [0.15.3] - 2026-06-24

### Fixed

- **"Add section" result not reflected immediately** — after confirming the
  split dialog, the Document Map and Focused Content pane showed no change
  until two or three more interactions forced a refresh. Root cause: the
  `handle_split_choice` action handler mutated the session but never called
  `sync_draft`, so the `draft` signal kept the pre-split body text and the
  right panel displayed stale content. The Document Map tree updated
  eventually only because unrelated session reads triggered its `use_effect`.
  Fixed by:
  1. Calling `sync_draft` after every successful split so the right panel
     immediately reflects the new section's body.
  2. Navigating focus to the newly created section after the split, so the
     user lands directly in the new section ready to write.
  3. Handling "Add section" from overview mode (no focused section) with a
     new `EditorSession::add_top_level_section` method that appends an H1
     at the document root, instead of failing silently with
     `CannotDeleteRoot`.

## [0.15.2] - 2026-06-24

### Changed

- **Domain-based source grouping across all three crates.** Each crate's
  `src/` directory is now organised by domain rather than left as a flat list
  of files.

  `omriss` (core):

  | Before | After |
  |--------|-------|
  | `src/document.rs`, `edit.rs`, `history.rs`, `revision.rs` | `src/doc/` |
  | `src/index.rs`, `outline.rs` | `src/index/` |
  | `src/preview.rs`, `structural.rs` | `src/doc/` (operate on `Document`) |
  | `src/error.rs`, `range.rs` | `src/` (cross-cutting primitives, kept at root) |

  `omriss-ui`:

  | Before | After |
  |--------|-------|
  | `src/navigation.rs`, `search.rs`, `view_state.rs`, `stats.rs` | `src/editor/` |
  | `src/file_profile.rs` | `src/file/` |
  | `src/commands.rs`, `document_map.rs` | `src/interface/` |
  | `src/i18n.rs` + `src/i18n/` | unchanged |
  | `src/session.rs` + `src/session/` | unchanged |

  `omriss-app`:

  | Before | After |
  |--------|-------|
  | `src/app.rs`, `app_ctx.rs`, `actions.rs`, `dispatch.rs` | `src/shell/` |
  | `src/file_dialog.rs` | `src/file/` |
  | `src/keyboard.rs` + `src/keyboard/` | `src/input/` |
  | `src/settings.rs` + `src/settings/` | `src/storage/` |
  | `src/components.rs` + `src/components/` | unchanged |

  Public API and all test counts are unchanged.

## [0.15.1] - 2026-06-24

### Changed

- **`dirs` dependency upgraded from `"5"` to `"6"`** — Dioxus already pulls
  `dirs v6` transitively; the workspace was pinning a duplicate `v5` copy.
  Upgrading to `"6"` removes the redundant compiled copy.

- **`Cargo.toml` internal dependency version references** changed from exact
  patch (`"0.15.0"`) to minor-only (`"0.15"`) so the workspace `version` field
  does not need to be updated in `[workspace.dependencies]` on every patch
  release.

- **Rust 2024 module style applied** — `mod.rs` files replaced by same-name
  sibling files where applicable:
  - `crates/app/src/components/mod.rs` → `crates/app/src/components.rs`
  - `crates/ui/src/i18n/mod.rs` → `crates/ui/src/i18n.rs`

- **Inline tests extracted to separate files** — `#[cfg(test)] mod tests { … }`
  blocks removed from source files; test code now lives in dedicated files:
  - `crates/app/src/keyboard.rs` → `crates/app/src/keyboard/tests.rs`
  - `crates/app/src/settings.rs` → `crates/app/src/settings/tests.rs`

- **`crates/core/tests/structural_ops.rs` split** — the 293-line monolithic
  integration test file is now a thin driver that delegates to four focused
  files in `tests/`:
  - `structural_ops_promote_demote.rs`
  - `structural_ops_delete_split_merge.rs`
  - `structural_ops_move_ops.rs`
  - `structural_ops_revision_guard.rs`

- **`crates/core/src/` logical grouping** introduced in preparation for
  structured-format adapters (RFC-052/053):
  - `src/doc/` — document model (`document`, `edit`, `history`, `revision`)
  - `src/index/` — heading tree (`index`, `outline`)
  - `error.rs`, `range.rs` remain at `src/` as cross-cutting primitives

- **CSS styles added** for `DocumentMapPane` and `FocusedContentPane` — the
  two components introduced in v0.15.0 shipped without matching stylesheet
  rules, leaving the left panel invisible. Styles for `.document-map-pane`,
  `.row-menu`, `.focused-content-pane`, and related classes are now present.

### Fixed

- **`components/mod.rs` doubled on build** — a previous `str_replace` appended
  a second copy of the original module block, producing 40 duplicate-definition
  errors when building `omriss-app`. The file is now a single clean block.

- **`FocusEditor` and `OutlinePane` unused-import warnings** — both superseded
  components were removed from the components module and from the `app.rs`
  import list. They are no longer compiled into the binary.

- **`DocumentMapNode` missing `PartialEq`** — the Dioxus `#[component]` macro
  requires all prop types to implement `PartialEq` for change detection.
  `PartialEq` is now derived on `DocumentMapNode`.

- **`focus_then` closure borrow error** — the closure in `NodeRowMenu` was
  declared immutably but captured signals requiring mutable access. Declared
  `mut`.

- **`DocumentMapPane` froze on "New"** — a Dioxus reactive loop: `session` was
  subscribed both inside `use_effect` and directly in the component render body
  (`let map_root = session.read()…`). A `session` change triggered the effect,
  which wrote a local signal, which triggered a re-render, which re-read
  `session`, which re-triggered the effect — infinite. Fixed by reading
  `session` only inside `use_effect` and storing the derived tree in a second
  local signal (`map_root_sig`) that the render body reads instead.

## [0.15.0] - 2026-06-24

### Added

- **Document Map panel** (`DocumentMapPane`) replaces the outline sidebar as
  the left-panel structure-organization surface (RFC-048, RFC-049). The panel
  owns all structural editing actions (move up/down, move inside/out, join with
  previous, delete, add section); they are no longer visible in the right
  content area. Per-node `⋯` row menus expose only the actions the active
  adapter permits, with plain-language disabled reasons from the i18n catalog.

- **Focused Content pane** (`FocusedContentPane`) replaces `FocusEditor` as
  the right panel (RFC-050). It contains only the body textarea, Preview
  toggle, save status, and read-only child navigation links. All structural
  controls have been removed from this area. Drafts apply on navigation, blur,
  save, and preview (apply-on-navigation; no primary "Done" action).

- **`omriss-ui::document_map` module** with `DocumentMapNode`,
  `MapNodeCapabilities`, `MapCapability`, `CapabilityReason`, and `DraftState`
  — the RFC-053 format-neutral types, Dioxus-free and fully unit-tested.

- **`EditorSession::document_map_nodes()`** builds a `DocumentMapNode` tree
  with per-node granular capability flags (`can_move_up`, `can_move_down`,
  `can_move_inside_previous`, `can_move_out_one_level`, `can_join_with_previous`,
  `can_add_inside`, `can_add_after`, `can_rename`, `can_delete`,
  `can_show_plain_text`) derived from sibling/parent state without exposing
  Markdown internals to the UI.

- **16 new i18n keys** (EN + JA) covering capability disabled reasons, Document
  Map action labels, and panel titles (`document_map.*`,
  `capability.disabled.*`, `focused_content.*`).

- **16 new unit tests** in `omriss-ui` for `DraftState`, `CapabilityReason`
  catalog keys, `DocumentMapNode` tree shape, and all capability edge cases.

### Changed

- `app.rs` now renders `DocumentMapPane` (left) + `FocusedContentPane` (right
  in focus mode) instead of `OutlinePane` + `FocusEditor`. Both old components
  remain compiled in for reference during the transition.

- RFC directory updated: RFC-048–056 imported to `rfcs/proposed/` in repo
  conventions (`NNN-slug.md`, `**Status.** Proposed` header). RFC-000 lifecycle
  checks pass. README index updated; next free RFC number: 057.

### RFC compliance

Implements RFC-048 (role split), RFC-049 (Document Map structural editing
surface), RFC-050 (Focused Content panel, no structure controls), RFC-051
(migration: old components kept, new components wired into layout). Lays
groundwork for RFC-052/053 (format adapter architecture). Does not touch
RFC-001 boundary: core has no Dioxus dependency; `dioxus-swdir-tree` remains
confined to `omriss-app`.

## [0.14.1] - 2026-06-23

### Changed

- GitHub badges.

## [0.14.0] - 2026-06-23

### Changed

- **App renamed to Omriss** (*omriss* = outlines in Norwegian).

- **Crate and directory restructure:**

  | Old | New |
  |-----|-----|
  | `crates/omriss-core/` · `omriss-core` | `crates/core/` · `omriss` |
  | `crates/omriss-ui/` · `omriss-ui` | `crates/ui/` · `omriss-ui` |
  | `crates/omriss-desktop/` · `omriss-desktop` | `crates/app/` · `omriss-app` |

  The `-desktop` suffix is dropped because there is no `-mobile` counterpart;
  `-app` is the neutral choice. Short directory names (`core/`, `ui/`, `app/`)
  are used inside `crates/`; the full crate names carry the `omriss` prefix.

- The produced binary is named **`omriss`** (via an explicit `[[bin]]` entry
  in `crates/app/Cargo.toml`), not `omriss-app`.

- All user-visible strings, config paths (`~/.config/omriss/`), Rust import
  paths (`use omriss::`, `use omriss_ui::`), and documentation updated.

- Upstream project edits applied: Cargo workspace reformatted with aligned
  keys and workspace-shorthand notation; `LICENSE` copyright filled in
  (`2026 nabbisen`); `.gitignore` expanded; `.vscode/` settings added;
  `crates/ui/src/session/mod.rs` renamed to `crates/ui/src/session.rs`
  (Rust 2024 module style).

## [0.13.3] - 2026-06-14

### Audit — five-dimension codebase review

**1 — RFC compliance:** All 47 done RFCs (000–046) verified against the
codebase. Every RFC has corresponding production code. No gaps found.

**2 — Dead code removed:**

- `AppSettings::clear_recent()` in `settings.rs` removed. It had no caller,
  no test, and no mention in any RFC or requirements document. The
  `#[allow(dead_code)]` suppression on `remove_recent()` is retained — that
  method is tested, has a clear future-UI use case, and shares the same
  pattern as `push_recent`.
- `AppSettings` fields `font_size` and `line_wrap` retained — they are
  explicitly planned in RFC-036 and requirements §9.8, and survive
  round-trips through the TOML settings file.

**3 — Tests extended to match requirements:**

The external design (§25) and app requirements (§17.1) mandate semantic
assertions for each fixture, not only byte-preservation. Nine fixtures had
only byte-preservation coverage in the source-preservation test suite.
Nine semantic tests were added to `fixture_catalog.rs`:

| Fixture | Assertion added |
|---------|----------------|
| `code_fences.md` | Heading-like text inside a fence is not indexed |
| `setext.md` | Setext headings are indexed as sections |
| `crlf.md` | CR+LF line endings survive parsing |
| `skipped_levels.md` | Skipped heading levels are handled without data loss |
| `no_headings.md` | A heading-free document has no child sections |
| `empty_bodies.md` | Sections with no body text still appear in the outline |
| `html_content.md` | An HTML comment containing `# Heading` is not indexed |
| `no_trailing_newline.md` | Source is byte-identical after parsing |
| `toml_front_matter.md` | TOML front matter is not treated as a heading |

**4 — Code–test alignment:** Structural ops and session tests were audited
against the production code. All assertions reflect actual behaviour. No
mismatches found.

**5 — Documentation corrected:**

- `README.md`: MSRV corrected from `1.85+` to `1.87+`.
- `docs/src/keyboard-reference.md`: "Edit" button → "Done"; "Add Child"
  → "Add section"; bare `⋯` toggle → "Arrange"; note that **Done** is
  only visible when the section has unsaved changes.
- `docs/src/structural-editing.md`: opening description updated to reflect
  the current layout (Arrange toggle for rearrangement ops; Add section
  always-visible in the child-sections area); "Add Child Section" heading
  renamed to "Add section".

### Changed

167 tests total (was 158).

## [0.13.2] - 2026-06-09

### Changed

Plain-language and clarity improvements adopted from an external UX review,
selected to fit layered's audience (Markdown-literate writers) rather than
the review's broader "non-technical user" framing. Wholesale changes that
conflicted with the product identity — hiding the word "Markdown", a
touch-first 16px/44px redesign, loading-spinner buttons for instant local
operations — were intentionally not adopted.

- **Button and control labels clarified:**
  - "Edit" (the body-commit button) → **"Done"**. The button committed the
    current draft; labelling it "Edit" while already editing was confusing.
  - "Add Child" → **"Add section"**. "Child" is tree-structure jargon.
  - The `⋯` structural toggle now shows a visible **"Arrange"** label
    beside the glyph instead of being icon-only, closing a discoverability
    gap.
  - Unsaved-changes dialog: "Discard" → **"Leave without saving"**.

- **Error messages rewritten to be calmer and recovery-oriented:**
  - Open failure now suggests trying another file.
  - Save failure now reassures that the writing is safe and points to
    Save As, rather than just stating the failure.
  - "Heading level limit reached" → "This section can't move any deeper."
  - "No adjacent sibling to merge with" → "There's no nearby section to
    merge into."

- **Accessibility:** the per-child `×` delete button gained an `aria-label`
  ("Delete section") so screen readers announce its purpose instead of just
  reading the glyph.

## [0.13.1] - 2026-06-09

### Changed

- **"Add section" button is now always visible.** Previously it was hidden
  inside the `⋯` structural toolbar, requiring two clicks before a child
  section could be created. It is now a persistent `+ Add section` button at
  the bottom of the child-sections area in focus mode — visible on every
  focused section, whether it already has children or not. The structural
  toolbar (`⋯`) retains the rearrangement and merge operations but no longer
  contains Add section.

- **Per-child delete button.** Each child section card now has a `×` button
  that navigates into the child and immediately opens the existing delete-
  confirmation dialog. This removes the two-step "zoom in, then open ⋯ and
  Delete" flow. Delete still requires confirmation; the `×` is styled as a
  subtle secondary action and uses the danger colour on hover.

- **Outline tree auto-expands on Add section.** After a child section is
  created, `OutlinePane` detects the node-count increase in `use_effect` and
  expands the parent node in the `ItemTreeView` if it was collapsed, making
  the new section immediately visible in the outline.

## [0.13.0] - 2026-06-09

Minor release consolidating the 0.12.1–0.12.7 patch series into a single
tagged version for downstream testing. The detailed entries for each patch
remain below. Highlights of the series:

- **Outline rendering** moved to `dioxus-swdir-tree` v0.9's `ItemTreeView`
  (0.12.5), retiring the hand-rolled list.
- **"Less is more" UX pass** (0.12.6): structural toolbar collapsed behind a
  toggle, status bar trimmed, welcome tutorial removed.
- **Architecture cleanup** (0.12.7): `app.rs` 573→199 ELOC and `session.rs`
  474→307 ELOC via focused module splits; dual selection models unified;
  document statistics wired into the command palette.
- **Correctness fixes**: i18n sort-order and missing-key bugs, a clippy
  lint, and several stale-documentation corrections.

### Added

- Pure-logic unit tests for the desktop shell: keyboard shortcut mapping
  (`interpret_code`) and recent-files management (`AppSettings`). 158 tests
  total. Documented in `TESTING.md` why the Dioxus component/hook/end-to-end
  testing styles are intentionally not used.

## [0.12.7] - 2026-06-09

### Changed

All three "findings to track" from the v0.12.6 audit report resolved.

**A — ELOC guideline: over-500 and over-300 files split**

- `app.rs` (573 → 199 ELOC) split into four files. `Signal<T>: Copy` lets
  every action handler take an `AppCtx` struct (one bundled argument) instead
  of eight separate signals:
  - `app_ctx.rs` — `AppCtx` bundle, `Modal` enum, `sync_draft`,
    `commit_pending`
  - `actions.rs` — all file/session action handlers (`handle_load`,
    `handle_save`, `handle_open_guarded`, `handle_new_guarded`,
    `handle_unsaved_choice`, `handle_ext_modified_choice`,
    `handle_confirm_delete`, `handle_split_choice`)
  - `dispatch.rs` — keyboard command dispatch (`dispatch_command`) and
    palette dispatch (`dispatch_palette`)
  - `app.rs` — signal declarations, `use_callback` wrappers (one line
    each), sentinel intercept, and the `rsx!` render tree

- `session.rs` (474 → 307 ELOC) split into a submodule:
  - `session/mod.rs` — struct, constructors, accessors, navigation, edit ops
  - `session/structural.rs` — structural editing façade (RFC-023..026):
    `can_promote` / `promote_focused` / `demote_focused` / `move_focused` /
    `merge_focused_up` / `split_focused` / `delete_focused` and the six
    `can_*` guards
  - `session/outline_bridge.rs` — `OutlineNode`, `outline_nodes()`,
    `build_outline_node`

**B — Dual selection models unified**

- `selected_card: Signal<usize>` removed from `OutlinePane` props and its
  sync code removed. `ItemTreeView` (dioxus-swdir-tree) manages its own
  selection and keyboard navigation internally; `selected_card` now lives
  only in `OverviewPane` where it drives the main-canvas card highlight.

**C — Stats module wired to UI**

- "Show Statistics" command (`view.stats`) added to the command palette.
  Selecting it writes word count and section count to the status bar as a
  one-time message, making `EditorSession::stats()` reachable from the UI.
  `view.stats` i18n key added to both `en` and `ja` catalogs in correct
  sorted position.

### Fixed

- `i18n/en.rs` and `i18n/ja.rs`: `struct.toolbar.toggle` was out of sort
  order (before `struct.delete` instead of after `struct.split`), breaking
  the binary-search catalog invariant. `welcome.tagline` was never inserted
  (multi-line match failed silently). Both corrected; 47 i18n tests pass.
- `preview.rs`: collapsed two identical link-rendering branches (clippy
  `clippy::if_same_then_else`).

## [0.12.6] - 2026-06-09

### Changed

Three UI simplifications applying the "less is more" design principle —
users start as immature; advanced features should be discoverable, not
front-and-centre.

- **Structural toolbar collapsed by default.** The seven structural-editing
  buttons (Promote, Demote, Move ↑, Move ↓, Merge ↑, Split, Delete) are now
  hidden behind a single `⋯` toggle button in the focus editor. One click
  reveals the toolbar; another hides it. The toolbar state is local to the
  current focus session (resets when zooming out). Expert users still have
  full access; new users are no longer confronted with seven unfamiliar
  buttons on first use.

- **Status bar trimmed.** Word count, section count, and the newline-style
  label (LF/CRLF) have been removed from the always-visible status bar.
  Remaining: live status/error messages, unsaved-changes indicator, file
  name. The RFC-046 statistics module is retained in `layered-ui` for future
  use (e.g. a command-palette "Show Statistics" action).

- **Welcome screen tutorial removed.** The five-step onboarding list was
  shown on every launch, becoming noise for returning users. Removed.
  The title, tagline, Open/New buttons, and recent-files list remain.

## [0.12.5] - 2026-06-09

### Changed

- **Outline pane now uses `dioxus-swdir-tree` v0.9's `ItemTreeView`.**
  `dioxus-swdir-tree` v0.9.0 shipped RFC-012 (generic item tree) and
  RFC-013 (item tree drag-and-drop), which implement exactly what the
  feature request described. The hand-rolled `OutlinePane` for loop is
  replaced by `ItemTreeView<String>`:
  - Expand/collapse, keyboard navigation (Up/Down/Left/Right/Enter/Home/End),
    and incremental search are handled by the widget.
  - `ItemTree::set_tree` is called on every session change; key-based
    diffing preserves expansion state across pure body edits and resets it
    when the heading structure changes (expected behaviour).
  - Drag-and-drop is disabled (not enabled via `with_drag_and_drop`);
    structural editing remains through the existing toolbar buttons.
- `EditorSession::outline_nodes() -> OutlineNode` added to `layered-ui`:
  converts the full document `Outline` into a `OutlineNode` tree (plain
  `u64` keys + `String` titles) without exposing `layered-core` types to
  the desktop crate.
- `dioxus-swdir-tree = { version = "0.9", default-features = false }` added
  to `layered-desktop` dependencies (`default-style` disabled; the existing
  `assets/style.css` themes the outline pane instead).

## [0.12.4] - 2026-06-07

### Fixed

- **Outline pane showed nothing.** The `for` loop in `OutlinePane` used a
  `{ let …; rsx! { … } }` block as its body. In Dioxus 0.7 this pattern
  silently produces no output — the inner `rsx!` call inside a Rust block
  is not forwarded to the parent element. Fixed by removing the wrapper
  block and capturing per-iteration values directly in the event-handler
  closures (`let id = item.id; move |_| { … }`), which is the correct
  Dioxus 0.7 pattern.

## [0.12.3] - 2026-06-07

### Fixed

- **"New" button on the welcome screen did nothing.** `is_welcome` was
  computed as `source().is_empty() && !is_dirty()`, which remained true
  after "New" because a freshly created empty document satisfies both
  conditions. Fixed by adding a `document_open: bool` field to
  `EditorSession`:
  - `new_empty()` sets `document_open = false` — startup placeholder,
    keeps the welcome screen.
  - New method `new_document()` sets `document_open = true` — used by all
    three "New" code paths (direct click, save-then-new, discard-then-new),
    dismisses the welcome screen and shows the editor.
  - `is_welcome` is now simply `!session.document_open()`.

## [0.12.2] - 2026-06-07

### Fixed

- **Linux build failure**: added `libjavascriptcoregtk-4.1-dev` and
  `libssl-dev` to the required system package list in `PLATFORMS.md`,
  `getting-started.md`, and the `Cargo.toml` comment. Both packages are
  required by Dioxus 0.7's desktop renderer on Linux but were missing from
  the documentation.

### Changed

- **MSRV restored and corrected to 1.87**: `rust-version = "1.87"` is now
  set in the workspace `Cargo.toml`. The previous value of 1.85 was removed
  in v0.12.1 because it could not be verified; a full scan of the 631-package
  dependency tree (`cargo metadata`) found that `wit-bindgen 0.51` (a
  transitive dep of Dioxus 0.7) requires Rust 1.87. All other transitive
  dependencies state 1.85 or lower.

## [0.12.1] - 2026-06-07

### Changed

- **Renamed: `layerd` → `layered` throughout the entire codebase.** The
  previous name was a typo/abbreviation that caused confusion. All crate
  names (`layered-core`, `layered-ui`, `layered-desktop`), Rust module
  paths, the app title in both i18n catalogs, the OS config directory
  (`~/.config/layered/`), and all documentation files have been updated.
  110 files changed; no functional behaviour modified.

## [0.12.0] - 2026-06-07

Post-MVP expansion — two Future RFC items promoted to implemented:

### Added

**RFC-045: Markdown Preview Pane**

- A **Preview** button appears in the editor-actions bar when a section is
  focused. Clicking it (or pressing **Ctrl+Shift+P**) commits the pending
  draft and switches to a rendered HTML view of the section body.
- The rendered preview uses a manual pulldown-cmark event walker — no
  additional C build-script features required. Supported: headings H1–H6,
  paragraphs, bold, italic, strikethrough, inline code, fenced code blocks,
  unordered and ordered lists, block quotes, links, images, tables,
  task-list checkboxes, horizontal rules, hard/soft breaks, raw HTML pass-through.
- Preview is read-only. The source-preservation invariant is unaffected:
  `section_html` and `document_html` are pure functions over the outline.
- `PreviewPane` component with `role="region"` and accessible back-to-edit
  button. Scoped `.preview-body` CSS styles the rendered Markdown.
- 7 new tests in `crates/layered-core/src/preview.rs` covering bold, headings,
  empty body, unknown node, Japanese text, code fences, and HTML escaping.

**RFC-046: Document Statistics**

- The status bar now shows word count and section count for the open document.
  When a section is focused, the focused-section word count is shown first:
  `42 words / 1 234 words · 17 sections`.
- `layered_ui::stats` module with `DocumentStats`, `word_count`, and
  `compute_stats`. Statistics recompute only on committed edits (not per
  keystroke), following the render-boundary policy from RFC-033.
- 6 new tests covering zero, normal, leading/trailing whitespace, focused
  scope, section count, and post-edit update.

### Changed

- `AppSettings::remove_recent` and `clear_recent` marked `#[allow(dead_code)]`
  (reserved for future clear-recents UI).
- Workspace version bumped to 0.12.0.



## [0.11.0] - 2026-06-07

Phase G release (M8 — Cross-Platform Delivery, RFC-035..038; M9 — Production
Readiness, RFC-039..042). All 45 design RFCs are now implemented.

### Added

**Cross-platform delivery (M8):**

- **App settings with persistent recent files** (RFC-036): `AppSettings`
  stored as TOML in the OS config directory. Recent files (up to 10) are
  loaded on startup and shown on the welcome screen. Opening a file adds it
  to the list automatically; stale paths are filtered on load. Backed by
  `dirs`, `serde`, and `toml` workspace dependencies.
- **Recent files welcome screen** (RFC-036 / RFC-041): The welcome screen now
  shows a five-step onboarding guide and the recent-files list with file name
  and directory. Clicking an item opens the file immediately.
- **Platform documentation** (RFC-035): `PLATFORMS.md` documents the support
  matrix, required Linux system packages, keyboard modifier policy, file dialog
  backend, config directory paths, and known constraints per platform.
- **Packaging and release checklist** (RFC-037 / RFC-038 / RFC-042):
  `RELEASE_CHECKLIST.md` covers pre-release gates, data-integrity tests,
  per-platform smoke test workflow, artifact matrix with checksum instructions,
  unsigned build policy, and the required sign-off form.

**Production readiness (M9):**

- **Structured open-error dialog** (RFC-039): `OpenOutcome::Failed` now carries
  a plain-language `cause` string. File-open failures show an `ErrorDialog`
  modal with the specific reason (permission denied, not valid UTF-8, file not
  found) instead of a bare status-bar message. `ErrorDialog` is a reusable
  component for future error surfaces.
- **`open_markdown_path`** (RFC-039): opens a file at a known path without
  displaying a dialog, used by the recent-files list and testable in isolation.
- **Test strategy documentation** (RFC-040): `TESTING.md` formalises the test
  pyramid, fixture catalog, regression policy (reproduce → classify →
  fix → keep test), and CI requirements.
- **4 new regression tests** (RFC-040): empty document, whitespace-only
  document, edit-last-preserves-first, and UTF-8 multibyte body edit are now
  in `tests/source_preservation.rs`.
- **Known limitations page** (RFC-041): `docs/src/known-limitations.md`
  documents read-only raw source, setext promote/demote constraint, focus-return
  WebView limitation, deferred features, and what is explicitly not limited.
- **Release policy in README** (RFC-042): public releases require explicit
  product-owner sign-off; unsigned build verification instructions added.

### Changed

- `WelcomeScreen` gains `recent_files: Signal<Vec<String>>` and
  `on_open_recent: EventHandler<String>` props; all callers updated.
- Workspace version bumped to 0.11.0.
- `SUMMARY.md` updated with Known Limitations and Architecture pages.



## [0.10.0] - 2026-06-07

Sixth + seventh milestone release (M6 — Accessibility Hardening, RFC-027..030;
M7 — Performance and Large Document Readiness, RFC-031..034).

### Added

**Accessibility (M6):**

- **Semantic landmark regions** (RFC-027): toolbar rendered as `<header
  role="toolbar">`, the outline side-panel as `<aside>`, keeping `<main>` for
  the focus editor and `<footer>` for the status bar. Interactive elements
  across every component carry explicit accessible names.
- **Keyboard focus after zoom** (RFC-028): the body editor textarea now
  receives `autofocus` when a section is entered, so keyboard-only users land
  directly in the editor without extra Tab presses.
- **Polite vs assertive live regions** (RFC-029): the status bar now uses
  `aria-live="assertive"` for error keys (anything starting with `error.`) so
  screen readers interrupt to announce failures, while save confirmations and
  status updates remain `polite`. Save-failure status now includes an inline
  **Save As** recovery affordance rendered as a button (RFC-029 error pattern).
- **Light theme via `prefers-color-scheme: light`** (RFC-030): CSS custom
  properties remap the full token set to a light palette automatically.
- **Reduced-motion support** (RFC-030): `@media (prefers-reduced-motion:
  reduce)` disables all transitions and animations site-wide.
- **Enhanced focus ring** (RFC-030): `:focus-visible` rule now applies `!important`
  to ensure visibility overrides component-level styles. Focus ring is visible
  in both light and dark themes at sufficient contrast.
- `dirty-indicator` in toolbar now carries `aria-label` for screen readers.

**Performance and large-document readiness (M7):**

- **Three new test fixtures** (RFC-034): `large-10k-words.md` (~15 000 words,
  deterministically generated), `academic-paper.md` (realistic academic
  structure with nested sections and references), `technical-rfc.md` (RFC-style
  document with code fences and tables in body ranges). All fixtures are
  version-controlled and covered by golden tests.
- **Fixture catalog** (RFC-034): 11 new tests in
  `crates/layered-core/tests/fixture_catalog.rs` verify outline shape, heading
  count, round-trip byte-preservation, and source integrity across every
  fixture.
- **Criterion benchmarks** (RFC-031): `crates/layered-core/benches/indexing.rs`
  measures parse+index, section body replacement, promote, move, and split on
  small, medium, and large fixtures. Run with `cargo bench -p layered-core`.
- **Architecture documentation** (RFC-033): `docs/src/architecture.md` records
  the render boundary contract, state ownership table, re-index lifecycle, and
  anti-patterns. Added to `SUMMARY.md` alongside a new structural-editing
  user guide page.

### Changed

- `StatusBar` gains an `on_save_as: EventHandler<()>` prop for the inline
  recovery affordance; all callers updated.
- Workspace version bumped to 1.0.0.



## [0.9.0] - 2026-06-07

Fifth milestone release (M5 — Structural Editing, per RFCs 023–026).

### Added

- **Promote / Demote heading** (RFC-023): raise or lower a section's ATX
  heading level by one step (`#`→`##` or vice-versa). Guards reject H1
  promote, H6 demote, and Setext headings (with a clear message directing
  the user to convert via raw view). Only the heading marker bytes change;
  all body text, child sections, siblings, and unrelated bytes are preserved
  exactly.
- **Move section up / down** (RFC-024): swap a section with its previous or
  next sibling. The full subtree (`full_range` — heading + body +
  descendants) is extracted and reinserted as a single source-text operation.
  Cyclic moves (into own descendants) and self-moves are rejected with typed
  errors before any mutation.
- **Delete section** (RFC-025): removes a section's `full_range`. A
  confirmation dialog (RFC-026 guard) displays the title and child count
  before the user can proceed. Fully undoable via Ctrl+Z.
- **Add child section (split)** (RFC-025): a dialog collects the new
  section title; the heading is inserted at the end of the focused body,
  splitting off a new child. Undo restores the original body.
- **Merge up** (RFC-025): removes a section's heading line, making its body
  a continuation of the previous sibling's body. Undo is byte-exact.
- **Structural edit validation framework** (RFC-026): `StructuralEditError`
  enum centralises preflight rejections: `RevisionMismatch`, `StaleNode`,
  `InvalidLevel`, `CannotMoveIntoDescendant`, `CannotDeleteRoot`,
  `NoAdjacentSibling`, `UnsupportedHeadingStyle`, `InvalidSplitOffset`.
  Every structural op rolls back automatically on re-index failure, preserving
  the pre-edit source.
- `layered_core::structural` module exposed as `pub`; `MoveTarget` and
  `StructuralEditError` re-exported from `layered_ui`.
- 23 new golden tests in `tests/structural_ops.rs` covering every operation,
  each error variant, undo round-trips, and byte-preservation invariants.
- 20 new i18n keys for structural ops, dialogs, and error messages (en + ja).



## [0.8.0] - 2026-06-07

Fourth milestone release (M4 — Navigation and Search, per RFCs 019–022).

### Added

- **Sibling and depth navigation** (RFC-020): Parent / First Child / Prev /
  Next buttons appear in the focus editor beneath the breadcrumb and title.
  Clicking any of them commits the pending draft first, then navigates.
  `EditorSession` exposes `navigate_parent`, `navigate_first_child`,
  `navigate_prev_sibling`, `navigate_next_sibling` and `sibling_info`.
- **Whole-document and section-scoped search** (RFC-021): `Ctrl+F` opens a
  slide-in search panel. Query is case-insensitive and UTF-8-safe; results are
  grouped by section path with a preview snippet. Selecting a result focuses
  the containing section and closes the panel. `layered_ui::search` module
  provides `search_document` / `search_section` as pure functions over
  `Document`.
- **Command palette** (RFC-022): `Ctrl+P` opens a filterable command list
  drawn from the static `layered_ui::commands::COMMANDS` registry. Each entry
  shows the command title and default shortcut. Selecting a command executes
  the corresponding app action. `filter_commands` is testable with a mock
  localizer.
- **Focus history with stale-node reporting** (RFC-019): `Alt+←` / `Alt+→`
  back/forward history was already implemented; this release adds non-blocking
  status feedback (`nav.stale_section`) when a history target no longer exists
  after a document edit. `EditorSession::prune_and_report` returns `true` when
  pruning occurred so the UI can surface the message.
- `Esc` now also dismisses the search panel and command palette before
  triggering zoom-out.
- `layered_ui::navigation` module with `SiblingInfo` and `sibling_info()`.
- `layered_ui::search` module with `SearchMatch`, `search_document`,
  `search_section` and 5 tests including a UTF-8 range-validity check.
- `layered_ui::commands` module with `CommandSpec`, `COMMANDS` and
  `filter_commands`; 3 unit tests.
- 15 new i18n keys in both English and Japanese catalogs (search, palette,
  navigation labels, stale-node message).

### Changed

- Workspace version bumped to 0.8.0.
- `app.rs` wired with search/palette overlay signals; all signal mutations
  continue to follow the `let mut sig = sig` shadowing pattern required by
  Dioxus 0.6 `Writable::set(&mut self)`.

## [0.2.0] - 2026-06-07

Second milestone release (M2 — Basic Desktop UX, per RFCs 010–014).

### Added

- **Desktop application shell** (RFC-010): welcome screen for new sessions,
  dirty indicator `●` in the toolbar, Save As button, document name display,
  and status bar covering ready/saved/unsaved/error states.
- **Outline and overview UI** (RFC-011): heading cards in the main canvas
  show level badges (H1…H6), child counts, and keyboard-selected highlight.
  Arrow-key navigation + Enter to zoom into any visible card; empty/headingless
  document state with hint text.
- **Focus editor** (RFC-012): breadcrumb header, section title with level
  label, textarea with `aria-label`, local-dirty indicator `●`, commit on
  blur and on the Edit button. Failed commits keep the draft text so the user
  can see and recover their unsaved work.
- **Breadcrumb navigation** (RFC-013): `<nav aria-label>` with `aria-current`
  on the current segment; long paths collapse to root › … › parent › current;
  clicking any ancestor navigates and commits pending draft.
- **Keyboard interaction** (RFC-014): Ctrl/Cmd+O/S/Shift+S, Ctrl/Cmd+Z/Y,
  Alt+←/→, Esc (commit + zoom out), Enter (zoom in from overview), ↑/↓
  (card selection) — all wired through a pure `interpret()` function and a
  global `onkeydown` handler on the app div.
- **Outline side panel** (RFC-011): left-panel `<nav role="listbox">` with
  roving tabindex; keyboard Enter/Space to zoom; Up one level button when
  focused; keyboard hint for sighted users.
- Keyboard shortcut reference page added to the mdBook user guide.

### Changed

- `app.rs` refactored to `use_callback` pattern (Dioxus 0.6 `Writable` trait
  uses `&mut self`; `let mut sig = sig` shadowing inside callbacks makes
  closures `Fn` and fully `Copy`-shareable).
- `file_dialog.rs` handles open/save I/O; `keyboard.rs` is pure and
  dependency-free from editor state.

## [0.7.0] - 2026-06-07

First milestone release (M0 "Core Document Engine" + M1 "Layered Editing MVP"
foundations, per the roadmap and RFCs 001–009, 043, 044).

### Added

- `layered-core`: canonical-text document model with derived outline index
  over `pulldown-cmark` (ATX + Setext headings, code fences and YAML/TOML
  front matter excluded), ordinal-path `NodeId`s stable across body edits,
  byte-exact section-body replacement with optimistic revision checking,
  and bounded byte-exact undo/redo.
- Golden integration suite: 13 fixture documents (Japanese text, CRLF,
  duplicate titles, skipped heading levels, HTML blocks, front matter,
  missing trailing newline, …) verified for source preservation and
  undo/redo round-trips on every section.
- `layered-ui`: `EditorSession` facade (content-based dirty tracking,
  focused-body commits, dead-focus pruning after structural edits),
  browser-style focus navigation history, and i18n catalogs (English,
  Japanese) with graceful fallback.
- `layered-desktop`: Dioxus desktop shell — outline pane, focus editor with
  breadcrumbs and subsection cards, undo/redo/back/forward toolbar, open/save
  dialogs, runtime language switching.
- Project documentation: README, mdBook user guide skeleton, 44 RFCs under
  the lifecycle policy.
## [0.3.0] - 2026-06-07

Third milestone release (M3 — File Lifecycle and Recovery, per RFCs 015–018).

### Added

- **Raw Markdown source view** (RFC-017): Ctrl+` toggles a read-only overlay
  displaying the exact canonical text with line count. Back to Structure returns
  to the previous outline or focus mode. Status bar shows a "Raw Markdown
  Source" badge when active.
- **Unsaved changes guard** (RFC-016): opening or creating a new document while
  the current document is dirty shows a three-button dialog (Save / Discard /
  Cancel). Save commits pending draft, saves to disk, then proceeds only on
  success.
- **External modification detection** (RFC-015): when saving, if the file on
  disk has a newer mtime than when it was last written by layered, a dialog
  offers Overwrite / Save As / Cancel before touching the disk.
- **Atomic save** (RFC-015, NFR-REL-003): saves write through a temp file
  then rename, so a crash mid-write cannot corrupt the original.
- **UTF-8 BOM preservation** (RFC-018): files with a UTF-8 BOM are opened with
  the BOM stripped internally; it is re-prepended on save.
- **Line ending detection** (RFC-018): `FileTextProfile` detects LF / CRLF /
  Mixed at open time; the status bar shows the policy label.
- **`EditorSession::open_with_profile`**: desktop crate passes pre-detected
  profile on open rather than re-running detection in the session.
- `layered_ui::file_profile` module exported as public API.
- Keyboard reference page updated with Ctrl+` shortcut.
