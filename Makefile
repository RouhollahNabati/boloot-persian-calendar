PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
LIBEXECDIR = $(PREFIX)/libexec
DATADIR = $(PREFIX)/share/boloot-calendar
SYSTEMD_SYSTEM_DIR = $(PREFIX)/lib/systemd/system
ifeq ($(PREFIX),/usr)
PROFILE_D = /etc/profile.d
DCONF_LOCAL_DIR = /etc/dconf/db/local.d
SYSTEM_CONFIG_DIR = /etc/boloot-calendar
else
PROFILE_D = $(PREFIX)/etc/profile.d
DCONF_LOCAL_DIR = $(PREFIX)/etc/dconf/db/local.d
SYSTEM_CONFIG_DIR = $(PREFIX)/etc/boloot-calendar
endif
# GDM: Fedora/Arch use /etc/dconf/db/gdm.d; Ubuntu/Debian compile
# /usr/share/gdm/dconf into /var/lib/gdm3/greeter-dconf-defaults.
DCONF_GDM_DIR = /etc/dconf/db/gdm.d
GDM_GREETER_DCONF_DIR = /usr/share/gdm/dconf
GNOME_EXT_DIR = $(PREFIX)/share/gnome-shell/extensions/boloot-calendar@boloot.ir
GNOME_EXT_SYSTEM_DIR = /usr/share/gnome-shell/extensions/boloot-calendar@boloot.ir
DBUS_SYSTEM_SERVICES = $(PREFIX)/share/dbus-1/system-services
DBUS_SYSTEM_CONF = $(PREFIX)/share/dbus-1/system.d
DBUS_ETC_POLICY_DIR = /etc/dbus-1/system.d
DBUS_INTERFACES = $(PREFIX)/share/dbus-1/interfaces
POLKIT_ACTIONS = $(PREFIX)/share/polkit-1/actions
APPLICATIONS = $(PREFIX)/share/applications
KDE_PLASMOID = $(PREFIX)/share/plasma/plasmoids/org.boloot.calendar
XFCE_DIR = $(PREFIX)/share/boloot-calendar/xfce
SCRIPTS_DIR = $(PREFIX)/share/boloot-calendar/scripts

CARGO_FLAGS ?= --release
CRATES = -p boloot-cal-core -p boloot-calendard -p boloot-calendar-ctl

# Use the invoking user's cargo when building (sudo strips PATH).
CARGO ?= $(shell \
	if command -v cargo >/dev/null 2>&1; then command -v cargo; \
	elif [ -x "$(HOME)/.cargo/bin/cargo" ]; then echo "$(HOME)/.cargo/bin/cargo"; \
	elif [ -n "$(SUDO_USER)" ] && [ -x "/home/$(SUDO_USER)/.cargo/bin/cargo" ]; then \
		echo "/home/$(SUDO_USER)/.cargo/bin/cargo"; \
	else echo cargo; fi)

RELEASE_BIN = target/release/boloot-calendard
RELEASE_CTL = target/release/boloot-calendar-ctl

USER_GNOME_EXT = $(HOME)/.local/share/gnome-shell/extensions/boloot-calendar@boloot.ir
WRONG_GNOME_EXT = $(HOME)/.local/share/gnome-shell/extensions/boloot@boloot.ir

.PHONY: all build test install install-only install-local install-extension-local setup-gnome-extension setup-systemd setup-gdm-extension uninstall clean check-build fix-topbar i18n package-deb package-flatpak package-rpm

USER_PREFIX = $(HOME)/.local

all: build

build:
	@test -x "$(CARGO)" || (echo "Error: cargo not found. Install Rust: curl -sSf https://sh.rustup.rs | sh" && exit 1)
	BOLOOT_DATA_DIR="$(CURDIR)/data" "$(CARGO)" build $(CARGO_FLAGS) $(CRATES)

