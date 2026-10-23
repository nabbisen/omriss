"""Invoke the default action of one node in a running application's AT-SPI
tree, exactly as an assistive technology would ("press this button").

    python3 -I atspi_act.py omriss button 'Abstract'
    python3 -I atspi_act.py omriss button 'Open…' --focus      # grab focus only

Matches the first node whose role equals ROLE and whose accessible name
starts with NAME. Exits non-zero if nothing matches. A QA tool, not shipped.
"""
import sys

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi  # noqa: E402

pos = [a for a in sys.argv[1:] if not a.startswith("--")]
app_name, role, name = pos[0], pos[1], pos[2]
focus_only = "--focus" in sys.argv
Atspi.init()


def walk(node):
    yield node
    for i in range(node.get_child_count()):
        child = node.get_child_at_index(i)
        if child is not None:
            yield from walk(child)


desktop = Atspi.get_desktop(0)
for i in range(desktop.get_child_count()):
    app = desktop.get_child_at_index(i)
    if app is None or app_name.lower() not in (app.get_name() or "").lower():
        continue
    for node in walk(app):
        if node.get_role_name() == role and (node.get_name() or "").startswith(name):
            if focus_only:
                ok = Atspi.Component.grab_focus(node)
                print(f"grab_focus -> {ok}")
            else:
                n = node.get_n_actions()
                acts = [node.get_action_name(k) for k in range(n)]
                print(f"actions: {acts}")
                print(f"do_action(0) -> {node.do_action(0)}")
            sys.exit(0)
print("NO MATCH")
sys.exit(1)
