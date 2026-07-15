# RFC-057 (v2) Acceptance / QA Checklist

**Project:** omriss — Omriss Editor
**RFC:** RFC-057 — Multi-Document Workspace and Tabs (v2)
**Document type:** Acceptance and QA checklist
**Related RFC:** RFC-057 (`../../proposed/057-multi-document-workspace-and-tabs.md`)
**Lifecycle status:** inherited from RFC-057

---

## 1. Architecture acceptance

- [ ] `Workspace`, `Tab`, `TabId`, `DocumentIdentity`, and the `FocusedDraft`
      seam exist in `omriss-ui` (Dioxus-free).
- [ ] `omriss-core` crate has no workspace/tab/TabId concept.
- [ ] `omriss` app package owns the Dioxus tab strip rendering and the hot-draft local
      signal.
- [ ] `TabId` is stable across close/open during one app process; never reused.
- [ ] App modals target tabs by `TabId`, not raw index.
- [ ] RFC-001 crate boundaries preserved.
- [ ] No format adapter logic introduced by RFC-057.
- [ ] **No build dependency on RFC-050/RFC-053** — the project compiles and
      tests pass without JSON/TOML adapters.

---

## 2. Workspace model tests

- [ ] Empty workspace has no active tab.
- [ ] Opening first tab makes it active.
- [ ] Opening second tab appends and activates it.
- [ ] Creating new untitled tab appends and activates it.
- [ ] `TabId`s are unique and not reused.
- [ ] Closing first / middle / active-rightmost preserves a valid active id.
- [ ] Closing last tab returns workspace to empty.
- [ ] Activating unknown `TabId` returns `NoSuchTab` (no panic).
- [ ] Closing unknown `TabId` returns `NoSuchTab` (no panic).
- [ ] Dirty detection works; `dirty_tabs` ordered predictably.
- [ ] Stable `TabId` after closing an earlier tab.
- [ ] Active invariant enforced after every mutator.

---

## 3. Draft-validity seam (constraints #1, #2)

- [ ] `FocusedDraft` and `DraftApplicability` live in RFC-057's crate, not in
      an adapter crate.
- [ ] Markdown body editor returns `Applicable`/`Clean`, never `Invalid`.
- [ ] A fake validator drives the `Invalid` branch in tests.
- [ ] `apply()` mutates only when `Applicable`.
- [ ] On `Invalid`, source text is not mutated.
- [ ] Seam tests pass with no RFC-050/RFC-053 present.

---

## 4. Open / New behavior

- [ ] Open with no document open creates one tab.
- [ ] Open while a document is open appends a new tab.
- [ ] New while a document is open appends an untitled tab.
- [ ] Open/New no longer show the old unsaved-replacement guard.
- [ ] Opening an already-open file activates the existing tab (no duplicate).
- [ ] Dedupe uses `DocumentIdentity` normalized path, not raw string equality.
- [ ] Untitled tabs are never deduped.
- [ ] Recent files still update correctly.

---

## 5. Single-document regression

- [ ] One document open → tab strip hidden.
- [ ] Open/edit/save workflow visually unchanged.
- [ ] Document Map behavior unchanged.
- [ ] Focused Content / Writing Area behavior unchanged.
- [ ] Search, preview, raw source, undo, redo work for one document.
- [ ] Existing unsaved-close behavior still protects the document.
- [ ] Markdown byte-preservation golden tests pass.

---

## 6. Tab strip UI

- [ ] Tab strip appears at two documents, disappears back at one.
- [ ] Active tab visually clear.
- [ ] Dirty tab shows marker AND describes unsaved state in accessible name.
- [ ] Long tab names ellipsized / horizontally scrollable; overflow doesn't
      break layout.
- [ ] Close affordance discoverable and keyboard-reachable.
- [ ] Optional new-tab button only when strip is visible.

---

## 7. Tab switching

- [ ] Click activates a tab.
- [ ] Ctrl/Cmd+Tab next; Ctrl+Shift+Tab previous.
- [ ] Ctrl/Cmd+1 first; 2..8 corresponding; 9 last.
- [ ] Switching preserves each tab's focus location, undo/redo, dirty state.
- [ ] Switching applies a valid Markdown draft.
- [ ] Switching applies a valid structured draft (when structured editing
      exists).
- [ ] Switching **blocks** an invalid structured draft (fake validator now).
- [ ] When blocked: active tab unchanged; focus stays on field; source text
      unchanged; friendly guidance shown.

---

## 8. Draft and rendering performance (constraint #4)

- [ ] Typing in the active editor does not update inactive tabs.
- [ ] Typing does not re-render the tab strip on every keystroke (a single
      dirty-marker transition is acceptable).