test:
	@test -x "$(CARGO)" || (echo "Error: cargo not found." && exit 1)
	BOLOOT_DATA_DIR="$(CURDIR)/data" "$(CARGO)" test -p boloot-cal-core

i18n:
	python3 scripts/compile-i18n.py

check-build:
	@test -f "$(RELEASE_BIN)" && test -f "$(RELEASE_CTL)" || \
		(echo "Binaries not built. Run 'make build' first (without sudo)." && exit 1)

# Install pre-built binaries only — safe to run with sudo after 'make build'.
install: check-build install-only

install-only: check-build i18n
	install -d $(DESTDIR)$(BINDIR)
	install -d $(DESTDIR)$(LIBEXECDIR)
	install -d $(DESTDIR)$(DATADIR)/data
	install -d $(DESTDIR)$(SYSTEMD_SYSTEM_DIR)
	install -d $(DESTDIR)$(SYSTEM_CONFIG_DIR)
	install -d $(DESTDIR)$(PROFILE_D)
	install -d $(DESTDIR)$(GNOME_EXT_DIR)
	install -d $(DESTDIR)$(DBUS_SYSTEM_SERVICES)
	install -d $(DESTDIR)$(DBUS_SYSTEM_CONF)
	install -d $(DESTDIR)$(DBUS_INTERFACES)
	install -d $(DESTDIR)$(POLKIT_ACTIONS)
	install -d $(DESTDIR)$(APPLICATIONS)
	install -d $(DESTDIR)$(KDE_PLASMOID)/contents/ui
	install -d $(DESTDIR)$(XFCE_DIR)
	install -d $(DESTDIR)$(SCRIPTS_DIR)
	install -d $(DESTDIR)$(DCONF_GDM_DIR)
	install -d $(DESTDIR)$(DCONF_LOCAL_DIR)
	install -m755 $(RELEASE_BIN) $(DESTDIR)$(BINDIR)/
	install -m755 $(RELEASE_CTL) $(DESTDIR)$(BINDIR)/
	install -m755 scripts/boloot-calendar-settings $(DESTDIR)$(BINDIR)/
	install -m755 scripts/boloot-calendar-apply-system $(DESTDIR)$(LIBEXECDIR)/
	install -m755 scripts/enable-gdm-extension.sh $(DESTDIR)$(SCRIPTS_DIR)/
	install -m755 settings/boloot-settings.py $(DESTDIR)$(DATADIR)/
	install -m644 settings/boloot-settings.css $(DESTDIR)$(DATADIR)/
	install -d $(DESTDIR)$(DATADIR)/locale
	if [ -d settings/locale ]; then cp -a settings/locale/. $(DESTDIR)$(DATADIR)/locale/; fi
	install -m755 xfce/boloot-xfce-clock.py $(DESTDIR)$(XFCE_DIR)/
	cp -a data/. $(DESTDIR)$(DATADIR)/data/
	install -m644 data/system-config/config.toml $(DESTDIR)$(SYSTEM_CONFIG_DIR)/
	install -m644 data/system-config/config.toml $(DESTDIR)$(DATADIR)/config.toml.example
	sed 's|@PREFIX@|$(PREFIX)|g' systemd/boloot-calendard.service > $(DESTDIR)$(SYSTEMD_SYSTEM_DIR)/boloot-calendard.service
	install -m644 locale/profile.d/boloot-calendar.sh $(DESTDIR)$(PROFILE_D)/
	install -m644 gnome-shell/metadata.json $(DESTDIR)$(GNOME_EXT_DIR)/
	install -m644 gnome-shell/extension.js $(DESTDIR)$(GNOME_EXT_DIR)/
	install -m644 gnome-shell/stylesheet.css $(DESTDIR)$(GNOME_EXT_DIR)/
	install -d $(DESTDIR)$(GNOME_EXT_DIR)/locale
	if [ -d gnome-shell/locale ]; then cp -a gnome-shell/locale/. $(DESTDIR)$(GNOME_EXT_DIR)/locale/; fi
