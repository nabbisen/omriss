"""Log the AT-SPI events a screen reader would receive while you drive the app.

    python3 -I atspi_events.py SECONDS > events.txt &

Listens for text-changed, children-changed and state-changed events from any
application and prints role, accessible name and the text involved. Read-only.
A QA tool, not shipped code.
"""
import sys
import time

import gi

gi.require_version("Atspi", "2.0")
gi.require_version("GLib", "2.0")
from gi.repository import Atspi, GLib  # noqa: E402

SECONDS = float(sys.argv[1]) if len(sys.argv) > 1 else 20
T0 = time.time()


def safe(fn, default=""):
    try:
        return fn()
    except Exception:  # noqa: BLE001
        return default


def on_event(ev):
    # Events from nodes that vanish before we can query them (the DOM node that
    # carried the event was replaced) are still evidence: log them, degraded.
    try:
        src = ev.source
        app = safe(lambda: src.get_application().get_name(), "?")
        if app != "?" and "omriss" not in app.lower():
            return
        if str(ev.type).endswith(":defunct"):
            return
        role = safe(src.get_role_name, "?")
        name = safe(lambda: src.get_name() or "", "?")
        detail = ev.any_data
        if isinstance(detail, str):
            detail = detail[:120]
        elif detail is not None and hasattr(detail, "get_role_name"):
            detail = f"<{detail.get_role_name()} {detail.get_name()!r}>"
        live = ""
        try:
            attrs = src.get_attributes() or {}
            if attrs.get("container-live") or attrs.get("live"):
                live = f" live={attrs.get('container-live') or attrs.get('live')}"
        except Exception:  # noqa: BLE001
            pass
        print(f"{time.time() - T0:6.2f}s {ev.type} [{role}] {name!r}{live} "
              f"detail1={ev.detail1} data={detail!r}", flush=True)
    except Exception as exc:  # noqa: BLE001
        print(f"{time.time() - T0:6.2f}s {getattr(ev, 'type', '?')} (error: {exc})", flush=True)


Atspi.init()
listener = Atspi.EventListener.new(on_event)
for kind in ("object:text-changed", "object:text-changed:insert", "object:text-changed:delete",
             "object:property-change", "object:state-changed", "object:text-caret-moved",
             "object:children-changed:add", "object:children-changed:remove",
             "object:state-changed:focused", "object:announcement"):
    try:
        listener.register(kind)
    except Exception as exc:  # noqa: BLE001
        print(f"cannot register {kind}: {exc}", flush=True)
loop = GLib.MainLoop()
GLib.timeout_add(int(SECONDS * 1000), loop.quit)
loop.run()
print("--- listener done ---")
