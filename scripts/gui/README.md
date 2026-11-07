# GUI test harness

The tools every GUI task uses to launch omriss, drive it, and collect evidence.
QA tooling, not shipped code.

**Use these and extend these.** Do not copy them into a task's temporary
directory, and never depend on a helper that lives in a scratchpad or under
`.git-exclude/tmp/`: it will not be there next time. Fix the tracked copy
instead, so the fix survives.

## Why this directory exists

Until it did, each GUI task rebuilt its own launcher and helpers in a temporary
directory. The same boilerplate was copied five times and drifted each time —
one copy isolated only one of the four XDG directories, another only *printed*
the zero-instances check instead of enforcing it — and one helper,
`atspi_text.py`, lived in a session scratchpad and was lost. The evidence
scripts for RFC-071 and RFC-076 depended on it and can no longer be re-run.

A harness rebuilt per task is debugged during the run it is meant to perform,
which is why GUI runs kept having to be repeated. Tracked, it arrives at each
task already known to work.

## Prerequisites

Checked by `gui_require` before anything starts, and named when missing.

| Needed for | Requirement | Arch package |
|---|---|---|
| everything | `niri` — window ids, focus checks, screenshots | `niri` |
| everything | an X server on `:1` — the app runs on the X11 backend so `xdotool` can reach it. On the maintainer's machine niri starts it through `xwayland-satellite` | `xwayland-satellite`, `xorg-xwayland` |
| everything | Python with `gi` and Atspi 2.0, and a running accessibility bus | `python-gobject`, `at-spi2-core` |
| everything | a built `target/release/omriss` | `cargo build --release -p omriss` |
| keyboard and text input | `xdotool` | `xdotool` |
| native Wayland windows only (the portal file chooser) | `wtype` — not wrapped by `lib.sh` | `wtype` |

**`xdotool` matters twice.** On Arch the package also provides `libxdo`, which
the desktop application *links against* (`crates/app/Cargo.toml` lists it as
`libxdo-dev`). Without it, not only is GUI input impossible —
`cargo build -p omriss` and `cargo test --workspace` fail to link with
`unable to find library -lxdo`. The default-members gate (`cargo test` without
`--workspace`) still runs, because RFC-075 moved the components into a crate
that does not need the WebView.

## Quick start

```sh
cargo build --release -p omriss
bash scripts/gui/selftest.sh
```

Run the self-test before a GUI task, not after the first failure. Steps that
need keyboard input are reported as `SKIPPED` when `xdotool` is absent — never
as passed.

## Writing a scenario

```bash
#!/usr/bin/env bash
set -uo pipefail
. scripts/gui/lib.sh

gui_init                                         # fresh isolated state
cp crates/core/tests/fixtures/academic-paper.md "$OMRISS_GUI_WORK/case.md"
before=$(gui_sha "$OMRISS_GUI_WORK/case.md")

gui_launch "$OMRISS_GUI_WORK/case.md"            # zero before, exactly one after
gui_press button 'Abstract'
gui_wait_until "Abstract is focused" gui_tree_has "[heading] 'Abstract'"
gui_type_into entry 'Edit section text' 'hello'  # asserts the text landed
gui_press button 'Save'
gui_wait_until "saved" grep -qF hello "$OMRISS_GUI_WORK/case.md"
gui_quit                                         # stops ours, asserts zero after
```

**After every action, wait for its effect.** `gui_press` and `gui_key` return
as soon as the input is delivered; whether it *did* anything is the next line's
job. A press that succeeded and changed nothing is the failure worth catching.

## Functions