ifneq ($(PREFIX),/usr)
	install -d $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)
	install -m644 gnome-shell/metadata.json $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)/
	install -m644 gnome-shell/extension.js $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)/
	install -m644 gnome-shell/stylesheet.css $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)/
	install -d $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)/locale
	if [ -d gnome-shell/locale ]; then cp -a gnome-shell/locale/. $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)/locale/; fi
endif
	sed 's|@PREFIX@|$(PREFIX)|g' dbus/org.boloot.Calendar.service > $(DESTDIR)$(DBUS_SYSTEM_SERVICES)/org.boloot.Calendar.service
	install -m644 dbus/org.boloot.Calendar.conf $(DESTDIR)$(DBUS_SYSTEM_CONF)/
ifneq ($(PREFIX),/usr)
	install -d $(DESTDIR)$(DBUS_ETC_POLICY_DIR)
	install -m644 dbus/org.boloot.Calendar.conf $(DESTDIR)$(DBUS_ETC_POLICY_DIR)/
endif
	install -m644 dbus/org.boloot.Calendar.xml $(DESTDIR)$(DBUS_INTERFACES)/
	sed 's|@PREFIX@|$(PREFIX)|g' packaging/polkit/org.boloot.calendar.apply-system.policy > $(DESTDIR)$(POLKIT_ACTIONS)/org.boloot.calendar.apply-system.policy
	install -m644 settings/org.boloot.Calendar.desktop $(DESTDIR)$(APPLICATIONS)/
	install -m644 kde/plasmoid/metadata.json $(DESTDIR)$(KDE_PLASMOID)/
	install -m644 kde/plasmoid/main.qml $(DESTDIR)$(KDE_PLASMOID)/contents/ui/main.qml
	install -m644 data/dconf/gdm/00-boloot-calendar $(DESTDIR)$(DCONF_GDM_DIR)/
	install -d $(DESTDIR)$(GDM_GREETER_DCONF_DIR)
	install -m644 data/dconf/gdm/50-boloot-calendar $(DESTDIR)$(GDM_GREETER_DCONF_DIR)/
	install -m644 data/dconf/user/00-boloot-calendar $(DESTDIR)$(DCONF_LOCAL_DIR)/
ifeq ($(DESTDIR),)
	@if command -v dconf >/dev/null 2>&1; then dconf update || true; fi
endif
	@echo ""
	@echo "Installed to $(PREFIX)"
	@echo "Next: sudo systemctl reload dbus.service"
	@echo "      sudo systemctl enable --now boloot-calendard.service"
	@echo "      make setup-gnome-extension"
	@echo "      sudo make setup-gdm-extension   # login screen"
	@echo "      reboot (or restart gdm) to apply login-screen extension"

# User-local install (no sudo): binaries under ~/.local/bin.
install-local: check-build
	$(MAKE) install-only PREFIX=$(USER_PREFIX)
	@echo ""
	@echo "User-local install complete under $(USER_PREFIX)"
	@echo "System D-Bus service requires a privileged install:"
	@echo "  sudo make install && sudo systemctl enable --now boloot-calendard.service"
	@echo "Then reload GNOME Shell: Alt+F2 → r → Enter"

# GNOME loads user extensions from ~/.local (not /usr/local).
install-extension-local: i18n
	install -d $(USER_GNOME_EXT)
	@for f in metadata.json extension.js stylesheet.css; do \
		src="gnome-shell/$$f"; dst="$(USER_GNOME_EXT)/$$f"; \
		if [ "$$(readlink -f "$$src" 2>/dev/null)" != "$$(readlink -f "$$dst" 2>/dev/null)" ]; then \
			cp -f "$$src" "$$dst"; \
		fi; \
	done
	@if [ -d gnome-shell/locale ]; then \
		mkdir -p "$(USER_GNOME_EXT)/locale"; \
		cp -a gnome-shell/locale/. "$(USER_GNOME_EXT)/locale/"; \
	fi
	@echo "Extension copied to $(USER_GNOME_EXT)"
	@echo ""
	@echo "IMPORTANT: GNOME Shell caches extension.js for the whole session."
	@echo "After updating extension files you MUST restart the shell:"
	@echo "  Alt+F2 → r → Enter   (or log out and back in)"
	@echo "disable/enable alone does NOT reload JavaScript changes."

