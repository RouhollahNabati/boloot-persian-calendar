%global debug_package %{nil}

Name:           boloot-calendar
Version:        1.0.0
Release:        1%{?dist}
Summary:        Persian calendar system for Linux — boloot.ir

License:        GPL-3.0-or-later
URL:            https://boloot.ir
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rustc
BuildRequires:  libicu-devel
BuildRequires:  pkg-config
BuildRequires:  python3
Requires:       python3-gobject
Requires:       gtk4
Requires:       libadwaita
Requires:       libnotify
Requires:       dbus
Requires:       polkit
Requires:       util-linux
Requires(post): %{_bindir}/glib-compile-schemas
Requires(postun): %{_bindir}/glib-compile-schemas

%description
BOLOOT Persian Calendar integrates Jalali calendar, Islamic prayer times,
and official holidays for Iran, Afghanistan, and Tajikistan on Linux desktops.
Includes D-Bus daemon, CLI, GTK4 settings app, GNOME Shell extension,
KDE plasmoid scaffold, and XFCE panel helper.

%prep
%autosetup -n boloot-calendar-%{version}

%build
make build

%install
make install-only PREFIX=%{_prefix} DESTDIR=%{buildroot}

%post
%systemd_post boloot-calendard.service
if [ -x %{_datadir}/boloot-calendar/scripts/enable-gdm-extension.sh ]; then
    %{_datadir}/boloot-calendar/scripts/enable-gdm-extension.sh || :
fi
if command -v dconf >/dev/null 2>&1; then
    dconf update || :
fi

%preun
%systemd_preun boloot-calendard.service

%postun
%systemd_postun_with_restart boloot-calendard.service

%files
%license README.md
%{_bindir}/boloot-calendard
%{_bindir}/boloot-calendar-ctl
%{_bindir}/boloot-calendar-settings
%{_libexecdir}/boloot-calendar-apply-system
%{_datadir}/boloot-calendar/
%{_unitdir}/boloot-calendard.service
%{_sysconfdir}/profile.d/boloot-calendar.sh
%{_sysconfdir}/dconf/db/gdm.d/00-boloot-calendar
%{_sysconfdir}/dconf/db/local.d/00-boloot-calendar
%{_sysconfdir}/boloot-calendar/
%{_datadir}/gnome-shell/extensions/boloot-calendar@boloot.ir/
%{_datadir}/dbus-1/system-services/org.boloot.Calendar.service
%{_datadir}/dbus-1/system.d/org.boloot.Calendar.conf
%{_datadir}/dbus-1/interfaces/org.boloot.Calendar.xml
%{_datadir}/polkit-1/actions/org.boloot.calendar.apply-system.policy
%{_datadir}/applications/org.boloot.Calendar.desktop
%{_datadir}/plasma/plasmoids/org.boloot.calendar/

%changelog
* Wed Jul 08 2026 BOLOOT <info@boloot.ir> - 1.0.0-1
- Initial RPM package
