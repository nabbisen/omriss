"""Walk the AT-SPI tree of a running application and report what a screen
reader would be able to see. Read-only: it registers as a client, nothing more.
"""
import sys
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi  # noqa: E402

TARGET = sys.argv[1] if len(sys.argv) > 1 else "omriss"
DEADLINE = time.time() + 30


def find_app():
    desktop = Atspi.get_desktop(0)
    names = []
    for i in range(desktop.get_child_count()):
        try:
            app = desktop.get_child_at_index(i)
            if app is None:
                continue
            name = app.get_name() or "(unnamed)"
            names.append(name)
            if TARGET.lower() in name.lower():
                return app, names
        except Exception as exc:  # noqa: BLE001
            names.append(f"(error: {exc})")
    return None, names


def dump(node, depth=0, limit=400, counter=None):
    if counter is None:
        counter = {"n": 0}
    if counter["n"] >= limit:
        return counter
    counter["n"] += 1
    try:
        role = node.get_role_name()
    except Exception:  # noqa: BLE001
        role = "?"
    try:
        name = node.get_name() or ""
    except Exception:  # noqa: BLE001
        name = "?"
    try:
        states = sorted(
            s.value_nick for s in node.get_state_set().get_states()
        )
    except Exception:  # noqa: BLE001
        states = []
    keep = [s for s in states if s in ("focusable", "focused", "editable", "selected")]
    print(f"{'  ' * depth}[{role}] {name!r}{(' ' + ','.join(keep)) if keep else ''}")
    try:
        n = node.get_child_count()
    except Exception:  # noqa: BLE001
        n = 0
    for i in range(n):
        try:
            child = node.get_child_at_index(i)
        except Exception:  # noqa: BLE001
            continue
        if child is not None:
            dump(child, depth + 1, limit, counter)
    return counter


Atspi.init()
app = None
seen = []
while time.time() < DEADLINE:
    app, seen = find_app()
    if app is not None:
        break
    time.sleep(0.5)

if app is None:
    print(f"NOT FOUND on the a11y bus: {TARGET!r}")
    print("applications currently registered:")
    for n in seen:
        print(f"  - {n}")
    sys.exit(1)

print(f"FOUND on the a11y bus: {app.get_name()!r} [{app.get_role_name()}]")
print(f"toolkit: {app.get_toolkit_name()} {app.get_toolkit_version()}")
print("--- accessible tree ---")
c = dump(app)
print(f"--- {c['n']} nodes printed ---")
