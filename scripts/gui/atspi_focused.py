"""Print the focused node(s) of a running application's AT-SPI tree, one per
line as `[role] 'name'`. Prints nothing when no node is focused — that is a
result (RFC-060 F-4: after a dialog closes, nothing is), not an error.

    python3 -I atspi_focused.py omriss [--all]

WebKit reports the *ancestors* of the focused element as focused too: with the
section editor focused the tree says `scroll pane`, `document web` and `entry`
are all focused. "Which node has focus" is the deepest one, so by default only
a focused node with no focused descendant is printed; `--all` prints every
node that reports the state.

Walks the whole tree (unlike `atspi_probe.py`, which caps at 400 nodes for
readability): a focus assertion must not miss a node because it sits late in
the document. Read-only. Exit status: 0 when the application was found, 2 when
it is not on the accessibility bus. A QA tool, not shipped code.
"""
import sys
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi  # noqa: E402

args = [a for a in sys.argv[1:] if not a.startswith("--")]
ALL = "--all" in sys.argv
if len(args) != 1:
    print("usage: atspi_focused.py APP [--all]", file=sys.stderr)
    sys.exit(64)
app_name = args[0]
Atspi.init()


def find_app():
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(i)
        if app is not None and app_name.lower() in (app.get_name() or "").lower():
            return app
    return None


def is_focused(node):
    try:
        return "focused" in {s.value_nick for s in node.get_state_set().get_states()}
    except Exception:  # noqa: BLE001
        return False


def focused_nodes(node):
    """Yield (node, has_focused_descendant) for every focused node."""
    below = False
    for i in range(node.get_child_count()):
        child = node.get_child_at_index(i)
        if child is None:
            continue
        for found, has_below in focused_nodes(child):
            below = True
            yield found, has_below
    if is_focused(node):
        yield node, below


deadline = time.time() + 10
app = find_app()
while app is None and time.time() < deadline:
    time.sleep(0.25)
    app = find_app()
if app is None:
    print(f"NOT ON THE ACCESSIBILITY BUS: {app_name!r}", file=sys.stderr)
    sys.exit(2)

for node, has_focused_descendant in focused_nodes(app):
    if ALL or not has_focused_descendant:
        print(f"[{node.get_role_name()}] '{node.get_name() or ''}'")
