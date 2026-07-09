#!/usr/bin/env bash
# Enable BOLOOT GNOME extension on the GDM login screen.
set -euo pipefail

UUID="boloot-calendar@boloot.ir"
GDM_SYSTEM_DCONF="/etc/dconf/db/gdm.d/00-boloot-calendar"
GDM_GREETER_DCONF="/usr/share/gdm/dconf/50-boloot-calendar"
GREETER_DEFAULTS="/var/lib/gdm3/greeter-dconf-defaults"

DCONF_BODY="[org/gnome/shell]
enabled-extensions=['${UUID}']
disable-user-extensions=false
"

EXT_SRC=""
for prefix in /usr /usr/local "${PREFIX:-}"; do
    [ -n "$prefix" ] || continue
    candidate="${prefix}/share/gnome-shell/extensions/${UUID}"
    if [ -d "$candidate" ] && [ -f "${candidate}/metadata.json" ]; then
        EXT_SRC="$candidate"
        break
    fi
done

if [ -z "$EXT_SRC" ]; then
    echo "GNOME extension not found under /usr or /usr/local (expected ${UUID})" >&2
    echo "Install first: sudo make install" >&2
    exit 1
fi

if ! command -v dconf >/dev/null 2>&1; then
    echo "Error: dconf command not found" >&2
    exit 1
fi

# Fedora/Arch: system-db:gdm profile reads /etc/dconf/db/gdm.d/
install -d -m755 "$(dirname "$GDM_SYSTEM_DCONF")"
printf '%s\n' "$DCONF_BODY" >"$GDM_SYSTEM_DCONF"
dconf update

# Ubuntu/Debian: gdm profile uses file-db:/var/lib/gdm3/greeter-dconf-defaults
# compiled from /usr/share/gdm/dconf/ at GDM start.
if [ -d /usr/share/gdm/dconf ]; then
    install -d -m755 /usr/share/gdm/dconf
    printf '%s\n' "$DCONF_BODY" >"$GDM_GREETER_DCONF"
    if [ -x /usr/share/gdm/generate-config ]; then
        /usr/share/gdm/generate-config
    else
        install -d -m711 -ogdm -ggdm /var/lib/gdm3
        runuser -u gdm -- dconf compile "$GREETER_DEFAULTS" /usr/share/gdm/dconf
        pkill --signal HUP --uid gdm dconf-service 2>/dev/null || true
    fi
fi

echo "GDM login screen extension enabled (${UUID})"
echo "  Extension source:     ${EXT_SRC}"
echo "  system dconf (gdm.d):   ${GDM_SYSTEM_DCONF}"
if [ -f "$GDM_GREETER_DCONF" ]; then
    echo "  Ubuntu greeter dconf:   ${GDM_GREETER_DCONF}"
    echo "  greeter defaults db:  ${GREETER_DEFAULTS}"
fi
echo ""

if [ -f "$GREETER_DEFAULTS" ]; then
    if strings "$GREETER_DEFAULTS" | grep -q "$UUID"; then
        echo "Verified: ${UUID} is present in greeter dconf defaults."
    else
        echo "Warning: ${UUID} not found in ${GREETER_DEFAULTS}." >&2
        echo "         Try: sudo /usr/share/gdm/generate-config" >&2
    fi
fi

echo ""
echo "Reboot or restart GDM for the login screen to pick up changes:"
echo "  sudo systemctl restart gdm"
echo ""
echo "After restart, check greeter journal:"
echo "  journalctl -b | grep -i boloot"
