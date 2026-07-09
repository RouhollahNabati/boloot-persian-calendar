#!/usr/bin/env python3
"""XFCE panel helper — BOLOOT Persian Calendar (boloot.ir)."""

import subprocess
import sys


def fetch_text() -> str:
    try:
        import dbus

        bus = dbus.SystemBus()
        proxy = bus.get_object("org.boloot.Calendar", "/org/boloot/Calendar")
        iface = dbus.Interface(proxy, "org.boloot.Calendar")
        return str(iface.GetTopBarText())
    except Exception:
        pass

    try:
        return subprocess.check_output(
            ["boloot-calendar-ctl", "preview"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except Exception:
        return "BOLOOT Persian Calendar"


if __name__ == "__main__":
    print(fetch_text())
