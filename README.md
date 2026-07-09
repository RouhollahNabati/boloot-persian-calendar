# BOLOOT Persian Calendar

Persian (Jalali) calendar integration for Linux — GNOME and system-wide locale, prayer times, and official holidays for Iran, Afghanistan, and Tajikistan.

**Website:** [boloot.ir](https://boloot.ir)

## Features

- Jalali / Hijri / Gregorian / dual calendar display
- Country profiles: Iran, Afghanistan (Dari/Pashto), Tajikistan
- Official holidays database with **lunar (Hijri) date resolution per year**
- Islamic prayer times (Tehran method for Iran)
- **Prayer and holiday desktop notifications** (`notify-send`)
- Customizable font, size, and colors
- **GNOME calendar popup** (month grid, holidays, prayer times)
- D-Bus API (`org.boloot.Calendar`)
- GNOME Settings desktop entry + GTK4 settings app
- KDE Plasma plasmoid scaffold + XFCE helper script
- deb, Flatpak, RPM, and Arch (PKGBUILD) packaging

## Build

```bash
make build
make test
```

## Install (local)

```bash
sudo make install
sudo systemctl enable --now boloot-calendard.service
sudo make setup-gdm-extension   # login screen (GDM)
gnome-extensions enable boloot-calendar@boloot.ir
```

## Install by distribution

See [packaging/README.md](packaging/README.md) for full details.

| Distribution | Command |
|--------------|---------|
| Debian / Ubuntu | `make package-deb` then `sudo dpkg -i …/boloot-calendar-*.deb` |
| Fedora / RHEL | `make package-rpm` then `sudo dnf install …/boloot-calendar-*.rpm` |
| Arch Linux | use `packaging/arch/PKGBUILD` with `makepkg -si` |
| Any (Flatpak) | `make package-flatpak` then `flatpak install --user …/boloot-calendar.flatpak` |

Build dependencies: `./scripts/install-deps.sh` (supports apt, dnf, pacman, zypper).

Post-install on all native packages (automatic on deb/rpm; verify with):

```bash
systemctl status boloot-calendard.service
gnome-extensions enable boloot-calendar@boloot.ir   # GNOME (also enabled via dconf defaults)
```

For Flatpak, also run once:

```bash
flatpak run --command=boloot-calendar-host-setup org.boloot.Calendar
```

## CLI

```bash
boloot-calendar-ctl status
boloot-calendar-ctl date
boloot-calendar-ctl prayer --city tehran
boloot-calendar-ctl holidays --year 1404 --month 1
boloot-calendar-ctl holidays-today
boloot-calendar-ctl preview
boloot-calendar-ctl month --year 1405 --month 3
boloot-calendar-ctl export
boloot-calendar-settings
sudo boloot-calendar-ctl apply-system   # system-wide LC_TIME hook (Polkit)
```

## Settings UI

```bash
boloot-calendar-settings
```

Requires GTK4 and libadwaita (`gir1.2-adw-1` on Ubuntu).

## D-Bus API

Service: `org.boloot.Calendar`  
Path: `/org/boloot/Calendar`

| Method | Returns |
|--------|---------|
| `GetDate` | Today's formatted date |
| `GetTopBarText` | Top bar display string |
| `GetMonthView` | JSON month grid (year, month; 0 = current) |
| `GetCalendarView` | JSON calendar view |
| `GetPrayerTimes` | JSON prayer schedule |
| `GetHolidaysToday` | JSON holiday list |
| `GetSettings` | JSON config |
| `SetSettings` | Accepts JSON config |
| `Reload` | Reload from disk |

Signal: `SettingsChanged` — emitted after `SetSettings`.

## Configuration

| File | Purpose |
|------|---------|
| `~/.config/boloot-calendar/config.toml` | Per-user settings (overrides system defaults) |
| `/etc/boloot-calendar/config.toml` | System defaults (GDM login screen + new users) |
| `/usr/share/boloot-calendar/config.toml.example` | Copy source for administrators |

After editing the system file:

```bash
sudo systemctl reload boloot-calendard.service
```

## Project layout

```
core/           Rust library (calendar, prayer, holidays)
daemon/         boloot-calendard D-Bus service
cli/            boloot-calendar-ctl
data/           holidays and city locations
gnome-shell/    GNOME extension
settings/       GTK4 settings app
packaging/      debian, flatpak, rpm, arch (PKGBUILD)
```

## Support / Donate

If BOLOOT is part of your daily routine, a small gift helps keep the Persian calendar free and improving for everyone.

**USDT (TRC20 only)**

`TQh9Sge2aNKfW4S9GBAA7iCEeyzvq6kugg`

![USDT TRC20 QR](data/donate/usdt-trc20-qr.png)

**Bitcoin (BTC)**

`bc1qhd0gjehjhc5glpjeg2w70yyd2vapjdjf4dgkxe`

![Bitcoin QR](data/donate/btc-qr.png)

> For USDT, send only on the **TRC20** network. Sending on another network may result in permanent loss.

## License

GPL-3.0-or-later
