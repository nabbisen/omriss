#!/usr/bin/env bash
# Exercise the GUI harness against a copy of a tracked fixture, so a task can
# start from "the harness works" instead of finding out halfway through a run.
#
#     bash scripts/gui/selftest.sh
#
# Steps that need keyboard input run only when xdotool is installed, and are
# reported as SKIPPED otherwise — never as passed.
set -uo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

fixture="$GUI_REPO/crates/core/tests/fixtures/academic-paper.md"
tracked_before=$(gui_sha "$fixture")

gui_init
copy="$OMRISS_GUI_WORK/selftest.md"
cp "$fixture" "$copy"
original=$(gui_sha "$copy")

echo "launch and read"
gui_launch "$copy"
gui_check "the Document Map is in the accessible tree" gui_tree_has "[landmark] 'Document Map'"
gui_check "the status bar is readable" gui_text 'status bar' ''
gui_check "nothing is focused on a fresh launch (gui_focused prints nothing)" gui_focused_none

echo "act through the accessibility API"
gui_press button 'Abstract'
gui_wait_until "Abstract becomes the focused section" gui_tree_has "[heading] 'Abstract'"
gui_focus_node entry 'Edit section text'
gui_check "exactly one node is focused, and it is the editor" gui_focused_is entry 'Edit section text'
gui_check "an ancestor is not mistaken for the focus" bash -c '! grep -q "document web" <<<"$1"' _ "$(gui_focused)"

echo "keyboard input"
if command -v xdotool >/dev/null 2>&1; then
  gui_type_into entry 'Edit section text' 'harness selftest'
  gui_press button 'Save'
  gui_wait_until "the typed text is on disk" grep -qF 'harness selftest' "$copy"
  gui_press button 'Undo'
  gui_press button 'Save'
  gui_wait_until "undo and save restore the original bytes" \
    bash -c '[ "$(sha256sum "$1" | cut -d" " -f1)" = "$2" ]' _ "$copy" "$original"
  gui_log "ok: edit, save, undo, save is byte-exact"
else
  gui_log "SKIPPED: keyboard input — xdotool is not installed"
fi

gui_shot selftest
gui_quit

gui_check "the tracked fixture was never touched" [ "$(gui_sha "$fixture")" = "$tracked_before" ]
echo "selftest: done"
