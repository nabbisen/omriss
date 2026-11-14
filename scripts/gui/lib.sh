# shellcheck shell=bash
# omriss GUI test harness. Source it from a scenario script:
#
#     . scripts/gui/lib.sh
#     gui_init
#     gui_launch "$OMRISS_GUI_WORK/copy.md"
#     gui_press button 'Abstract'
#     gui_wait_until "Abstract is the focused section" gui_tree_has "[heading] 'Abstract'"
#     gui_quit
#
# It implements the "GUI Evidence and Automation Rules" in
# .git-exclude/rules/project-instructions-gui-common.md, so a scenario gets them
# by default rather than by remembering them. scripts/gui/README.md explains
# each function and lists the prerequisites.
#
# Every function is prefixed `gui_`: an earlier copy of this file defined
# `tree`, which silently shadowed /usr/bin/tree in any script that sourced it.

GUI_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
GUI_REPO=$(cd "$GUI_DIR/../.." && pwd)

: "${OMRISS_BIN:=$GUI_REPO/target/release/omriss}"
: "${OMRISS_GUI_WORK:=$GUI_REPO/.git-exclude/tmp/gui}"
: "${OMRISS_GUI_EVIDENCE:=$OMRISS_GUI_WORK/evidence}"
: "${OMRISS_GUI_DISPLAY:=:1}"
: "${OMRISS_GUI_LANG:=en_US.UTF-8}"
: "${OMRISS_GUI_TIMEOUT:=15}"

GUI_PID=""        # the omriss process this run launched
GUI_NIRI_ID=""    # its window, as niri knows it
GUI_X_WINDOW=""   # its X11 window, for keyboard input (resolved on first use)

# ── reporting and failure ────────────────────────────────────────────────────

gui_log() { printf '  %s\n' "$*"; }

# Abort at the first anomaly (rule: "Assert, then continue"). Stops only the
# instance this run launched — never an omriss the user has open themselves.
gui_abort() {
  printf '  ABORT: %s\n' "$*" >&2
  gui__stop_ours
  exit 1
}

# gui_check DESC CMD [ARGS...] — run CMD; abort unless it succeeds.
gui_check() {
  local desc=$1; shift
  if "$@"; then gui_log "ok: $desc"; else gui_abort "check failed: $desc"; fi
}

# gui_wait_until DESC CMD [ARGS...] — poll CMD until it succeeds, or abort after
# OMRISS_GUI_TIMEOUT seconds. This replaces fixed `sleep`s: it waits exactly as
# long as the state takes to arrive, and fails loudly when it never does. It
# polls *state*; it never captures frames (rule: "No frame-sampling loops").
gui_wait_until() {
  local desc=$1; shift
  local deadline=$((SECONDS + OMRISS_GUI_TIMEOUT))
  until "$@"; do
    [ "$SECONDS" -lt "$deadline" ] || gui_abort "timed out after ${OMRISS_GUI_TIMEOUT}s waiting for: $desc"
    sleep 0.2
  done
}

# ── prerequisites ────────────────────────────────────────────────────────────

# Everything the harness needs, checked up front and named when missing. A
# dependency nobody wrote down surfaces as a confusing failure halfway through
# a run; this makes it surface as a sentence before anything starts.
gui_require() {
  local missing=()
  command -v niri >/dev/null 2>&1 || missing+=("niri — window ids, focus checks, screenshots")
  python3 -I -c 'import gi; gi.require_version("Atspi", "2.0"); from gi.repository import Atspi' \
    >/dev/null 2>&1 || missing+=("python3 with gi and Atspi 2.0 — python-gobject, at-spi2-core")
  [ -S "/tmp/.X11-unix/X${OMRISS_GUI_DISPLAY#:}" ] \
    || missing+=("an X server on $OMRISS_GUI_DISPLAY — the app runs under XWayland so input reaches it")
  [ -x "$OMRISS_BIN" ] || missing+=("the binary $OMRISS_BIN — cargo build --release -p omriss")
  if [ ${#missing[@]} -ne 0 ]; then
    printf '  ABORT: missing prerequisites:\n' >&2
    printf '    - %s\n' "${missing[@]}" >&2
    exit 1
  fi
  # pgrep -x and the one-instance checks match the process by this exact name.
  [ "$(basename "$OMRISS_BIN")" = omriss ] \
    || gui_abort "the binary under test must be named 'omriss', not '$(basename "$OMRISS_BIN")'"
}

# Keyboard and text input need xdotool on top of gui_require. Kept separate so
# the accessibility-only helpers work on a machine without it.
gui_require_input() {
  command -v xdotool >/dev/null 2>&1 \
    || gui_abort "xdotool is not installed — needed for keyboard and text input (pacman -S xdotool)"
  [ -n "$GUI_X_WINDOW" ] && return 0
  local w g
  for w in $(DISPLAY=$OMRISS_GUI_DISPLAY xdotool search --pid "$GUI_PID" 2>/dev/null); do
    g=$(DISPLAY=$OMRISS_GUI_DISPLAY xdotool getwindowgeometry "$w" | tail -1)
    case "$g" in *10x10*) ;; *) GUI_X_WINDOW=$w ;; esac   # skip the toolkit's 10×10 helpers
  done
  [ -n "$GUI_X_WINDOW" ] || gui_abort "no X11 window found for omriss pid $GUI_PID"
}

