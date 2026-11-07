"""Print the full text of one node in a running application's AT-SPI tree.

    python3 -I atspi_text.py omriss entry 'Edit section text'
    python3 -I atspi_text.py omriss 'status bar' ''

Matches the first node whose role equals ROLE and whose accessible name starts
with NAME — the same rule as `atspi_act.py`, so the two always agree about which
node is meant. An empty NAME matches the first node of that role.

Unlike `atspi_probe.py`, which truncates text for a readable tree, this prints
the text whole: it is what a script asserts against.

Exit status: 0 when the node was found (its text may be empty — that is a
result, not an error), 1 when no node matched, 2 when the application is not on
the accessibility bus. Read-only. A QA tool, not shipped code.
"""
import sys
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi  # noqa: E402

pos = [a for a in sys.argv[1:] if not a.startswith("--")]
if len(pos) != 3:
    print(__doc__.strip().splitlines()[0], file=sys.stderr)
    print("usage: atspi_text.py APP ROLE NAME_PREFIX", file=sys.stderr)
    sys.exit(64)
app_name, role, name = pos
Atspi.init()


def walk(node):
    yield node
    for i in range(node.get_child_count()):
        child = node.get_child_at_index(i)
        if child is not None:
            yield from walk(child)


def find_app():
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(i)
        if app is not None and app_name.lower() in (app.get_name() or "").lower():
            return app
    return None


# The application can take a moment to register after launch.
deadline = time.time() + 10
app = find_app()
while app is None and time.time() < deadline:
    time.sleep(0.25)
    app = find_app()
if app is None:
    print(f"NOT ON THE ACCESSIBILITY BUS: {app_name!r}", file=sys.stderr)
    sys.exit(2)

for node in walk(app):
    if node.get_role_name() == role and (node.get_name() or "").startswith(name):
        try:
            n_chars = Atspi.Text.get_character_count(node)
            text = Atspi.Text.get_text(node, 0, n_chars) if n_chars else ""
        except Exception:  # noqa: BLE001 — a node without a text interface has no text
            text = ""
        sys.stdout.write(text)
        sys.exit(0)

print(f"NO MATCH: role={role!r} name starts with {name!r}", file=sys.stderr)
sys.exit(1)
