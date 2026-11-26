# RFC-083: User Settings

**Project:** omriss — Omriss Editor
**Milestone:** M16 — Readable and operable (0.19.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, UI/UX designer, Rust developer, QA engineer
**Depends on:** RFC-036, RFC-079, RFC-059 (§12, landed)
**Related RFCs:** RFC-068, RFC-073, RFC-080, RFC-081

---

## 1. Summary

One Settings dialog, a small set of settings that all work, saved so they
are never lost, and designed for the people RFC-080 serves: findable from
everywhere, few and plainly named choices, each showing its effect, instantly
applied, always reversible, and bounded so no combination can make the app
unusable.

Approved by the owner on 2026-11-26 (plan:
`.git-exclude/specs/omriss-plan-accessibility-keyboard-rtl-draft.md`).

## 2. Findings (2026-11-26)

| Finding | Where |
|---|---|
| Three settings stored; **only `recent_files` is used** — `font_size` and `line_wrap` are saved and loaded and read by nothing | `crates/app/src/storage/settings.rs` |
| **The language is not saved**; it is detected at every start (RFC-073 §4.5, not built) | `main.rs`, `toolbar.rs` |
| The only settings UI is the unlabelled gear popover holding the language select | `toolbar.rs` |
| **An unparsable settings file silently resets every setting**, and the next save writes the defaults over it | `settings.rs` `load()` / `save()` |
| **Saves are not atomic** (`fs::write`): a crash mid-write truncates the file, which then triggers the full reset | `settings.rs` `save()` |
| The window opens at 1080 × 720 every time | `main.rs` |

## 3. The settings

| Group | Setting | Visible | Source of the need |
|---|---|---|---|
| Display | Text size | **always** | RFC-080 §5.1 |
| Display | Colours | **always** | RFC-080 §5.1 |
| Display | Font for writing | disclosure | RFC-080 §5.1 |
| Display | Line spacing | disclosure | RFC-080 §5.1 |
| Language | Language | **always** | RFC-073 §4.5 |
| Editing | Wrap long lines | disclosure | RFC-080 §5.5 |
| Editing | Clear the recent-files list | disclosure | RFC-036 |

Nothing else until a finding asks for it [COGA 4.6.3, 4.7.4].

Also persisted, not shown as settings: window size and maximised state (not
position — it breaks when a monitor is disconnected); which disclosures the
user left open.

## 4. Dialog design

1. **Findable from everywhere** [COGA 4.6.1]: a toolbar button with the word
   "Settings" and its icon; a Quick Actions entry. A key only if RFC-081's
   study gives one.
2. **Always visible:** text size, colours, language — on one screen without
   scrolling even at 200% in the default window (RFC-079's layout check).
   Older users are the least likely to go looking [NNG]; a user stuck in a
   language they cannot read must still find Language. **Each language is
   named in itself** ("English", "日本語"), never translated.
3. **Disclosures that do not confuse** [COGA 4.3.2, 4.9.3]:
   - the disclosure names what is inside ("More display options: font, line
     spacing");
   - a visible, labelled control, never hover; left open, it stays open;
   - **never labelled "Advanced"** — it tells readers who already blame
     themselves that the options are not for them;
   - **a hidden setting that differs from its default is announced on the
     disclosure** ("2 changed"), so the app never looks different for a
     reason the user cannot see.
4. **Each choice shows its effect** before and as it is chosen: text sizes
   written at their size; fonts in their own face [LV 3.3.2]; colours as
   sample text on its background **with the name in words**, never a bare
   swatch. "Same as my computer" says what that is now ("currently: Dark").
5. **Applied and saved at once** — no Apply or OK to forget. The user's place
   in the document stays in view while it changes (RFC-080 §5.4 rule 9).
6. **Reversible** [COGA 4.5.2, 4.9.4]: "Reset to defaults" says what it will
   change, asks once, covers hidden settings, and can itself be undone from
   the same place.
7. **Bounded:** no free colour picker, no size beyond 200%; every combination
   passes the layout check; the dialog scrolls at any size [LV 3.7.1]. There
   is no state a user cannot get back from, which is why there is no hidden
   recovery switch.
8. Built to RFC-080's clarity rules, contained and keyboard-complete
   (RFC-059 §12), every control labelled. It replaces the gear popover.

## 5. Storage

- **Never lose settings silently.** A value that fails to parse resets that
  value only. An unreadable file is kept as `settings.toml.bak` before
  anything overwrites it, and the user is told once, in words, near where
  they are (RFC-080 §5.4 rule 8).
- **Atomic writes:** a temporary file renamed into place — the technique
  RFC-068 plans for documents; one helper if both land.
- `font_size` is replaced by text size; old files load without it.
  `line_wrap` keeps its name and becomes real.
- RFC-036's invariant holds: settings never contain document content.

## 6. Non-goals

- Per-document settings.
- Settings sync across machines.
- Import and export of settings.

## 7. Validation

- Storage: round trip; one bad value keeps every other value; an unreadable
  file is backed up and reported; a write interrupted mid-way leaves the
  previous file intact.
- Every setting changes what it says — level-2 tests, and the layout check for
  every combination.
- The always-visible set fits without scrolling at 200% — layout check.
- Keyboard-complete and screen-reader-labelled — `scripts/gui/` and AT-SPI;
  Windows by the owner.
- Restart: every setting, the language and the window size survive.

## 8. Acceptance criteria

- No stored setting is read by nothing.
- No path loses a user's settings without telling them.
- The dialog meets RFC-080 in every palette and size, and replaces the gear
  popover.

## 9. Sources

As in RFC-080 §9.
