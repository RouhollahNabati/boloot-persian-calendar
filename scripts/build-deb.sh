#!/bin/sh
# Build a .deb from packaging/debian/ using the current source tree.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BUILD_DIR="${TMPDIR:-/tmp}/boloot-calendar-deb-build"
VERSION="${BOLOOT_VERSION:-1.0.0}"
SRC="$BUILD_DIR/boloot-calendar-$VERSION"

rm -rf "$BUILD_DIR"
mkdir -p "$SRC"
tar -C "$ROOT" \
    --exclude='./target' \
    --exclude='./.git' \
    --exclude='./.venv-i18n' \
    --exclude='./.cursor' \
    -cf - . | tar -C "$SRC" -xf -

cp -a "$ROOT/packaging/debian" "$SRC/debian"
cd "$SRC"
dpkg-buildpackage -us -uc -b
echo ""
echo "Packages written to: $BUILD_DIR"