| Function | What it does |
|---|---|
| `gui_init` | checks prerequisites, makes a fresh isolated state, stops our instance on exit |
| `gui_launch FILE` | refuses if any omriss is running; starts exactly one, X11 backend, all four XDG directories isolated, locale pinned; requires exactly one new window |
| `gui_quit` | stops this run's instance and asserts zero remain |
| `gui_press ROLE NAME` | invokes a node's default action through AT-SPI |
| `gui_focus_node ROLE NAME` | moves focus to a node through AT-SPI and asserts it arrived |
| `gui_type_into ROLE NAME TEXT` | replaces a field's content and asserts the text landed |
| `gui_key KEYS…` | sends keys to this run's window after verifying it has focus |
| `gui_tree`, `gui_tree_has TEXT` | the accessible tree, and a fixed-string search in it |
| `gui_text ROLE NAME`, `gui_text_contains ROLE NAME TEXT` | a node's full text |
| `gui_wait_until DESC CMD…` | polls a condition with a deadline instead of sleeping a fixed time |
| `gui_check DESC CMD…` | aborts the run unless the command succeeds |
| `gui_sha FILE`, `gui_bytes FILE` | byte evidence: a digest, and the bytes with line endings visible |
| `gui_shot NAME` | a capture of this run's window, only where a picture is the evidence |

Every function is prefixed `gui_`. An earlier copy of these helpers defined
`tree`, which silently shadowed `/usr/bin/tree` in any script that sourced it.

## Configuration

| Variable | Default |
|---|---|
| `OMRISS_BIN` | `target/release/omriss` — must be named `omriss` |
| `OMRISS_GUI_WORK` | `.git-exclude/tmp/gui` |
| `OMRISS_GUI_EVIDENCE` | `$OMRISS_GUI_WORK/evidence` |
| `OMRISS_GUI_DISPLAY` | `:1` |
| `OMRISS_GUI_LANG` | `en_US.UTF-8` — set `ja_JP.UTF-8` for a Japanese run |
| `OMRISS_GUI_TIMEOUT` | `15` seconds per wait |

The application's own output goes to `$OMRISS_GUI_WORK/omriss.log`. When a
launch times out waiting for a window, read that first: a binary that exits on
startup fails here, not where it looks like it did.

The locale is pinned by unsetting `LC_ALL`, `LC_MESSAGES` and `LANGUAGE` as well
as setting `LANG`. On a machine whose shell sets `LC_*` to Japanese, `LANG`
alone is overridden and a run silently comes up in the wrong language.

## The Python tools

All four register as AT-SPI clients — the same interface a screen reader uses —
and are runnable on their own.

| Tool | Purpose |
|---|---|
| `atspi_probe.py [APP]` | the accessible tree: roles, names, states, live-region attributes. Truncates text for readability |
| `atspi_text.py APP ROLE NAME` | one node's **full** text, for asserting against |
| `atspi_act.py APP ROLE NAME [--focus]` | press a node, or move focus to it |
| `atspi_events.py SECONDS` | log the events an assistive technology would receive |

`atspi_text.py` and `atspi_act.py` match nodes by the same rule — role equals
ROLE, name starts with NAME — so a script never reads one node and presses
another.

## Headless runs — tried, and deliberately not adopted

GTK 3's own `broadwayd` runs omriss with no display server and no window on
screen, and it works further than expected (2026-11-07): the full WebView
content reaches the accessibility tree, and AT-SPI presses round-trip through
the event channel. **Text entry does not work.** There is no X server for
`xdotool`, WebKit exposes no `EditableText` interface, and delivering keys means
speaking broadwayd's client protocol by hand.

That is more machinery than the benefit justifies, and a half-capable backend
would make every scenario author track which helpers work where. The X11 path
above covers everything. Do not reopen this without a reason the X11 path
cannot meet.

## How the rules map onto the harness

`.git-exclude/rules/project-instructions-gui-common.md` states the rules; this
is where each one is enforced, so a scenario gets it by default.

| Rule | Where |
|---|---|
| isolate every directory the app reads | `gui_launch`: config, state, cache and data |
| one instance only; never act on a stale window | `gui_launch` refuses unless zero instances and zero Omriss windows exist, then requires exactly one, so the window is provably this run's. The niri window is matched by `app_id`, **not** pid — under xwayland-satellite niri reports the satellite's pid for every X11 window. The X11 window used for input is matched by the real pid |
| assert, then continue; abort at the first anomaly | `gui_check`, `gui_wait_until`, `gui_type_into`'s read-back, `gui_abort` |
| no frame-sampling loops | `gui_wait_until` polls *state*, never captures frames |
| backends | `gui_launch` runs X11; `gui_key` and `gui_type_into` drive it through `xdotool` |
| compare before and after identically | `gui_sha`, `gui_bytes`; the pinned locale and isolated state |
