#!/bin/sh
# Build RPM from packaging/rpm/boloot-calendar.spec
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
VERSION="${BOLOOT_VERSION:-1.0.0}"
BUILD_DIR="${TMPDIR:-/tmp}/boloot-calendar-rpm-build"
TARBALL="$BUILD_DIR/boloot-calendar-$VERSION.tar.gz"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/SOURCES" "$BUILD_DIR/SPECS" "$BUILD_DIR/BUILD" "$BUILD_DIR/RPMS" "$BUILD_DIR/SRPMS"
tar -C "$ROOT" -czf "$TARBALL" \
    --transform "s,^,boloot-calendar-$VERSION/," \
    --exclude='./target' \
    --exclude='./.git' \
    --exclude='./.venv-i18n' \
    --exclude='./.cursor' \
    .

cp "$TARBALL" "$BUILD_DIR/SOURCES/"
cp "$ROOT/packaging/rpm/boloot-calendar.spec" "$BUILD_DIR/SPECS/"
rpmbuild --define "_topdir $BUILD_DIR" \
         --define "version $VERSION" \
         -ba "$BUILD_DIR/SPECS/boloot-calendar.spec"
echo ""
echo "RPMs written to: $BUILD_DIR/RPMS/"