- [ ] Inactive tab state intact while typing in the active tab.
- [ ] Switching tabs does not lose local draft state.
- [ ] Large Markdown typing remains responsive; no worse than single-document
      baseline.

---

## 9. Preview / search / status (constraints #7, #8, #9)

- [ ] `preview_open` is per-tab; switching restores each tab's preview/edit
      state.
- [ ] Search panel closes/clears on tab switch.
- [ ] Search results never remain visible for the wrong document.
- [ ] Command palette actions apply to the active tab only.
- [ ] Status bar reflects the active tab only.
- [ ] Stale status messages clear or refresh on tab switch.

---

## 10. Close-tab behavior

- [ ] Ctrl/Cmd+W closes the current tab.
- [ ] Closing clean active tab removes it; clean inactive removes it without
      needless active change.
- [ ] Closing dirty tab shows a tab-scoped dialog naming the document.
- [ ] Labels: Save / Close without saving / Cancel.
- [ ] Cancel keeps tab; Close without saving removes; Save commits valid draft
      then closes.
- [ ] Save failure keeps tab open with a friendly message.
- [ ] Invalid draft blocks save-and-close.
- [ ] External-modification conflict handled for the target tab.
- [ ] Closing active tab moves focus to the newly active header; closing last
      moves focus to Welcome primary action.

---

## 11. Quit behavior (constraints #3, #12)

- [ ] Quit with no dirty tabs exits immediately.
- [ ] Quit with dirty tabs shows the workspace-level guard listing count.
- [ ] Labels: Save All / Quit without saving / Cancel.
- [ ] Cancel keeps app open; Quit without saving exits.
- [ ] Save All validates the **active** draft first; blocks on invalid active
      draft and activates its field.
- [ ] Save All saves dirty saved-path tabs in order.
- [ ] Save All prompts Save As for untitled dirty tabs; Save As cancel cancels
      quit.
- [ ] Save All handles external-modification conflict per tab.
- [ ] Save All stops on first save failure, activates the failed tab, keeps the
      app open, preserves already-saved tabs.
- [ ] Save All does **not** scan inactive tabs for invalid drafts.

---

## 12. Accessibility QA

- [ ] `role="tablist"`, each `role="tab"`, active `aria-selected="true"`.
- [ ] Tabs associated with the active panel via ARIA where practical.
- [ ] Screen reader announces name, position, selected, unsaved state.
- [ ] Keyboard focus moves across tabs; Enter/Space activates.
- [ ] Close affordance keyboard-reachable.
- [ ] Dirty state not color-only.
- [ ] Focus after close predictable.
- [ ] Reduced motion honored.

---

## 13. i18n QA (constraint #12)

- [ ] All RFC-057 user-facing strings are catalog keys.
- [ ] English and Japanese catalogs include all new keys.
- [ ] Keys sorted per RFC-043.
- [ ] No untranslated fallback in normal UI.
- [ ] Wording plain and non-technical.
- [ ] No "Discard" where "Close without saving" / "Quit without saving" is
      clearer.

---

## 14. Keyboard conflict verification (constraint #10)

- [ ] `Ctrl+W`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+1..9`, `Ctrl+N` verified
      against the current `keyboard::interpret` map at implementation time.
- [ ] No collision with O/S/Shift+S/Z/Y/Shift+Z/Alt-arrows/Esc/Enter/Up/Down/
      Backquote/F/P/Shift+P or palette bindings (RFC-022).

---

## 15. Cross-platform smoke tests

### Linux
- [ ] Open multiple files; switch by mouse and keyboard; close dirty tab;
      quit with Save All; dedupe same path; long-strip overflow.

### macOS
- [ ] Cmd shortcuts; Cmd+W close; Cmd+O open; Cmd+S save; quit guard on app
      quit; native Save As in Save All.

### Windows
- [ ] Ctrl shortcuts; WebView renders the strip; path dedupe case behavior;
      per-tab external-modification check; Save All handles permission failure.

---

## 16. Final release readiness (constraint #11)

- [ ] `cargo fmt --check` passes. **(required)**
- [ ] `cargo test --workspace` passes. **(required)**
- [ ] `scripts/check-rfcs.sh` passes. **(required)**
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`. **(recommended,
      not a gate unless RFC-040/CI is updated)**
- [ ] Single-document regression suite passes.
- [ ] Multi-document suite passes.
- [ ] Manual QA completed on target platforms.
- [ ] RFC-057 open questions resolved or explicitly deferred.
- [ ] RFC-057 can move `rfcs/proposed/` → `rfcs/done/` after the
      implementation release.