# ── isolation and lifecycle ──────────────────────────────────────────────────

# A fresh, isolated state for this run (rule: "Isolate every directory the app
# reads"). Relaunches within one run share it, so a close-and-reopen scenario
# still sees what the previous launch saved.
gui_init() {
  gui_require
  rm -rf "${OMRISS_GUI_WORK:?}/xdg"
  mkdir -p "$OMRISS_GUI_WORK"/xdg/config "$OMRISS_GUI_WORK"/xdg/state \
           "$OMRISS_GUI_WORK"/xdg/cache "$OMRISS_GUI_WORK"/xdg/data "$OMRISS_GUI_EVIDENCE"
  trap 'gui__stop_ours' EXIT
}

gui__process_count() { pgrep -c -x omriss || true; }

# The niri ids of every Omriss window.
#
# Matched by app_id, NOT by pid, deliberately: omriss runs on the X11 backend,
# and to niri an X11 window is a surface created by xwayland-satellite, so niri
# reports *satellite's* pid for it, never the application's. Verified on
# 2026-11-07 with glxgears: niri said pid 1731, glxgears was 4117642. Matching
# niri windows by the app's pid therefore never matches anything.
#
# The rule's purpose — never act on a stale window — is met another way: launch
# refuses unless there are zero Omriss windows, then requires exactly one, so
# the window that appears is provably ours. The X11 window used for input *is*
# chosen by the real pid (gui_require_input), because X clients set
# _NET_WM_PID themselves.
gui__omriss_windows() {
  niri msg --json windows | python3 -I -c '
import json, sys
print(" ".join(str(w["id"]) for w in json.load(sys.stdin) if w.get("app_id") == "Omriss"))'
}

gui__one_process() { [ "$(gui__process_count)" = 1 ]; }
gui__no_process()  { [ "$(gui__process_count)" = 0 ]; }
gui__one_window()  { local ids; ids=$(gui__omriss_windows); [ -n "$ids" ] && [ "${ids// /}" = "$ids" ]; }
gui__no_window()   { [ -z "$(gui__omriss_windows)" ]; }

# gui_launch FILE — exactly one isolated instance, on the X11 backend.
gui_launch() {
  local file=$1
  [ -f "$file" ] || gui_abort "no such file to open: $file"
  # Rule "One instance only": require zero before starting. Refusing here also
  # protects an omriss the user has open — this harness never stops that one.
  gui__no_process || gui_abort "an omriss process is already running; close it first"
  gui__no_window  || gui_abort "an Omriss window already exists; close it first"

  # The locale is pinned by unsetting LC_ALL / LC_MESSAGES / LANGUAGE as well as
  # setting LANG: on a machine whose shell sets LC_* to Japanese, LANG alone is
  # overridden and the run silently comes up in the wrong language.
  env -u WAYLAND_DISPLAY -u LC_ALL -u LC_MESSAGES -u LANGUAGE \
    XDG_CONFIG_HOME="$OMRISS_GUI_WORK/xdg/config" \
    XDG_STATE_HOME="$OMRISS_GUI_WORK/xdg/state" \
    XDG_CACHE_HOME="$OMRISS_GUI_WORK/xdg/cache" \
    XDG_DATA_HOME="$OMRISS_GUI_WORK/xdg/data" \
    GDK_BACKEND=x11 DISPLAY="$OMRISS_GUI_DISPLAY" LANG="$OMRISS_GUI_LANG" \
    setsid "$OMRISS_BIN" "$file" >"$OMRISS_GUI_WORK/omriss.log" 2>&1 &

  gui_wait_until "exactly one omriss process" gui__one_process
  GUI_PID=$(pgrep -x omriss)
  # Zero before and exactly one after: the window is provably this run's.
  gui_wait_until "exactly one Omriss window" gui__one_window
  GUI_NIRI_ID=$(gui__omriss_windows)
  GUI_X_WINDOW=""
  # Registration on the accessibility bus can lag the window.
  local rc=0
  python3 -I "$GUI_DIR/atspi_text.py" omriss frame '' >/dev/null 2>&1 || rc=$?
  [ "$rc" -ne 2 ] || gui_abort "omriss never appeared on the accessibility bus"
  # The window and the bus registration both precede the WebView's content, so
  # a scenario's first read could race it (seen on a cold first launch of a
  # fresh build). The toolbar is rendered whatever is open: wait for it.
  # Match the role, not its name: the name is a translated label ("File",
  # "ファイル"), so a name match times out on every non-English run, and it is
  # the vestigial label RFC-060's P-4 will change.
  gui_wait_until "the application's content is in the accessible tree" gui_tree_has "[tool bar]"
  gui_log "launched: zero instances before, pid $GUI_PID, niri window $GUI_NIRI_ID"
}