# Enable extension on current GNOME Shell (handles Shell 49 version gate).
setup-gnome-extension: i18n
	rm -f /tmp/boloot-calendar@boloot.ir.zip
	cd gnome-shell && zip -j /tmp/boloot-calendar@boloot.ir.zip metadata.json extension.js stylesheet.css
	cd gnome-shell && zip -r /tmp/boloot-calendar@boloot.ir.zip locale
	-rm -rf $(WRONG_GNOME_EXT)
	gnome-extensions install --force /tmp/boloot-calendar@boloot.ir.zip
	bash scripts/enable-gnome-extension.sh
	@echo ""
	@bash scripts/check-gnome-extension.sh || true
	@echo ""
	@echo "If check shows BLOCKER: log out completely and log back in once."

fix-topbar:
	bash scripts/fix-top-bar.sh

# Enable and start the system-wide D-Bus daemon (requires root).
setup-systemd:
	systemctl daemon-reload
	systemctl enable --now boloot-calendard.service
	@echo "Daemon status:"
	@systemctl --no-pager status boloot-calendard.service || true

setup-gdm-extension:
	bash scripts/enable-gdm-extension.sh

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/boloot-calendard
	rm -f $(DESTDIR)$(BINDIR)/boloot-calendar-ctl
	rm -f $(DESTDIR)$(BINDIR)/boloot-calendar-settings
	rm -f $(DESTDIR)$(LIBEXECDIR)/boloot-calendar-apply-system
	rm -rf $(DESTDIR)$(DATADIR)
	rm -f $(DESTDIR)$(SYSTEMD_SYSTEM_DIR)/boloot-calendard.service
	rm -f $(DESTDIR)$(PROFILE_D)/boloot-calendar.sh
	rm -rf $(DESTDIR)$(GNOME_EXT_DIR)
ifneq ($(PREFIX),/usr)
	rm -rf $(DESTDIR)$(GNOME_EXT_SYSTEM_DIR)
endif
	rm -f $(DESTDIR)$(DBUS_SYSTEM_SERVICES)/org.boloot.Calendar.service
	rm -f $(DESTDIR)$(DBUS_SYSTEM_CONF)/org.boloot.Calendar.conf
ifneq ($(PREFIX),/usr)
	rm -f $(DESTDIR)$(DBUS_ETC_POLICY_DIR)/org.boloot.Calendar.conf
endif
	rm -f $(DESTDIR)$(DBUS_INTERFACES)/org.boloot.Calendar.xml
	rm -f $(DESTDIR)$(POLKIT_ACTIONS)/org.boloot.calendar.apply-system.policy
	rm -f $(DESTDIR)$(APPLICATIONS)/org.boloot.Calendar.desktop
	rm -rf $(DESTDIR)$(KDE_PLASMOID)
	rm -f $(DESTDIR)$(SYSTEM_CONFIG_DIR)/config.toml
	rm -f $(DESTDIR)$(DATADIR)/config.toml.example
	rm -f $(DESTDIR)$(DCONF_GDM_DIR)/00-boloot-calendar
	rm -f $(DESTDIR)$(GDM_GREETER_DCONF_DIR)/50-boloot-calendar
	rm -f $(DESTDIR)$(DCONF_LOCAL_DIR)/00-boloot-calendar

clean:
	@if [ -x "$(CARGO)" ]; then "$(CARGO)" clean; else rm -rf target; fi

package-deb:
	bash scripts/build-deb.sh

package-flatpak:
	bash scripts/build-flatpak.sh

package-rpm:
	bash scripts/build-rpm.sh
