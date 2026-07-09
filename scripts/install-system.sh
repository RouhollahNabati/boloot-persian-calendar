#!/usr/bin/env bash
# System-wide install: service, GDM login screen, GNOME extension.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

echo "==> Building..."
make build

echo "==> Installing to /usr (requires sudo)..."
sudo make install

echo "==> Stopping legacy user-session unit if present..."
systemctl --user disable --now boloot-calendard.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/boloot-calendard.service"

echo "==> Enabling system service..."
sudo systemctl reload dbus.service 2>/dev/null || true
sudo systemctl daemon-reload
sudo systemctl enable --now boloot-calendard.service

echo "==> GDM login screen + GNOME extension..."
sudo make setup-gdm-extension
make setup-gnome-extension

echo ""
echo "==> Status:"
systemctl status boloot-calendard.service --no-pager || true
busctl status org.boloot.Calendar 2>/dev/null || true
echo ""
echo "Done. Reload GNOME Shell if needed: Alt+F2 → r → Enter"
echo "For login screen: reboot or sudo systemctl restart gdm"