gui__stop_ours() {
  [ -n "$GUI_PID" ] || return 0
  kill "$GUI_PID" 2>/dev/null || true
  local deadline=$((SECONDS + OMRISS_GUI_TIMEOUT))
  while kill -0 "$GUI_PID" 2>/dev/null && [ "$SECONDS" -lt "$deadline" ]; do sleep 0.2; done
  GUI_PID=""
  GUI_NIRI_ID=""
  GUI_X_WINDOW=""
}

# gui_quit — stop this run's instance and assert none remains.
gui_quit() {
  gui__stop_ours
  gui_wait_until "no omriss process" gui__no_process
  gui_wait_until "no Omriss window" gui__no_window
  gui_log "stopped: zero instances after"
}

# ── reading the application ──────────────────────────────────────────────────

gui_tree() { python3 -I "$GUI_DIR/atspi_probe.py" omriss; }

# gui_tree_has TEXT — the accessible tree contains TEXT (fixed string).
gui_tree_has() { local t; t=$(gui_tree) || return 1; grep -qF -- "$1" <<<"$t"; }

# gui_text ROLE NAME_PREFIX — the full text of a node (see atspi_text.py).
gui_text() { python3 -I "$GUI_DIR/atspi_text.py" omriss "$1" "$2"; }

# gui_text_contains ROLE NAME_PREFIX TEXT
gui_text_contains() { local t; t=$(gui_text "$1" "$2" 2>/dev/null) || return 1; grep -qF -- "$3" <<<"$t"; }

# gui_sha FILE — the evidence for "unchanged" (rule: prefer byte comparison).
gui_sha() { sha256sum "$1" | cut -d' ' -f1; }

# gui_bytes FILE — the file's bytes with line endings visible, for a log.
gui_bytes() {
  python3 -I -c 'import sys; print(repr(open(sys.argv[1], "rb").read().decode("utf-8")))' "$1"
}

# gui_shot NAME — a capture of this run's window into the evidence directory.
# Only where a picture is itself the evidence (rule: "Evidence answers a question").
gui_shot() {
  local path="$OMRISS_GUI_EVIDENCE/$1.png"
  rm -f "$path"
  niri msg action screenshot-window --id "$GUI_NIRI_ID" --path "$path" >/dev/null
  gui_wait_until "screenshot written: $path" test -s "$path"
  gui_log "captured $path"
}

# ── acting on the application ────────────────────────────────────────────────

# gui_press ROLE NAME_PREFIX — invoke a node's default action through AT-SPI,
# as an assistive technology would. Follow it with gui_wait_until on the effect
# you expect: a press that "worked" but changed nothing is the failure to catch.
gui_press() {
  python3 -W ignore -I "$GUI_DIR/atspi_act.py" omriss "$1" "$2" >/dev/null 2>&1 \
    || gui_abort "nothing to press: $1 '$2'"
}

# gui_focus_window — give this run's window keyboard focus, and verify it got
# it, before any keystroke (rule: input must reach the window under test).
gui_focus_window() {
  niri msg action focus-window --id "$GUI_NIRI_ID" >/dev/null
  local focused
  focused=$(niri msg --json focused-window | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["id"])')
  [ "$focused" = "$GUI_NIRI_ID" ] || gui_abort "focus is on window $focused, not $GUI_NIRI_ID"
}

# gui__node_focused ROLE NAME_PREFIX — the tree reports that node as focused.
gui__node_focused() {
  local t; t=$(gui_tree) || return 1
  grep -F -- "[$1] '$2" <<<"$t" | grep -q 'focused'
}

# gui_focus_node ROLE NAME_PREFIX — move keyboard focus to a node through AT-SPI
# and assert that it arrived.
gui_focus_node() {
  python3 -W ignore -I "$GUI_DIR/atspi_act.py" omriss "$1" "$2" --focus >/dev/null 2>&1 \
    || gui_abort "nothing to focus: $1 '$2'"
  gui_wait_until "$1 '$2' has focus" gui__node_focused "$1" "$2"
}

# gui_key KEY... — xdotool key names, e.g. `gui_key ctrl+s`.
gui_key() {
  gui_require_input
  gui_focus_window
  DISPLAY=$OMRISS_GUI_DISPLAY xdotool key --window "$GUI_X_WINDOW" "$@"
}

# gui_type_into ROLE NAME_PREFIX TEXT — replace the field's content with TEXT
# and assert the text actually landed. A script that types and never checks
# produced a confident, meaningless run in RFC-071's evidence; this one cannot.
gui_type_into() {
  local role=$1 name=$2 text=$3
  gui_require_input
  gui_focus_node "$role" "$name"
  gui_focus_window
  DISPLAY=$OMRISS_GUI_DISPLAY xdotool key --window "$GUI_X_WINDOW" ctrl+a
  DISPLAY=$OMRISS_GUI_DISPLAY xdotool type --window "$GUI_X_WINDOW" --delay 40 -- "$text"
  gui_wait_until "typed text reached $role '$name'" gui_text_contains "$role" "$name" "$text"
}
