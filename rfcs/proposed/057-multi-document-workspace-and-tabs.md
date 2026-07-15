# RFC-057: Multi-Document Workspace and Tabs

**Project:** omriss — Omriss Editor
**Milestone:** M13 — Multi-Document Workspace (proposed)
**Status.** Proposed
**Revision:** v2 (incorporates architect and dev-team review)
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-001, RFC-014, RFC-015, RFC-016, RFC-036
**Integrates with (not build prerequisites):** RFC-050, RFC-053
**Related RFCs:** RFC-022, RFC-048

---

## 0. Revision history

- **v1** — Initial proposal: workspace layer, tab strip, per-tab vs app-global
  split, batched quit guard, single `Signal<Workspace>`.
- **v2** — Incorporates the architect review and two dev-team review rounds.
  Twelve changes folded in (see § 0.1). The direction is unchanged; the
  corrections protect existing omriss invariants (focused-typing
  responsiveness, source-preserving edits, single-document UX) and make the
  structured-format future correct without blocking the Markdown-first build.

### 0.1 What changed in v2

1. Invalid structured drafts **block** tab switching (no commit, no revert).
2. RFC-057 owns a **minimal draft-validity seam**; RFC-050/RFC-053 are
   integration points, **not build prerequisites** (§ 1.1, § 6.5).
3. Save All **validates only the active draft**, then saves dirty tabs
   (consequence of blocked switching) (§ 8.6).
4. Hot draft text must **not** write one large `Signal<Workspace>` on every
   keystroke (§ 6.6).
5. Stable **`TabId`**; modal/async actions target `TabId`, not index (§ 6.3).
6. Tab identity uses **`DocumentIdentity`**, not `FileIdentity`; only saved
   paths dedupe, untitled documents never dedupe (§ 6.7).
7. **`preview_open` is per-tab** (§ 6.1).
8. **Search** state is closed/reset on tab switch; stale results are never
   shown (§ 6.2, § 9.6).
9. **Status** messages are active-tab scoped and cleared on switch (§ 6.2).
10. **Shortcuts** are verified against RFC-014/RFC-022 and the actual
    `keyboard::interpret` map before landing (§ 10, Appendix A).
11. **clippy** is *recommended*, not a new hard gate, unless RFC-040/CI policy
    is updated (§ 15.3).
12. Friendlier dialog labels: **"Close without saving" / "Quit without
    saving"**, not "Discard" (§ 8).

## 1. Summary

This RFC proposes a workspace layer that lets omriss hold more than one open
document at a time, surfaced to the user as a tab strip with one tab per open
document.

Today omriss is strictly single-document. The application shell owns exactly
one `EditorSession`, and all per-document signals (`session`, `draft`,
`saved_mtime`, `selected_card`) plus the shell-global `preview_open` and
`search_open` live directly on the root component. This RFC introduces a
`Workspace` that owns an ordered collection of `Tab`s, each wrapping one
`EditorSession` and its associated transient view state, plus a stable active
`TabId`. The application-global concerns (locale, modal host, command palette,
recent files, settings) remain singular.

The central design claim is that **the document boundary is already clean
enough to multiply**. `EditorSession` is fully self-contained — it owns its
`Document`, view state, dirty flag, file name, and format profile, and assumes
no global singleton state. Multiplying it does not touch the `omriss-core`
crate at all and touches `omriss-ui` only to add a thin `Workspace`/`Tab`
container. The work is almost entirely in the app shell.

This RFC defines the data model, the per-tab vs app-global state split, the
draft-validity seam, the tab-bar visibility rule, the cross-tab unsaved-changes
lifecycle, keyboard and accessibility contracts, and the acceptance criteria.
It does **not** propose split views, tab groups, tab tear-off into separate
windows, drag-reorder, or session persistence across restarts; those are
explicit non-goals.

### 1.1 Dependency posture (build vs integration)

RFC-057 depends on the existing focused-edit lifecycle and the
dirty-state/file lifecycle behavior (RFC-014, RFC-015, RFC-016, RFC-036). It
**does not require** the JSON/TOML/YAML adapters (RFC-053) or structured-value
editing (RFC-050) to be implemented.

RFC-057 owns the minimal draft-validity seam needed by tab switching:

```text
- Markdown drafts are always valid as section-body text.
- The tab-switch path asks the active editor whether the current draft can be
  applied.
- If valid, the draft is applied and tab switching continues.
- If invalid, tab switching is blocked and focus remains on the active editor.

Until structured-format editing lands, the invalid-draft branch is mostly
dormant because Markdown body drafts are always valid. RFC-050 and RFC-053
later plug structured validation into the same seam without changing the
workspace logic.
```

