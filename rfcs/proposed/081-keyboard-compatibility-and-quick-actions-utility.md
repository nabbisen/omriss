# RFC-081: Keyboard Compatibility and the Quick Actions Utility

**Project:** omriss — Omriss Editor
**Milestone:** M16 — Readable and operable (0.19.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-022, RFC-059 (§12, landed)
**Related RFCs:** RFC-014, RFC-028, RFC-053, RFC-080, RFC-082, RFC-083

---

## 1. Summary

1. Shortcuts follow **the letter printed on the key**, on every keyboard
   layout.
2. **One binding table** declares each command's keys; the interpreter,
   Quick Actions and the keyboard reference page all derive from it.
3. **Quick Actions becomes the help utility**: one place to find every
   action, what it does, whether it can run now, and its keys — including the
   keys that work where the user is.
4. **New keys are chosen by a study**, never on instinct.
5. RFC-059's open sections §6.1, §6.3 and §6.5 are carried here.

Approved by the owner on 2026-11-26 (plan:
`.git-exclude/specs/omriss-plan-accessibility-keyboard-rtl-draft.md`).

## 2. Findings (2026-11-26)

| Finding | Where |
|---|---|
| **Shortcuts match physical key positions** (`Code::KeyZ`, …). On German QWERTZ, Z and Y swap: **the key labelled Z performs Redo**. On French AZERTY several shortcuts land on other letters | `crates/app/src/input/keyboard.rs:65-79` |
| Shortcut labels are hand-typed strings in the registry, separate from the mapping; they can drift, and macOS is told "Ctrl" where Cmd is used | `crates/ui/src/interface/commands.rs` |
| Quick Actions lists commands by title only; hints are 11px; contextual keys (Esc, Enter, arrows, Tab) appear nowhere in the app | `crates/components/src/command_palette.rs` |
| The full key list exists only in the online book | `docs/src/keyboard-reference.md` |
| RFC-059 §6.1 (toolbar reachable only by many Tabs), §6.3 (undo with a pending draft), §6.5 (no direct move to the Document Map or Writing Area); the Document Map's keys handled twice (RFC-059 §12) | RFC-059 |
| Ctrl+\` works on a Japanese JIS keyboard — checked by the owner, 2026-11-26 | — |

## 3. Design

### 3.1 Letters by what is printed on the key

A letter shortcut matches `KeyboardEvent.key` when that is a Latin letter, and
falls back to the physical `code` otherwise: an Arabic, Cyrillic or Greek
layout produces non-Latin letters, and there the key's position is the only
sensible match. Non-letter keys (arrows, Enter, Esc, F-keys, punctuation) are
decided case by case in the study (§3.5), because punctuation moves between
layouts.

### 3.2 One binding table

Each command declares its keys once, as data. From that table:

- the interpreter is generated (`keyboard::interpret` stops being hand-written
  matches);
- Quick Actions shows the keys;
- the keyboard reference page is checked against it by a test, so the book
  cannot drift;
- labels are platform-aware (⌘ on macOS) and translated where they contain
  words ("Esc", "Enter").

### 3.3 Quick Actions as the help utility

One place, rather than a palette and a separate help dialog that could
disagree:

- **every** action is a command — an audit makes the registry complete
  (toolbar, menus, the entry point of every dialog);
- each entry shows its title, **a one-line description**, and its keys at a
  readable size (RFC-080 §5.4);
- an action that cannot run now is **shown disabled with the reason** the
  user can act on ("Save — no document is open"), never silently absent
  (RFC-053's rule);
- search matches titles, descriptions and keys (typing "ctrl+s" finds Save);
- with an empty query, entries are grouped by area, and a **"Keys here"**
  section lists the keys that work where focus was — Document Map arrows,
  Enter, Esc, Tab — styled and announced as information, not as actions;
- its own keys are stated in a visible footer.

### 3.3a Finding Quick Actions and the keys — visible and hidden

*Added 2026-11-26 on the owner's finding: Ctrl+P opens Quick Actions, and
nothing in the visible app says so — the welcome screen, the toolbar and the
status bar never mention it.*

A help utility nobody can find is not help [COGA 4.3.4, 4.8.5]. What is
visible and what waits is decided by one question: **does a user who knows
nothing need it to get started?**

**Always visible**

| What | Where | Why |
|---|---|---|
| A **Quick Actions button**, word and icon | the toolbar, at a fixed place beside Settings | it is the door to every action and to the keys; a door must be visible. When the toolbar runs out of room (large text, narrow window), other buttons move into Quick Actions first — **this one is the last to go** |
| Its key, in words | the welcome screen's start line ("…or press Ctrl+P to find any action"), and the Quick Actions footer | the one key worth learning, shown where a new user looks first |
| Each command's keys | beside the command, inside Quick Actions | the place a user goes to find an action is the place to learn its key |

**One step away — hidden until asked for**

| What | How it is reached | Why not visible |
|---|---|---|
| **"Keys here"** — the keys that work where the user is | Quick Actions with an empty search (§3.3) | useful once the user wants to go faster, noise before |
| **All keyboard shortcuts**, grouped by area | a Quick Actions command | a reference, not a starting point |

**Not added**

- **Key hints on every toolbar button.** They would change today's look for
  everyone (RFC-080 §4 rule 3) and crowd the toolbar at large sizes. Instead,
  every control with a key carries **`aria-keyshortcuts`** from the binding
  table, so screen readers announce the key without anyone seeing clutter.
- **A Quick Actions entry inside Settings.** Settings is where settings are
  (RFC-083); a door hidden inside a dialog is not a door. The toolbar button
  is always visible, which Settings is not.
- **Tooltips as the only place a key is written** — hover is not for
  essential information (RFC-080 §5.4 rule 3).

The button's label is part of RFC-080's vocabulary audit (§5.4 rule 10):
"Quick Actions" is this application's own term, and the audit decides whether
a plainer word ("All actions") serves first-time users better, in both
languages.

### 3.4 Moving between areas — RFC-059 §6.1 and §6.5

Commands "Go to toolbar", "Go to Document Map" (the active node, not the
container), "Go to Writing Area", always in Quick Actions. Whether a key
cycles between areas — and which — is decided by the study.

### 3.5 The key study

Before any new key is assigned, this RFC records for each candidate:

- **platform conventions** — Windows, GNOME, KDE, macOS. GNOME's, for
  instance: Ctrl with a letter; Shift+Ctrl for the reverse of a function; no
  Alt+letter shortcuts, which collide with access keys; Esc closes transient
  containers; "don't assign shortcuts for everything" [GNOME];
- **what the WebViews already do** with it — WebView2 and WebKitGTK built-in
  keys (zoom, print, reload, find, developer tools);
- **input methods**, Japanese first;
- **screen readers** — NVDA, JAWS, Narrator, Orca, and the keys they reserve;
- **layouts** — QWERTY, QWERTZ, AZERTY, JIS, Arabic;
- **reach** — laptops where F-keys need Fn; one-handed use; **Sticky Keys**,
  which older users rely on: every chord must work with modifiers pressed in
  sequence.

Rules:

- **A shortcut is only an accelerator.** Every action is reachable without
  one — Quick Actions and visible controls.
- **Fewer is better.** Only frequent actions get keys.
- **Existing keys change only for a measured reason**, with a note to users:
  changing what a key does is the most confusing change an app can make. The
  study covers existing keys too (Ctrl+P is Print by convention on every
  platform, for example).

Candidates evaluated, none decided: help (opening Quick Actions at "Keys
here"), text size, moving between areas, settings. Measurements: dev team on
Linux; owner on Windows and JIS.

### 3.6 RFC-059's remainder

- **§6.3, undo with a pending draft:** a stated contract — what Ctrl+Z does
  while an edit is not yet applied, shown to the user, never discarding text.
- **The Document Map's double-handled keys:** a key the tree handles does not
  also reach the application's shortcuts.

## 4. Non-goals

- User-customisable shortcuts. The binding table makes them possible later
  without a redesign.
- Access keys (Alt+letter mnemonics) — a study candidate only.

## 5. Validation

- Unit tests over the binding table: each layout's key events (QWERTY,
  QWERTZ, AZERTY, JIS, Arabic) resolve to the intended command.
- The reference-page agreement test.
- Quick Actions: level-2 tests for descriptions, disabled reasons and "Keys
  here" per area; GUI check with `scripts/gui/`.
- Sticky Keys: every shortcut with sequential modifiers — owner on Windows,
  dev team on GNOME.
- The study's table, with each row's evidence.

## 6. Acceptance criteria

- On QWERTZ and AZERTY, the key labelled Z undoes.
- No hand-written shortcut label remains.
- Every action can be found and run from Quick Actions; every disabled entry
  says why.
- Every key assignment has a row in the study with its reasons.
- RFC-059 §6.1, §6.3 and §6.5 are each resolved or explicitly deferred here.

## 7. Sources

[GNOME] GNOME Human Interface Guidelines, *Keyboard* (read 2026-11-26).
Others as listed in RFC-080 §9.
