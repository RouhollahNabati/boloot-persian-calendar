#!/bin/sh
# Build Flatpak bundle from packaging/flatpak/org.boloot.Calendar.yml
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
MANIFEST="$ROOT/packaging/flatpak/org.boloot.Calendar.yml"
REPO="${TMPDIR:-/tmp}/boloot-calendar-flatpak-repo"
BUNDLE="${TMPDIR:-/tmp}/boloot-calendar.flatpak"

if ! command -v flatpak-builder >/dev/null 2>&1; then
    echo "Error: flatpak-builder not found."
    echo "  Debian/Ubuntu: sudo apt install flatpak-builder"
    echo "  Fedora:        sudo dnf install flatpak-builder"
    exit 1
fi

rm -rf "$REPO" "$BUNDLE"
flatpak-builder --force-clean --repo="$REPO" "$REPO/build" "$MANIFEST"
flatpak build-bundle "$REPO" "$BUNDLE" org.boloot.Calendar
echo ""
echo "Bundle: $BUNDLE"
echo "Install: flatpak install --user \"$BUNDLE\""
echo "Then run once: flatpak run --command=boloot-calendar-host-setup org.boloot.Calendar"