This lets tabs ship for the Markdown-first product after the UI-role split
(M10) is stable, independent of when structured formats (M11/M12) land.

## 2. Motivation

Users working on structured documents routinely need more than one file open:
a specification and its changelog, a config file and the doc that describes it,
two chapters being cross-referenced. Today omriss forces a full
open/unsaved-guard/replace cycle to move between them, which loses focus
position, draft state, and undo history for the document being left behind.

The current single-document model was the right starting point — it kept the
core engine, the file lifecycle, and the dirty-state guard simple, and it let
the "think by layers, edit by focus" experience stay uncluttered. But the cost
of switching documents is now the main friction point for multi-file work, and
the architecture is already shaped to remove it cheaply.

A tab per document is the most conventional, lowest-surprise way to expose
multiple open documents. It also composes naturally with the RFC-048 role
split (Document Map left, Focused Content right): tabs sit above both panels
and switch which document those panels are bound to.

## 3. Design principles

### 3.1 The document boundary is the tab boundary

One tab holds exactly one `EditorSession`. There is no shared mutable state
between tabs except the application-global singletons named in § 6.2. Switching
tabs swaps which session the panels read; it does not copy or merge document
state.

### 3.2 Single-document experience must stay pristine

With one document open, the application must look and behave as it does today.
The tab strip is suppressed entirely when only one document is open (§ 7.3).
"Less is more" governs here: tabs are chrome, and chrome that is always present
taxes the common single-document case for no benefit.

### 3.3 Per-tab state is owned by the tab; app state is owned by the workspace

The split is explicit and total (§ 6). Every signal currently on the root
component is classified as either per-tab (moves into `Tab`) or app-global
(stays at `Workspace`/`App`). No signal is ambiguous.

### 3.4 Invalid drafts cannot be escaped by navigation

A focused draft that cannot be safely applied blocks navigation — including
tab switching, which is navigation. The draft is neither silently committed
nor silently reverted; the user stays on the field with plain guidance. For
Markdown this never triggers (body text is always valid); for future
structured formats it is the safety guarantee (§ 6.5, § 8.4).

### 3.5 Closing a document is a guarded action

A document with unsaved changes cannot be silently discarded by closing its
tab or quitting. The unsaved-changes guard (RFC-016) is extended to be
tab-aware, not replaced. Switching tabs is *not* a discard and does not invoke
the guard (§ 8.2).

### 3.6 Focused typing stays cheap

Typing in the active editor must not re-render the tab strip or inactive tab
state. The workspace signal carries tab metadata; hot draft text lives in a
focused local signal and is applied to the tab only at commit boundaries
(§ 6.6). This preserves RFC-033's render-boundary policy.

### 3.7 The core crate does not learn about tabs

`omriss-core` stays document-scoped. The `Workspace` type lives in `omriss-ui`. The
app shell renders the tab strip. RFC-001 crate boundaries are unchanged: no
Dioxus in core, no multi-document concept in core.

## 4. Terminology

```text
Document         A single open file or new buffer, modeled by EditorSession.
Tab              One slot in the workspace holding one Document plus its
                 transient view state (draft buffer, saved mtime, selected
                 card, preview state).
TabId            A stable, process-unique identifier for a Tab. Survives the
                 close/open of other tabs. Used for modal and async targeting.
Workspace        The ordered collection of Tabs plus the active TabId and the
                 app-global singletons.
Active tab       The one Tab currently bound to the Document Map and Focused
                 Content panels.
Tab strip        The horizontal row of tab headers, shown only when more than
                 one Tab is open.
Document identity The dedupe key for "already open": a normalized saved path,
                 or Untitled (never deduped).
Draft-validity seam The trait the active editor implements to answer "can this
                 draft be applied?" — always Yes for Markdown body text.
```

## 5. User-facing behavior

### 5.1 Opening documents

```text
Open File (a document is already open, no conflict)
  -> a new Tab is appended
  -> the new Tab becomes active
  -> the tab strip appears (now two or more tabs)
```

Opening a file already open in another tab activates that existing tab rather
than opening a duplicate (§ 9.4), keyed on document identity (§ 6.7).

### 5.2 Creating documents

```text
New
  -> a new blank (Untitled) Tab is appended and becomes active
  -> Document Map shows the empty state; "+ Add section" is ready
  -> no dialog opens automatically
```

