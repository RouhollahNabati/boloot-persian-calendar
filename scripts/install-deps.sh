#!/bin/sh
# Runtime and build dependencies for BOLOOT Persian Calendar
set -eu

install_apt() {
    sudo apt-get update
    sudo apt-get install -y \
        build-essential curl pkg-config libicu-dev \
        python3 python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 libadwaita-1-dev \
        libgtk-4-dev libnotify-bin dbus-user-session policykit-1 \
        debhelper cargo rustc
}

install_dnf() {
    sudo dnf install -y \
        gcc make curl pkg-config libicu-devel \
        python3 python3-gobject gtk4 libadwaita libnotify dbus polkit \
        cargo rust rpm-build
}

install_pacman() {
    sudo pacman -Sy --needed --noconfirm \
        base-devel curl pkg-config icu \
        python python-gobject gtk4 libadwaita libnotify dbus polkit \
        cargo rust
}

install_zypper() {
    sudo zypper install -y \
        patterns-devel-base-devel_basis curl pkg-config libicu-devel \
        python3 python3-gobject gtk4 libadwaita libnotify dbus-1 polkit \
        cargo rust
}

if command -v apt-get >/dev/null 2>&1; then
    install_apt
elif command -v dnf >/dev/null 2>&1; then
    install_dnf
elif command -v pacman >/dev/null 2>&1; then
    install_pacman
elif command -v zypper >/dev/null 2>&1; then
    install_zypper
else
    echo "Unsupported package manager."
    echo "Install manually: Rust, ICU, GTK4, libadwaita, python3-gi, libnotify, dbus, polkit."
    exit 1
fi

echo ""
if ! command -v cargo >/dev/null 2>&1; then
    echo "If Rust is not installed:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \"\$HOME/.cargo/env\""
fi
echo "Done. Build with: make build"
