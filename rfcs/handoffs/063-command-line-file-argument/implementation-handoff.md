# RFC-063 Implementation Handoff

**Governing RFC:** [RFC-063](../../proposed/063-command-line-file-argument.md)
**Sequenced:** ahead of RFC-054 J7

---

## 1. Purpose

Accept a file path as omriss's first command-line argument and open it at
startup, so that a rendered document is reachable without synthetic input.

## 2. Scope

`crates/app/` only. **Zero diff under `crates/core/` and `crates/ui/`** —
nothing this RFC needs lives there.

## 3. Required implementation

1. **A pure argument-interpretation function** in `crates/app/`, taking an
   iterator of arguments and returning the path to open, if any. This is the
   unit that gets tested; keep it free of I/O and of Dioxus.
2. **`main.rs`** passes the interpreted result into the app as context, the same
   way `AppSettings` and the detected locale already are.
3. **The app loads it on mount** via the existing
   `file_dialog::open_markdown_path` → `handle_load` route — the identical path
   `app.rs`'s Recent Files entry takes today.

Behavior is RFC-063 §5, and §5 is binding:

| Input | Result |
|---|---|
| one path | open it |
| several | first wins, rest ignored — not an error (§5.1) |
| missing / unreadable / directory | existing `OpenOutcome::Failed` path; **app still starts** (§5.2) |
| begins with `-` | plain "omriss takes a file path and accepts no options" (§5.3) |
| none | unchanged Welcome screen (§5.5) |

## 4. Non-change scope

1. **No flag or option parsing.** Not `--help`, not `--version`. §5.3's message
   exists so `omriss --help` is intelligible, *not* as the start of a CLI.
2. **Do not add a dependency.** No `clap`, no `argh`. This is
   `std::env::args()` and a `match`.
3. **Do not change detection.** The argument goes through RFC-052 §5.1's
   mapping via the existing route. No new extension behavior.
4. **Do not change no-argument startup** — Welcome screen and Recent Files
   behave exactly as now.
5. **Do not exit nonzero or refuse to launch** on a bad path. A bad argument is
   a friendly message, not a fatal error.
6. No `mod.rs`; files over 500 ELOC get split.

## 5. Required tests

Argument interpretation is a pure function and must be unit-tested **without a
GUI**, matching how `interpret_code` and `AppSettings::push_recent` are already
the app crate's testable units:

- a plain path yields that path;
- no arguments yields none;
- extra arguments ignored, first wins;
- an argument beginning with `-` is rejected as an option, not a path;
- `./-weird.md` is treated as a path.

Do **not** re-test missing-file handling — `open_markdown_path` already owns it
and this RFC adds no new failure mode there.

Baseline: **401 passed, 12 suites.** Counts may only grow. No shipped test may
be modified.

## 6. Manual verification — the point of the RFC

```sh
cargo run -p omriss -- <some.md>
cargo run -p omriss -- <some.json>
```

Both must open with the document rendered. **Capture a screenshot of at least
one.** That artifact is what makes RFC-054 J7's rendering check performable
without synthetic input, and demonstrating it here is this slice's real
deliverable.

Also confirm `cargo run -p omriss` with no argument still shows the Welcome
screen unchanged.

## 7. Acceptance criteria

RFC-063 §13, all eight. Criterion 8's gates:

```sh
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check-rfcs.sh
```

## 8. Documentation

- `README.md` — the Quick Start section gains the invocation. One line.
- `docs/src/getting-started.md` — same, in prose.
- `CHANGELOG.md` — user-facing; say so.

Do **not** touch `docs/src/file-formats.md`. JSON's "Supported" promotion is
RFC-054 J7's, and nothing here changes what formats do.

## 9. Known risks

| Risk | Severity | Mitigation |
|---|---|---|
| Scope creep into flag parsing | Medium | §4.1–4.2; the RFC declines it explicitly so later options stay unblocked |
| A bad argument prevents launch | Medium | §5.2 and non-change item 5 — failure is a message, not a fatal |
| Load-on-mount races the app's own startup state | Low–Medium | reuse the Recent Files route rather than inventing one; if it needs different sequencing, say so rather than duplicating the load path |

## 10. Escalation triggers

Stop and file a clarification request if:

- opening at startup cannot reuse `open_markdown_path`/`handle_load` without
  duplicating logic;
- the load needs to happen before Dioxus starts, rather than on mount;
- a shipped test must change.