### 5.3 Switching tabs

```text
Click a tab header     -> that Tab becomes active
Ctrl+Tab / Ctrl+Shift+Tab -> next / previous Tab
Ctrl+1 .. Ctrl+9       -> activate Tab by position (9 = last)
```

Switching first asks the active editor whether its pending draft can be applied
(§ 6.5). If valid, the draft is applied and the panels rebind to the incoming
tab. If invalid (only possible for structured formats), the switch is blocked,
the active tab stays active, focus stays on the invalid field, and guidance is
shown. No document loses edits on a valid switch.

### 5.4 Closing tabs

```text
Close tab (clean document)
  -> Tab is removed
  -> if it was active, the tab to its right becomes active
     (or the tab to its left if it was the rightmost)
  -> if it was the last tab, the workspace returns to the welcome screen

Close tab (document with unsaved changes)
  -> the unsaved-changes dialog appears, scoped to that document
  -> Save / Close without saving / Cancel; Cancel keeps the tab open
```

`Ctrl+W` / `Cmd+W` closes the active tab through the same guard.

### 5.5 Tab header contents

Each tab header shows the document's display name (file name, or a localized
"Untitled N" for a new buffer) and a dirty marker (the same `●` used elsewhere)
when the document has unsaved changes. A close affordance (×) appears on hover
and is keyboard-reachable.

### 5.6 Dirty indication

The dirty marker appears both in the tab header and, for the active tab, in the
existing status bar. A user scanning the tab strip can see at a glance which
documents have unsaved work. Dirty state is conveyed in the tab's accessible
name in words, not by the `●` glyph or color alone (§ 10).

## 6. State model

### 6.1 Per-tab state (moves into `Tab`)

These are document-scoped and travel with the tab:

```text
session       EditorSession            document, view/focus state, dirty flag,
                                        file name, format profile, undo/redo
saved_mtime   Option<SystemTime>       external-modification baseline
selected_card usize                    overview-pane selection index
preview_open  bool                     per-tab preview/edit state (v2 change #7)
search_state  TabSearchState           per-tab search (see § 6.2; v1 closed
                                        on switch, may persist per-tab later)
```

Focus/view mode is already inside `EditorSession.view`, so it travels with the
session automatically. `preview_open` moves out of the shell (where it is a
global `use_signal` today) into the tab, so switching to a document restores
whether it was last in preview.

The **hot draft buffer is not stored as per-tab signal state that updates on
every keystroke** — see § 6.6.

### 6.2 App-global state (stays at `Workspace`/`App`)

```text
locale        Signal<Locale>           UI language
modal         Signal<Modal>            the single active modal dialog
status        Signal<String>           status line; ACTIVE-TAB SCOPED (see below)
search_open   Signal<bool>             search panel visibility (see below)
palette_open  Signal<bool>             command palette visibility
recent_files  Signal<Vec<String>>      recent-files list (settings-backed)
```

