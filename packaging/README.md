# Packaging — BOLOOT Persian Calendar

Install paths and file layout are defined in the root `Makefile` (`make install-only`).
Distribution packages should call that target (or mirror it) so all formats stay in sync.

## Supported formats

| Format | Path | Build command |
|--------|------|---------------|
| Native (any distro) | `Makefile` | `make build && sudo make install` |
| User-local | `Makefile` | `make install-local` |
| `.deb` | `packaging/debian/` | `make package-deb` |
| Flatpak | `packaging/flatpak/` | `make package-flatpak` |
| `.rpm` | `packaging/rpm/` | `make package-rpm` |
| Arch (AUR) | `packaging/arch/PKGBUILD` | copy PKGBUILD + source tarball |

## Dependencies

```bash
./scripts/install-deps.sh
```

Detects `apt`, `dnf`, `pacman`, or `zypper`.

## Debian / Ubuntu

```bash
make package-deb
# or manually:
sudo apt install debhelper cargo rustc libicu-dev
./scripts/build-deb.sh
sudo dpkg -i ../boloot-calendar-core_*.deb ../boloot-calendar-gnome_*.deb
systemctl status boloot-calendard.service
```

Packages: `boloot-calendar-core`, `boloot-calendar-gnome`, `boloot-calendar` (meta).

## Fedora / RHEL / openSUSE

```bash
make package-rpm
sudo dnf install /tmp/boloot-calendar-rpm-build/RPMS/*/boloot-calendar-*.rpm
systemctl status boloot-calendard.service
```

For openSUSE, use `zypper install` instead of `dnf`.

## Arch Linux

Copy `packaging/arch/PKGBUILD` into an AUR build directory, set the source tarball URL
(or `sha256sums`), then:

```bash
makepkg -si
```

## Flatpak (all distributions)

```bash
make package-flatpak
flatpak install --user /tmp/boloot-calendar.flatpak
flatpak run --command=boloot-calendar-host-setup org.boloot.Calendar
```

The host-setup command copies the GNOME extension to `~/.local` and registers
session D-Bus activation via Flatpak.

## Post-install (all native packages)

The system service starts automatically on install. Verify:

```bash
systemctl status boloot-calendard.service
gnome-extensions enable boloot-calendar@boloot.ir   # GNOME only, if needed
boloot-calendar-settings                            # optional GTK4 settings
```
