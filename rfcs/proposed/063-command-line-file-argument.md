# RFC-063: Command-Line File Argument

**Project:** omriss — Omriss Editor
**Milestone:** Verification enablement, ahead of RFC-054 J7
**Status.** Proposed
**Document type:** Small capability RFC
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-015 (file open lifecycle), RFC-036 (settings/recent files)
**Related RFCs:** RFC-054 (whose J7 verification this exists to enable)

---

## 1. Summary

Let omriss open a file named on the command line:

```sh
omriss notes.md
omriss config.json
```

Today it takes no arguments at all. Every path to a rendered document goes
through the Open dialog or the Recent Files list, both of which require a click.

## 2. Motivation

Two independent reasons, and the second is what made this urgent.

**For users.** `omriss notes.md` from a terminal is what anyone expects of a
desktop editor. It is also how a desktop environment's "Open with…" integration
passes a file — so without it, double-clicking a `.md` file in a file manager
cannot route to omriss even when it is the registered handler.

**For verification.** Since RFC-054 J3 every rendering check has depended on
synthetic input (`xdotool`) reaching the WebView. That failed outright in one
session — proven by a *visibly focused* button not responding to `Return` — and
worked in another. The J3 QA checklist had to be rewritten around the
possibility that it simply cannot be performed, and it names this gap as the
cause:

> `crates/app/src/main.rs` takes no command-line arguments, so there is no way
> to launch omriss with a file already open. Every path to a rendered document
> requires a click. That is what makes this item unperformable when input
> synthesis is unavailable — the cause is a product gap, not a harness one.

RFC-054 J7 is the largest user-visible slice in the sequence, and the one whose
rendering claim most needs evidence. Leaving its verification dependent on
whether input synthesis happens to work that day is the avoidable risk this RFC
removes. RFC-055 and RFC-056 will each need the same check.

## 3. Goals

- Open a file given as the first command-line argument, through the **same**
  load path the Recent Files list already uses.
- Report a missing or unreadable file the same way any other failed open is
  reported — no new error surface.
- Make a rendered document reachable without synthetic input.

## 4. Non-goals

- **No option or flag parsing.** No `--help`, no `--version`, no long options.
  omriss takes a path and nothing else. Adding a CLI surface is a separate
  decision with its own compatibility commitments.
- No multi-file open — that is RFC-057 (multi-document workspace).
- No new file formats, filters, or detection behavior. The argument goes through
  the format detection RFC-052 §5.1 already defines.
- No change to the Welcome screen, Recent Files, or settings behavior when no
  argument is given.

## 5. Behavior

### 5.1 One argument, treated as a path

The **first** argument is the file to open. Additional arguments are ignored;
they are not an error, because a future RFC may give them meaning and failing
now would make that a breaking change.

Relative paths resolve against the process working directory, per normal
platform behavior.

### 5.2 Failure is not special

A path that does not exist, is a directory, or cannot be read is reported
through the **existing** failed-open path (`OpenOutcome::Failed`, RFC-039's
friendly error handling). The app still starts; the user lands on the Welcome
screen with a plain message. omriss must not exit with an error code or refuse
to launch because of a bad argument.

### 5.3 An argument beginning with `-`

Rejected with a plain message stating that omriss takes a file path and accepts
no options. This exists so `omriss --help` produces something intelligible
rather than "File not found: --help", **without** implying flags are supported.

A file whose name genuinely begins with `-` can be opened as `./-weird.md`, the
standard convention.

### 5.4 Format detection is unchanged

The argument is detected exactly as a dialog-opened file is — RFC-052 §5.1's
extension mapping, via `detect_format`. A `.json` argument opens as JSON, a
`.txt` as Markdown, an unknown extension as `PlainText`. Nothing here is a new
detection rule.

### 5.5 No argument

Unchanged behavior: Welcome screen, Recent Files from settings.

## 6. Data model

None. This RFC adds no type and no persisted state.

## 7. Detailed design

`crates/app/src/main.rs` reads the first argument and passes it into the Dioxus
app as context, the same mechanism `AppSettings` and the detected locale already
use. The app performs the load on mount through the existing
`file_dialog::open_markdown_path` → `handle_load` path — the identical route
`app.rs`'s Recent Files entry already takes.

Nothing new is written for reading, detecting, or loading. The whole change is
"where does the path come from."

## 8. Accessibility

No new UI surface, so no new accessibility obligation. The resulting state — a
document open in the workspace — is the state a dialog-opened file already
produces, and is covered by existing keyboard and screen-reader behavior.

## 9. Security

The argument is a local file path handled exactly as a dialog-chosen path is.
No shell interpretation, no network, no execution of file contents. A path the
user could not otherwise open does not become openable.

## 10. Test plan

Argument interpretation must be a **pure function**, tested without launching a
GUI — matching how `interpret_code` (keyboard) and `AppSettings::push_recent`
are the app crate's existing testable units:

- a plain path yields that path;
- no arguments yields none;
- extra arguments are ignored, first one wins;
- an argument beginning with `-` is rejected as an option, not treated as a path;
- `./-weird.md` is treated as a path.

Missing-file and unreadable-file behavior is already covered by
`open_markdown_path`'s existing handling; this RFC adds no new failure mode to
test there.

**Manual, once:** `cargo run -p omriss -- <path>` opens that file. This is the
capability's whole point, and it is what makes RFC-054 J7's rendering check
performable without synthetic input.

## 11. Migration and compatibility

No compatibility surface. Launching with no argument is unchanged, so no
existing behavior, setting, or file is affected. Nothing persists.

Because §4 declines flag parsing, adding options later remains free — no
argument shape is committed to beyond "the first argument is a path."

## 12. Alternatives considered

**Do nothing, keep verifying through synthetic input.** Rejected: it has already
failed once, and it leaves every future format's rendering check dependent on
environment luck. The J3 checklist had to be written around its own possible
unperformability, which is a bad shape for a required check.

**A test-only launch hook** (an env var or debug flag that preloads a file).
Rejected: it would make the tested path different from the shipped one, which is
the specific weakness that makes a verification harness untrustworthy. A real
argument is tested *and* useful.

**Full CLI with flags** (`--help`, `--version`, `--new`). Rejected as
disproportionate. Those are a separate decision; this RFC deliberately commits
to nothing that would block them.

## 13. Acceptance criteria

1. `omriss <path>` opens that file; `.md` and `.json` both work.
2. A bad path lands on the Welcome screen with a plain message; the app still
   starts.
3. `omriss --help` says omriss takes a file path and accepts no options.
4. No argument behaves exactly as before.
5. Argument interpretation is unit-tested without a GUI.
6. Detection is RFC-052 §5.1's, unchanged.
7. Zero diff under `crates/core/` and `crates/ui/`.
8. Gates green: `cargo fmt --check`, `cargo test --workspace`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `bash scripts/check-rfcs.sh`.

## 14. Open questions

None. The scope is deliberately small enough that the §5 behavior table settles
it.

## 15. Final decision summary

omriss accepts a single file path as its first command-line argument, routed
through the load path it already has. It exists to serve users who expect
`omriss file.md` to work, and to make RFC-054 J7's rendering verification depend
on a real product capability rather than on whether synthetic input reaches the
WebView that day.