**Status (v2 change #9).** `status` remains a single host but always reflects
the active tab. On tab switch, transient success/error messages from the
previous tab are cleared (or replaced by the incoming tab's neutral status) so
a stale "Could not save" never lingers on a different document. Save results,
validation messages, and external-conflict notices are active-tab scoped;
genuinely global notices (language changed, settings saved) are rare and
transient.

**Search (v2 change #8).** Visibility may be app-global, but query and results
belong to a document. For v1, **switching tabs closes the search panel and
clears results** so stale results from another document are never shown as
current. Per-tab search persistence is a deferred refinement (§ 13.5).

### 6.3 The `Workspace` and `Tab` types (omriss-ui)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabId(u64);

pub struct Tab {
    pub id: TabId,
    pub session: EditorSession,
    pub saved_mtime: Option<SystemTime>,
    pub selected_card: usize,
    pub preview_open: bool,
    pub search_state: TabSearchState,
}

pub struct Workspace {
    tabs: Vec<Tab>,
    active: Option<TabId>, // None only when tabs is empty
    next_tab_id: u64,      // monotonic; never reused within a process
}
```

`TabId` is **stable** for the life of the process and is never reused, so a
modal or async action that captured a `TabId` always refers to the same
document (or resolves to "gone" if that tab was closed). Indexes are used only
as transient lookup results, never stored in modal state. (v2 change #5.)

The workspace API is total and id-first:

```rust
impl Workspace {
    pub fn new() -> Self;

    pub fn open_tab(&mut self, session: EditorSession) -> TabId; // appends + activates
    pub fn new_untitled_tab(&mut self, session: EditorSession) -> TabId;

    pub fn activate(&mut self, id: TabId) -> Result<(), WorkspaceError>;
    pub fn activate_index(&mut self, index: usize) -> Result<(), WorkspaceError>; // Ctrl+1..9
    pub fn close_tab(&mut self, id: TabId) -> Result<Tab, WorkspaceError>;

    pub fn active_id(&self) -> Option<TabId>;
    pub fn active(&self) -> Option<&Tab>;
    pub fn active_mut(&mut self) -> Option<&mut Tab>;

    pub fn tab(&self, id: TabId) -> Option<&Tab>;
    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab>;
    pub fn tabs(&self) -> &[Tab];
    pub fn index_of(&self, id: TabId) -> Option<usize>;

    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn any_dirty(&self) -> bool;
    pub fn dirty_tabs(&self) -> Vec<TabId>; // tab order

    pub fn find_by_identity(&self, identity: &DocumentIdentity) -> Option<TabId>;
}
```

The active invariant — `tabs.is_empty() || active.is_some_and(|id| present)` —
is enforced inside every mutator. `close_tab` reassigns `active` to the right
neighbor (else left), or `None` when the last tab closes.

### 6.4 Errors

```rust
pub enum WorkspaceError {
    NoSuchTab(TabId),
    IndexOutOfRange(usize),
}
```

`activate`/`close_tab` on an unknown `TabId` return `NoSuchTab` rather than
panicking, so a stale modal action degrades safely.

### 6.5 The draft-validity seam (v2 change #2)

Tab switching, save, preview, and close all need one question answered about
the active editor: *can the current draft be applied to its document?* RFC-057
owns the minimal trait for this; it does not depend on structured formats
existing.

```rust
/// Implemented by the active focused editor. Markdown's implementation always
/// returns Applicable. Structured-format editors (RFC-050/RFC-053) return
/// Invalid with a reason when the focused value does not parse/validate.
pub enum DraftApplicability {
    /// The draft can be applied to the document now.
    Applicable,
    /// The draft cannot be applied; keep focus and show this guidance.
    Invalid { message_key: &'static str },
    /// Nothing pending; no-op.
    Clean,
}

pub trait FocusedDraft {
    fn applicability(&self) -> DraftApplicability;
    fn apply(&mut self) -> Result<(), EditError>; // only call when Applicable
}
```

Until structured editing lands, the only implementor is the Markdown body
editor, whose `applicability()` is `Applicable` (or `Clean`). The
invalid-draft branch is present and unit-testable at the seam (with a fake
validator) but is not user-exercised until RFC-054/RFC-055. (v2 change #2 and
the dormant-branch note, § 15.2.)

### 6.6 Signal granularity in the shell (v2 change #4)

The shell holds a single `Signal<Workspace>` for **tab metadata** — order,
active id, titles, dirty markers, paths, preview flags. It is written on
structural events only:

```text
open tab · new tab · close tab · activate tab · save result ·
external-modification baseline change · dirty-marker transition
```

The **hot draft text** is a focused local signal owned by the editor panel, as
it is today. It is applied into the active tab's session at commit boundaries
(blur, save, preview, tab switch) via the § 6.5 seam. Typing therefore mutates
only the local draft signal and never writes the workspace signal, so the tab
strip and inactive tabs do not re-render per keystroke. This satisfies
RFC-033's render-boundary policy and is the same local-draft model the
single-document app uses now.

The v1 RFC's blanket rejection of per-tab signals (old § 13.1) is **softened**:
one workspace signal for metadata is fine; what matters is that hot draft text
stays out of it. If profiling ever shows a metadata write is too coarse, a
per-tab handle is an acceptable future refinement.

### 6.7 Document identity and dedupe (v2 change #6)

Tabs represent **documents**, not always files. Dedupe uses document identity,
not raw path strings:

```rust
pub enum DocumentIdentity {
    /// A document backed by a saved file, keyed on a normalized path.
    SavedPath(NormalizedPath),
    /// A new buffer with no file. Never deduped against anything.
    Untitled(TabId),
}
```

`NormalizedPath` canonicalizes where possible (absolute path, symlink
resolution where safe, platform case-folding on Windows/macOS, Windows
path-prefix normalization). Rules:

- only `SavedPath` participates in already-open dedupe;
- untitled buffers are never deduped (each New is its own document);
- canonicalization failures fall back conservatively to the absolute
  normalized path rather than treating distinct files as the same;
- display path is preserved separately for the tab label; dedupe never uses
  the display string.

## 7. UI design

### 7.1 Layout

```text
+--------------------------------------------------------------------------------+
| omriss                         [Open] [Save] [Preview] [Show File Text] [More] |
+--------------------------------------------------------------------------------+
| ▣ spec.md ●  | changelog.md  | settings.toml  |                       [+]      |  <- tab strip
+-----------------------------+--------------------------------------------------+
| Document Map                | Focused Content                                  |
|  (bound to the active tab)  |  (bound to the active tab)                       |
+-----------------------------+--------------------------------------------------+
| Ready                                                                          |
+--------------------------------------------------------------------------------+
```

The tab strip sits between the header toolbar and the body. The body is
unchanged; it reads the active tab.

### 7.2 Tab header anatomy

```text
[ name ● × ]
   |   | └ close affordance (hover / keyboard)
   |   └ dirty marker (only when unsaved)
   └ document display name, ellipsized if long
```

### 7.3 Visibility rule

The tab strip renders only when `workspace.len() > 1`. Zero documents shows the
welcome screen; one document shows the editor with no tab strip, identical to
today. The optional `[+]` new-tab affordance exists only while the strip is
present; with one document, New is reached through the toolbar/command palette.

### 7.4 Overflow

When tabs exceed the available width, the strip scrolls horizontally rather
than shrinking headers below a legible minimum. An overflow menu is out of
scope; horizontal scroll is the v1 behavior.

## 8. Unsaved-changes lifecycle

The RFC-016 guard currently protects a single document at three moments: Open,
New, and quit. With tabs the guard becomes tab-aware.

### 8.1 Events that can discard document state

```text
A. Close one tab
B. Switch away from a tab     (does NOT discard — draft applied via § 6.5)
C. Open / New                 (appends a tab — never replaces)
D. Quit the application       (can discard ALL tabs)
```

### 8.2 Per-event policy

**A. Close one tab.** If dirty, show the unsaved-changes dialog scoped to that
document. Labels: **Save / Close without saving / Cancel**. Cancel aborts the
close. The modal carries the target `TabId`, not an index.

**B. Switch tabs.** Switching never shows the unsaved guard. The active draft
is applied via the § 6.5 seam first; a *valid* (or clean) draft applies and the
switch proceeds, an *invalid* draft blocks the switch (§ 8.4). Dirty state is
preserved on the tab and is not a reason to block a switch. Leaving a document
is non-destructive.

**C. Open / New.** These append a tab instead of replacing the current
document, so the old `UnsavedBeforeOpen` / `UnsavedBeforeNew` guards are
**removed**. Opening or creating never threatens existing work.

**D. Quit.** Quit guards every dirty tab:

```text
On quit request:
  if workspace.any_dirty():
    show quit-review dialog
    labels: Save All / Quit without saving / Cancel
  else:
    quit immediately
```

### 8.3 Why batched quit, not sequential

A sequential "save this one? save the next?" chain is tedious and gets
click-through without reading. A single batched **Save All / Quit without
saving / Cancel** is faster and clearer. A per-document review list is a future
refinement, not v1.

### 8.4 Invalid-draft blocking (v2 change #1)

Tab switching, save, preview, and save-and-close all route through the § 6.5
seam. When the active draft's `applicability()` is `Invalid`:

```text
- do not switch / save / preview / close
- keep the current tab active
- keep focus on the invalid field
- show the message_key guidance, e.g. "Fix this value before moving to
  another tab." / "...before saving."
- do not mutate canonical source text
```

For Markdown this branch never fires. For structured formats it is the
guarantee that an invalid value cannot be escaped by navigating away.

### 8.5 Modal scoping

`Modal` stays app-global (one dialog at a time). Close-tab and quit dialogs
carry the `TabId` (or the dirty `TabId` set) they act on, resolved at action
time, so a tab closing or opening between dialog-open and confirm never targets
the wrong document.

### 8.6 Save All (v2 change #3, #10)

Because switching is blocked on invalid drafts (§ 8.4), **inactive tabs cannot
hold an invalid uncommitted draft** under normal operation — you could never
have switched away from one. Therefore Save All validates only the **active**
tab's pending draft, not every tab:

```text
1. Apply/validate the active tab's pending draft via § 6.5.
   - if Invalid: block Save All, keep the active tab, focus the field,
     show "Fix this value before saving all documents."
2. Save each dirty tab that has a saved path, in tab order.
3. For each dirty Untitled tab, run Save As (sequentially).
   - if the user cancels a Save As: stop; quit is cancelled; already-saved
     tabs stay saved.
4. If a dirty tab's file changed on disk: stop, activate that tab, show the
   existing external-modification dialog scoped to it.
5. If a save fails: stop immediately, activate the failed tab, show a friendly
   message; the app stays open; tabs saved before the failure remain saved.
```

Recommended failure message: "Could not save “settings.toml”. Your other saved
files were kept."

This is deterministic (active first, then tab order), never silently loses a
draft, and never silently overwrites a conflicted file.

## 9. Interaction patterns

### 9.1 Open appends and activates

```text
Open -> file picked
  -> EditorSession::open_with_profile(...)
  -> identity = DocumentIdentity::SavedPath(normalized)
  -> if find_by_identity(identity) = Some(id): activate id (no duplicate)
  -> else: workspace.open_tab(session) (appends + activates)
  -> recent-files list updated (RFC-036, unchanged)
```

### 9.2 New appends and activates

```text
New
  -> workspace.new_untitled_tab(EditorSession::new_document())
  -> new tab active; empty Document Map state; no dialog
```

### 9.3 Switch applies-or-blocks, then binds

```text
Activate tab N
  -> ask active editor: applicability()
     - Invalid -> block (§ 8.4); active tab unchanged
     - Applicable -> apply()
     - Clean -> proceed
  -> workspace.activate(target_id)
  -> panels read the new active tab (incl. its preview_open, search closed)
```

### 9.4 Open-already-open dedupe

```text
Open file whose normalized identity matches an open tab
  -> activate the existing tab; do not duplicate
  -> if the on-disk file changed since it was opened, the RFC-015
     external-modification check applies on that tab
```

### 9.5 Close adjusts active pointer

```text
Close tab id
  -> if dirty: guard (§ 8.2 A); Cancel aborts
  -> workspace.close_tab(id)
  -> active -> right neighbor, else left neighbor
  -> if no tabs remain: welcome screen
```

### 9.6 Tab switch resets search

```text
On any successful tab activation:
  -> close the search panel and clear results (§ 6.2)
  -> bind status to the new active tab; clear stale transient messages
  -> restore the new tab's preview_open state
```

## 10. Keyboard shortcuts and conflict check (v2 change #10)

Proposed shortcuts:

```text
Ctrl+Tab / Ctrl+Shift+Tab        Next / previous tab
Ctrl+1 .. Ctrl+9                  Activate tab by position (9 = last)
Ctrl+W / Cmd+W                    Close current tab (through the guard)
```

Open/New/Save/Save As keep their existing bindings (Ctrl/Cmd + O/N/S/Shift+S)
and now act on the active tab.

**Conflict check against the current `keyboard::interpret` map (RFC-014) and
the command palette (RFC-022):** the existing map binds O, S, Shift+S, Z, Y,
Shift+Z, Alt+Left/Right, Escape, Enter, Up, Down, Backquote, F (search), P
(palette), and Shift+P (preview). The new tab shortcuts use `KeyW`, `Tab`, and
the digit codes, **none of which are currently mapped** — so there is no
collision with existing bindings. `Ctrl+N` (New) is likewise currently
unmapped and is safe to add. This must be re-verified against the code at
implementation time, but as of this revision the map is clear (see Appendix A).

## 11. Accessibility requirements

- The tab strip is a labeled tab list: `role="tablist"`, each header
  `role="tab"` with `aria-selected`, panels associated via `aria-controls` /
  `aria-labelledby`, so AT announces "tab 2 of 4, changelog.md, unsaved
  changes, selected."
- Tab activation is fully keyboard-operable: arrow keys move between headers,
  Enter/Space activates, and the global Ctrl+Tab / Ctrl+1..9 shortcuts work
  from anywhere in the app.
- The close affordance is keyboard-reachable (e.g. Delete on a focused header,
  or a focusable × button), not hover-only.
- The dirty marker is not conveyed by color or `●` alone; the accessible name
  includes a localized "unsaved changes".
- Focus moves predictably on close: after closing the active tab, focus lands
  on the newly active tab's header (or the welcome screen's primary action if
  none remain).
- Reduced-motion preferences are honored for any tab-switch transition.

## 12. Internationalization (RFC-043)

New keys (English + Japanese, sorted, no untranslated fallback):

```text
workspace.tab.untitled
workspace.tab.untitled_numbered
workspace.tab.close
workspace.tab.unsaved_suffix
workspace.tab.new
workspace.action.next_tab
workspace.action.previous_tab
workspace.action.close_current_tab
workspace.close.review_title
workspace.close.review_message
workspace.close.save
workspace.close.close_without_saving
workspace.close.cancel
workspace.quit.review_title
workspace.quit.review_message_one
workspace.quit.review_message_many
workspace.quit.save_all
workspace.quit.quit_without_saving
workspace.quit.cancel
workspace.switch.invalid_draft
workspace.save_all.invalid_draft
workspace.save_all.failed
workspace.open.already_open
```

Dialog wording is plain and non-technical; no UI uses "Discard" where "Close
without saving" / "Quit without saving" is clearer (v2 change #12).

## 13. Alternatives considered

### 13.1 One workspace signal including hot draft text

Rejected. Writing the whole workspace signal on every keystroke would
re-render the tab strip and inactive tabs, violating RFC-033. The chosen model
keeps hot draft text in a focused local signal (§ 6.6).

### 13.2 One signal per tab for everything

Not required for v1. The editor only renders the active tab, so per-tab signals
add lifecycle bookkeeping (registry, cleanup on close) with no rendering
benefit beyond what § 6.6 already achieves. Left as an acceptable future
refinement if profiling demands it.

### 13.3 Index-based active pointer and modal targeting

Rejected. Indexes are fragile across close/open between dialog-open and
confirm. Stable `TabId` removes the stale-target footgun (§ 6.3).

### 13.4 Switch-with-revert for invalid drafts

Rejected. Reverting an invalid draft on switch feels like data loss and
complicates Save All. Blocking is safer, clearer, and consistent with omriss's
source-preserving, forgiving design (§ 8.4).

### 13.5 Multiple windows / per-tab search persistence / session restore

Deferred, not rejected. Tear-off windows multiply platform-integration
surface; per-tab search persistence and session restore each add state and
staleness questions. Each is a candidate follow-up RFC. v1 keeps one window,
closes search on switch, and does not persist tabs across restarts.

## 14. Migration and compatibility

- The single `Signal<EditorSession>` on `App` is replaced by a single
  `Signal<Workspace>`. `AppCtx`'s `session`, `draft`, `saved_mtime`, and
  `selected_card` are accessed through `workspace.active()`. App-global fields
  are unchanged.
- `preview_open` moves from a shell `use_signal` into `Tab` (§ 6.1).
- `search_open` stays a shell signal but is closed/cleared on tab switch
  (§ 6.2, § 9.6).
- `handle_open_guarded` / `handle_new_guarded` lose their unsaved-guard branch
  (§ 8.2 C). `handle_open` / `handle_new` append a tab. The
  `UnsavedBeforeOpen` / `UnsavedBeforeNew` `Modal` variants are removed; the
  `UnsavedDialog` component is retained and reused for close-tab and quit.
- Save, Save As, external-modification check (RFC-015), and dirty state
  (RFC-016) are unchanged at the document level; they now operate on the active
  tab's session.
- No core (`omriss-core`) change. Document Map / Focused Content panels read the
  active tab; their props are unchanged if the shell passes the active tab's
  signals through.
- Existing single-document tests pass unchanged: a one-tab workspace behaves
  identically.

Sequenced as the PR plan in the developer handoff: introduce `Workspace`/`Tab`/
`TabId` with unit tests; add `DocumentIdentity` + dedupe; rewire `App`/`AppCtx`
to one workspace signal (single-document behavior preserved); add the tab strip
component; add the tab-aware close guard; add the quit guard + Save All; add
shortcuts/i18n/search+status cleanup; cross-platform QA.

## 15. Acceptance criteria

### 15.1 Workspace, identity, and lifecycle

- A `Workspace`/`Tab`/`TabId` model in `omriss-ui` owns an ordered `Vec<Tab>`
  and an active `TabId`, with the active invariant enforced in every mutator
  and covered by unit tests (open, close-first/middle/active/last,
  activate/close unknown `TabId`, dedupe-by-identity, untitled-never-deduped,
  any-dirty, stable `TabId` after earlier close).
- Opening or creating a document appends a tab and activates it; it never
  replaces the current document and never shows an unsaved-changes guard.
- Opening a file whose normalized identity matches an open tab activates the
  existing tab instead of duplicating it; untitled tabs are never deduped.
- Tab-close and quit modals target stable `TabId` values, not positional
  indexes.

### 15.2 Drafts, switching, and Save All

- Switching tabs applies a valid/clean active draft via the § 6.5 seam.
- Switching tabs with an **invalid** active draft is blocked: the active tab
  is unchanged, focus stays on the field, and no source text is mutated.
  *(Seam-level test now with a fake validator; full user-flow coverage lands
  with RFC-054/RFC-055 when structured editors can produce invalid drafts —
  the dormant-branch note.)*
- Save All applies/validates only the **active** tab's draft, then saves dirty
  tabs in order; it handles untitled (Save As), external-modification
  conflicts, and partial save failure deterministically without losing drafts
  or silently overwriting conflicted files.

### 15.3 Reactivity, UI, accessibility, i18n, gates

- Focused typing does **not** re-render the tab strip or inactive tab state on
  every keystroke (render-boundary test, RFC-033).
- The tab strip is hidden at one document; the single-document experience is
  visually unchanged from the prior release.
- `preview_open` is per-tab; switching restores each tab's preview/edit state.
- The search panel closes/clears on tab switch; stale results are never shown.
- Status reflects only the active tab; stale messages clear on switch.
- The tab list meets the § 11 accessibility contract (tablist roles, keyboard
  activation, non-color dirty cue, predictable focus on close).
- `Ctrl+W`/`Cmd+W` closes the active tab through the normal guard; new tab
  shortcuts are verified collision-free against `keyboard::interpret` (§ 10).
- All new strings exist in both catalogs, sorted, no untranslated fallback.
- No change to the `omriss-core` crate; RFC-001 boundaries hold.
- **Gates:** the required gates are the existing project gates —
  `cargo fmt --check`, `cargo test`, and `scripts/check-rfcs.sh`. `cargo
  clippy` is **recommended**, not required, unless and until RFC-040/CI policy
  makes it mandatory (v2 change #11).

## 16. Open questions

1. New-buffer naming: "Untitled", "Untitled 2", … per session (numbered to
   disambiguate multiple new buffers in the strip). Leaning numbered.
2. Should the tab strip ever be shown at exactly one tab via a preference?
   Default hidden at one tab (§ 7.3); an always-on setting is possible later.
3. Maximum open tabs — a cap, or is horizontal scroll (§ 7.4) sufficient?
   Leaning no hard cap for v1.

Resolved since v1: invalid-draft handling (block — § 8.4), `preview_open`
scope (per-tab — § 6.1), quit-guard shape (batched — § 8.3), signal
granularity (metadata-only workspace signal — § 6.6), tab identity (stable
`TabId` — § 6.3), dedupe key (`DocumentIdentity` — § 6.7), search-on-switch
(close — § 6.2), Save All scope (active-only — § 8.6), dependency posture
(RFC-050/053 integration not build — § 1.1).

## 17. Final decision summary

omriss gains a `Workspace` layer above `EditorSession` holding one `Tab` per
open document, surfaced as a tab strip shown only when more than one document
is open. Per-document state — including preview — moves into `Tab`, keyed by a
stable `TabId`; application-global state stays singular. RFC-057 owns a minimal
draft-validity seam (Markdown always valid), making RFC-050/RFC-053 integration
points rather than build prerequisites. Tab switching applies a valid draft and
**blocks** an invalid one; because of that, Save All only validates the active
draft. Hot draft text stays in a focused local signal so typing never re-renders
chrome. Opening and creating become non-destructive (append a tab), removing the
Open/New guards; the unsaved guard is retained tab-aware for close and batched
for quit. The core crate is untouched and the single-document experience is
preserved exactly.

This RFC is **Proposed** (v2) and ready for final pre-implementation review as a
package with the updated developer handoff.

## Appendix A — Current keyboard map (verified at v2 authoring)

From `crates/app/src/input/keyboard.rs`, `interpret()` currently maps:

```text
Ctrl/Cmd+O          Open
Ctrl/Cmd+S          Save
Ctrl/Cmd+Shift+S    Save As
Ctrl/Cmd+Z          Undo
Ctrl/Cmd+Y          Redo
Ctrl/Cmd+Shift+Z    Redo
Alt+Left / Right    Back / Forward
Escape / Enter      Escape / Enter
ArrowUp / ArrowDown SelectUp / SelectDown
Ctrl/Cmd+`          ToggleRaw
Ctrl/Cmd+F          OpenSearch
Ctrl/Cmd+P          OpenPalette
Ctrl/Cmd+Shift+P    TogglePreview
```

The tab shortcuts (`Ctrl+W`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+1..9`) and
`Ctrl+N` use codes not present above, so they are collision-free. Re-verify
against the code at implementation time.
