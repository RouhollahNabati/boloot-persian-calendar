#!/usr/bin/env python3
"""BOLOOT Persian Calendar settings panel (GTK4 + libadwaita)."""

APP_NAME = "BOLOOT Persian Calendar"
WEBSITE = "https://boloot.ir"
WEBSITE_LABEL = "boloot.ir"
DONATE_USDT_TRC20 = "TQh9Sge2aNKfW4S9GBAA7iCEeyzvq6kugg"
DONATE_BTC = "bc1qhd0gjehjhc5glpjeg2w70yyd2vapjdjf4dgkxe"
DBUS_NAME = "org.boloot.Calendar"
DBUS_PATH = "/org/boloot/Calendar"
DBUS_IFACE = "org.boloot.Calendar"
SERVICE_UNIT = "boloot-calendard.service"

import json
import locale as locale_module
import os
import subprocess
import sys
import shutil
import gettext
import time
from pathlib import Path

try:
    import gi

    gi.require_version("Gtk", "4.0")
    gi.require_version("Adw", "1")
    gi.require_version("Gdk", "4.0")
    from gi.repository import Adw, Gdk, Gio, GLib, Gtk
except ImportError as exc:
    print("BOLOOT Persian Calendar — GTK4/libadwaita not available.", file=sys.stderr)
    print(f"Import error: {exc}", file=sys.stderr)
    print("", file=sys.stderr)
    print("Install on Ubuntu/Debian:", file=sys.stderr)
    print(
        "  sudo apt install python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 libadwaita-1-dev",
        file=sys.stderr,
    )
    sys.exit(1)

HAS_ADW_COLOR_ROW = hasattr(Adw, "ColorRow")

# Boloot custom palette (light) — kept in sync with core/src/colors.rs
DEFAULT_TEXT_COLOR = "#1c1917"
DEFAULT_BG_COLOR = "#faf8f5"
DEFAULT_HOLIDAY_COLOR = "#a61e2e"
DEFAULT_TODAY_COLOR = "#0f6e9c"
DEFAULT_PRAYER_COLOR = "#1a7a4c"

CTL = "boloot-calendar-ctl"
SAVE_DEBOUNCE_MS = 600
PREVIEW_DEBOUNCE_MS = 300

COUNTRY_OPTIONS = [
    ("iran", "ایران"),
    ("afghanistan", "افغانستان"),
    ("tajikistan", "تاجیکستان"),
]

LANGUAGE_OPTIONS = [
    ("persian", "فارسی"),
    ("dari", "دری"),
    ("pashto", "پشتو"),
    ("tajik", "تاجیکی"),
]

CALENDAR_OPTIONS = [
    ("jalali", "شمسی (جلالی)"),
    ("hijri", "قمری"),
    ("gregorian", "میلادی"),
    ("dual_jalali_gregorian", "دوگانه (شمسی + میلادی)"),
]

WEEKDAY_OPTIONS = [
    ("saturday", "شنبه"),
    ("sunday", "یکشنبه"),
    ("monday", "دوشنبه"),
    ("tuesday", "سه‌شنبه"),
    ("wednesday", "چهارشنبه"),
    ("thursday", "پنجشنبه"),
    ("friday", "جمعه"),
]

NUMERAL_OPTIONS = [
    ("persian", "اعداد فارسی"),
    ("latin", "اعداد لاتین"),
]

DATE_STYLE_OPTIONS = [
    ("short_slash", "کوتاه"),
    ("long_named", "بلند"),
]

PRAYER_METHOD_OPTIONS = [
    ("tehran", "تهران"),
    ("mwl", "اتحادیه جهانی اسلام"),
    ("karachi", "کراچی"),
    ("isna", "آمریکای شمالی"),
    ("egypt", "مصر"),
]

MADHAB_OPTIONS = [
    ("jafari", "شیعه"),
    ("shafi", "شافعی"),
    ("hanafi", "حنفی"),
]

PRAYER_DISPLAY_OPTIONS = [
    ("next_prayer", "نماز بعدی"),
    ("all_times", "همه اوقات"),
    ("countdown", "شمارش معکوس"),
    ("hidden", "مخفی"),
]

ADHAN_PRESET_FILES = {
    "mansouri": "mansouri.ogg",
    "makkah": "makkah.ogg",
    "madinah": "madinah.ogg",
}

ADHAN_PRESET_OPTIONS = [
    ("mansouri", "منصوری"),
    ("makkah", "مأذون حرم مکی"),
    ("madinah", "مأذون حرم مدنی"),
    ("custom", "فایل سفارشی"),
]

ADHAN_PRAYER_OPTIONS = [
    ("fajr", "فجر"),
    ("sunrise", "طلوع"),
    ("dhuhr", "ظهر"),
    ("asr", "عصر"),
    ("maghrib", "مغرب"),
    ("isha", "عشا"),
]

LANGUAGES_BY_COUNTRY = {
    "iran": ["persian"],
    "afghanistan": ["dari", "pashto"],
    "tajikistan": ["tajik"],
}


def safe_hex_color(value):
    if not isinstance(value, str):
        return None
    if len(value) == 7 and value.startswith("#") and all(c in "0123456789abcdefABCDEF" for c in value[1:]):
        return value
    return None


def setup_gettext():
    script_dir = Path(__file__).resolve().parent
    locale_dir = script_dir / "locale"
    if locale_dir.is_dir():
        gettext.bindtextdomain("boloot-settings", str(locale_dir))
        gettext.textdomain("boloot-settings")
    try:
        loc, _enc = locale_module.getlocale(locale_module.LC_MESSAGES)
        if loc:
            lang = loc.split(".")[0].replace("-", "_")
            gettext.bindtextdomain("boloot-settings", str(locale_dir))
            trans = gettext.translation(
                "boloot-settings", localedir=str(locale_dir), languages=[lang], fallback=True
            )
            trans.install()
    except (OSError, FileNotFoundError):
        pass


setup_gettext()
_ = gettext.gettext


def resolve_ctl():
    script_dir = Path(__file__).resolve().parent
    sibling = script_dir.parent / "bin" / "boloot-calendar-ctl"
    if sibling.is_file():
        return str(sibling)
    return shutil.which("boloot-calendar-ctl") or CTL


def resolve_data_dir():
    env = Path(os.environ["BOLOOT_DATA_DIR"]) if "BOLOOT_DATA_DIR" in os.environ else None
    if env and env.is_dir():
        return env
    script_dir = Path(__file__).resolve().parent
    candidates = (
        Path("/usr/share/boloot-calendar/data"),
        script_dir / "data",
        script_dir.parent / "data",
        Path("./data"),
    )
    for candidate in candidates:
        if candidate.is_dir():
            return candidate
    return script_dir / "data"


def resolve_settings_css():
    script_dir = Path(__file__).resolve().parent
    for candidate in (
        Path("/usr/share/boloot-calendar/boloot-settings.css"),
        script_dir / "boloot-settings.css",
    ):
        if candidate.is_file():
            return candidate
    return None


def resolve_donate_qr(filename):
    data_dir = resolve_data_dir()
    path = data_dir / "donate" / filename
    return path if path.is_file() else None


def load_donate_qr_texture(filename, size=180):
    path = resolve_donate_qr(filename)
    if path is None:
        return None
    picture = Gtk.Picture.new_for_filename(str(path))
    picture.set_content_fit(Gtk.ContentFit.CONTAIN)
    picture.set_size_request(size, size)
    picture.add_css_class("boloot-donate-qr")
    picture.set_can_shrink(True)
    return picture


def _system_bus():
    if hasattr(Gio, "DBus"):
        return Gio.DBus.system
    return Gio.bus_get_sync(Gio.BusType.SYSTEM, None)


def dbus_call(method, params=None, reply_type=None):
    try:
        return _system_bus().call_sync(
            DBUS_NAME,
            DBUS_PATH,
            DBUS_IFACE,
            method,
            params,
            reply_type,
            Gio.DBusCallFlags.NONE,
            -1,
            None,
        )
    except GLib.Error:
        return None


def dbus_call_void(method, params=None):
    try:
        _system_bus().call_sync(
            DBUS_NAME,
            DBUS_PATH,
            DBUS_IFACE,
            method,
            params,
            GLib.VariantType.new("()"),
            Gio.DBusCallFlags.NONE,
            -1,
            None,
        )
        return True
    except GLib.Error:
        return False


def rgba_to_hex(rgba):
    r = int(round(rgba.red * 255))
    g = int(round(rgba.green * 255))
    b = int(round(rgba.blue * 255))
    return f"#{r:02x}{g:02x}{b:02x}"


def hex_to_rgba(hex_color):
    rgba = Gdk.RGBA()
    if not rgba.parse(hex_color or DEFAULT_TEXT_COLOR):
        rgba.parse(DEFAULT_TEXT_COLOR)
    return rgba


def rgba_tint_css(hex_color, alpha):
    safe = safe_hex_color(hex_color)
    if not safe:
        return None
    rgba = hex_to_rgba(safe)
    return f"rgba({int(round(rgba.red * 255))}, {int(round(rgba.green * 255))}, {int(round(rgba.blue * 255))}, {alpha:.2f})"


def set_accessible_name(widget, name):
    if hasattr(widget, "set_accessible_name"):
        widget.set_accessible_name(name)
        return
    if hasattr(Gtk, "Accessible") and hasattr(Gtk, "AccessibleProperty"):
        Gtk.Accessible.update_property(
            widget,
            [Gtk.AccessibleProperty.LABEL],
            [name],
        )


SETTINGS_FALLBACK_W = 900
SETTINGS_FALLBACK_H = 760
SETTINGS_MIN_W = 640
SETTINGS_MIN_H = 520
SETTINGS_MAX_W = 1100
SETTINGS_MAX_H = 900
SETTINGS_WIDTH_RATIO = 0.58
SETTINGS_HEIGHT_RATIO = 0.74


def _compute_window_size():
    """Return (width, height) adapted to the primary monitor geometry."""
    display = Gdk.Display.get_default()
    monitor = None
    if display:
        monitors = display.get_monitors()
        if monitors.get_n_items() > 0:
            monitor = monitors.get_item(0)
    if monitor is None:
        return SETTINGS_FALLBACK_W, SETTINGS_FALLBACK_H

    geom = monitor.get_geometry()
    width = max(SETTINGS_MIN_W, min(SETTINGS_MAX_W, int(geom.width * SETTINGS_WIDTH_RATIO)))
    height = max(SETTINGS_MIN_H, min(SETTINGS_MAX_H, int(geom.height * SETTINGS_HEIGHT_RATIO)))
    return width, height


class BolootSettingsApp(Adw.Application):
    def __init__(self):
        super().__init__(application_id="org.boloot.Calendar.Settings")
        self._save_source = 0
        self._preview_source = 0
        self._loading = False
        self._dirty = False
        self._preview_css_provider = Gtk.CssProvider()
        self._own_change = False
        self._preview_year = 0
        self._preview_month = 0
        self._preview_selected_gregorian = None
        self._city_catalog = {}
        self._adhan_preview_proc = None
        self.connect("activate", self.on_activate)

    def on_activate(self, app):
        self._load_css()

        self.win = Adw.PreferencesWindow(
            application=app,
            title=_("تنظیمات {name}").format(name=APP_NAME),
        )
        self.win.add_css_class("boloot-settings")
        width, height = _compute_window_size()
        self.win.set_default_size(width, height)
        self.win.set_size_request(SETTINGS_MIN_W, SETTINGS_MIN_H)
        self.win.connect("close-request", self._on_close)

        self._build_calendar_page()
        self._build_appearance_page()
        self._build_prayer_page()
        self._build_preview_page()
        self._build_about_page()

        self.win.present()
        self.win.connect("notify::visible-page", self._on_visible_page_changed)
        self._subscribe_settings_changed()
        self.load_settings()
        self._apply_text_direction()
        self.refresh_preview()

    def _on_visible_page_changed(self, *_args):
        page = self.win.get_visible_page()
        if page and page.get_title() == "پیش‌نمایش":
            self.refresh_preview()
        elif not page or page.get_title() != "اوقات شرعی":
            self._stop_adhan_preview()

    def _stop_adhan_preview(self):
        proc = self._adhan_preview_proc
        self._adhan_preview_proc = None
        if proc is None:
            return
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=0.5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()

    def _on_close(self, *_args):
        self._stop_adhan_preview()
        if self._preview_source:
            GLib.source_remove(self._preview_source)
            self._preview_source = 0
        if self._save_source:
            GLib.source_remove(self._save_source)
            self._save_source = 0
        if self._dirty:
            self.save_settings(show_toast=False)
        return False

    def _load_css(self):
        css_path = resolve_settings_css()
        if css_path:
            provider = Gtk.CssProvider()
            provider.load_from_path(str(css_path))
            Gtk.StyleContext.add_provider_for_display(
                Gdk.Display.get_default(),
                provider,
                Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
            )
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(),
            self._preview_css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_USER,
        )

    def _build_calendar_page(self):
        page = Adw.PreferencesPage()
        page.set_title("تقویم")
        page.set_icon_name("x-office-calendar-symbolic")

        region = Adw.PreferencesGroup(
            title=_("منطقه و زبان"),
            description=_("کشور، زبان و قوانین تقویم محلی"),
        )
        self.follow_system_locale_row = Adw.SwitchRow(
            title=_("همگام با locale سیستم"),
            subtitle=_("کشور، زبان و اعداد از LANG/LC_TIME سیستم"),
        )
        self.sync_locale_btn = Adw.ButtonRow(
            title=_("همگام‌سازی اکنون"),
        )
        self.country_row = self._combo_row(_("کشور"), COUNTRY_OPTIONS)
        self.language_row = self._combo_row(_("زبان"), LANGUAGE_OPTIONS)
        region.add(self.follow_system_locale_row)
        region.add(self.sync_locale_btn)
        region.add(self.country_row)
        region.add(self.language_row)
        page.add(region)

        cal = Adw.PreferencesGroup(
            title="نمایش تقویم",
            description="نوع تقویم، اعداد و تعطیلات",
        )
        self.calendar_row = self._combo_row("نوع تقویم", CALENDAR_OPTIONS)
        self.week_start_row = self._combo_row("روز شروع هفته", WEEKDAY_OPTIONS)
        self.numerals_row = self._combo_row("اعداد", NUMERAL_OPTIONS)
        self.date_style_row = self._combo_row("قالب تاریخ", DATE_STYLE_OPTIONS)
        self.timezone_row = Adw.EntryRow(title="منطقه زمانی")
        self.show_holidays_row = Adw.SwitchRow(title="نمایش تعطیلات")
        self.holiday_notifications_row = Adw.SwitchRow(title="اعلان تعطیلات")
        cal.add(self.calendar_row)
        cal.add(self.week_start_row)
        cal.add(self.numerals_row)
        cal.add(self.date_style_row)
        cal.add(self.timezone_row)
        cal.add(self.show_holidays_row)
        cal.add(self.holiday_notifications_row)
        page.add(cal)

        self.country_row.connect("notify::selected", self._on_country_changed)
        self.language_row.connect("notify::selected", self._on_locale_manual_change)
        self.numerals_row.connect("notify::selected", self._on_locale_manual_change)
        self.calendar_row.connect("notify::selected", self._on_calendar_changed)
        self.follow_system_locale_row.connect(
            "notify::active", self._on_follow_system_locale_changed
        )
        self.sync_locale_btn.connect("activated", self._sync_system_locale_now)
        for row in (
            self.follow_system_locale_row,
            self.country_row,
            self.language_row,
            self.calendar_row,
            self.week_start_row,
            self.numerals_row,
            self.date_style_row,
            self.timezone_row,
            self.show_holidays_row,
            self.holiday_notifications_row,
        ):
            self._wire_change(row)

        self.win.add(page)

    def _build_appearance_page(self):
        page = Adw.PreferencesPage()
        page.set_title("ظاهر")
        page.set_icon_name("preferences-desktop-display-symbolic")

        panel = Adw.PreferencesGroup(title="نوار بالا و پاپ‌آپ")
        self.show_in_top_bar_row = Adw.SwitchRow(title="نمایش در نوار بالا")
        self.show_clock_row = Adw.SwitchRow(
            title="نمایش ساعت",
            subtitle="ساعت فعلی در کنار تاریخ در نوار بالا",
        )
        self.show_in_popup_row = Adw.SwitchRow(title="نمایش در پاپ‌آپ")
        panel.add(self.show_in_top_bar_row)
        panel.add(self.show_clock_row)
        panel.add(self.show_in_popup_row)
        page.add(panel)

        font = Adw.PreferencesGroup(title="فونت")
        self.font_family_row = Adw.EntryRow(title="خانواده فونت")
        self.font_size_row = Adw.SpinRow.new_with_range(8, 24, 1)
        self.font_size_row.set_title("اندازه فونت (pt)")
        font.add(self.font_family_row)
        font.add(self.font_size_row)
        page.add(font)

        theme = Adw.PreferencesGroup(
            title="رنگ‌ها",
            description="پالت بولوت با کنتراست WCAG AA؛ فقط در حالت سفارشی اعمال می‌شود",
        )
        self.use_system_theme_row = Adw.SwitchRow(
            title="استفاده از تم سیستم",
            subtitle="رنگ‌های semantic GNOME (پیشنهادی)",
        )
        self.text_color_row = self._color_row(
            "رنگ متن",
            subtitle="متن اصلی پاپ‌آپ و نوار بالا",
        )
        self.background_color_row = self._color_row(
            "رنگ پس‌زمینه",
            subtitle="پس‌زمینه بخش تقویم در پاپ‌آپ",
        )
        self.holiday_color_row = self._color_row(
            "رنگ تعطیلات",
            subtitle="روزهای تعطیل رسمی و مناسبت‌ها",
        )
        self.today_color_row = self._color_row(
            "رنگ امروز",
            subtitle="برجسته‌سازی روز جاری با پس‌زمینه ملایم",
        )
        self.prayer_color_row = self._color_row(
            "رنگ اوقات شرعی",
            subtitle="عنوان و زمان‌های نماز در پاپ‌آپ",
        )
        self.reset_colors_row = Adw.ButtonRow(
            title="بازنشانی رنگ‌ها",
        )
        self.reset_colors_row.add_css_class("destructive-action")
        theme.add(self.use_system_theme_row)
        theme.add(self.text_color_row)
        theme.add(self.background_color_row)
        theme.add(self.holiday_color_row)
        theme.add(self.today_color_row)
        theme.add(self.prayer_color_row)
        theme.add(self.reset_colors_row)
        page.add(theme)

        self.use_system_theme_row.connect(
            "notify::active", lambda *_: self._sync_theme_rows()
        )
        self.reset_colors_row.connect("activated", self._reset_custom_colors)
        self.show_in_top_bar_row.connect(
            "notify::active", lambda *_: self._sync_top_bar_rows()
        )
        self._sync_theme_rows()
        self._sync_top_bar_rows()

        for row in (
            self.show_in_top_bar_row,
            self.show_clock_row,
            self.show_in_popup_row,
            self.font_family_row,
            self.font_size_row,
            self.use_system_theme_row,
            self.text_color_row,
            self.background_color_row,
            self.holiday_color_row,
            self.today_color_row,
            self.prayer_color_row,
        ):
            self._wire_change(row)

        self.win.add(page)

    def _build_prayer_page(self):
        page = Adw.PreferencesPage()
        page.set_title("اوقات شرعی")
        page.set_icon_name("weather-clear-night-symbolic")

        basic = Adw.PreferencesGroup(title="مکان و محاسبه")
        self.prayer_enabled_row = Adw.SwitchRow(title="فعال‌سازی اوقات شرعی")
        self.city_row = self._combo_row("شهر", [])
        self.method_row = self._combo_row("روش محاسبه", PRAYER_METHOD_OPTIONS)
        self.madhab_row = self._combo_row("مذهب", MADHAB_OPTIONS)
        basic.add(self.prayer_enabled_row)
        basic.add(self.city_row)
        basic.add(self.method_row)
        basic.add(self.madhab_row)
        page.add(basic)

        display = Adw.PreferencesGroup(title="نمایش و اعلان")
        self.prayer_display_row = self._combo_row("حالت نمایش", PRAYER_DISPLAY_OPTIONS)
        self.prayer_top_bar_row = Adw.SwitchRow(title="نمایش در نوار بالا")
        self.prayer_popup_row = Adw.SwitchRow(title="نمایش در پاپ‌آپ")
        self.notification_row = Adw.EntryRow(title="یادآوری (دقیقه قبل)")
        display.add(self.prayer_display_row)
        display.add(self.prayer_top_bar_row)
        display.add(self.prayer_popup_row)
        display.add(self.notification_row)
        page.add(display)

        adhan = Adw.PreferencesGroup(title="اذان")
        self.adhan_row = Adw.SwitchRow(title="پخش اذان")
        self.adhan_preset_row = self._combo_row("صدای موذن", ADHAN_PRESET_OPTIONS)
        self.adhan_preset_row.set_subtitle("انتخاب صدای مأذون برای پخش اذان")
        preview_btn = Gtk.Button(icon_name="media-playback-start-symbolic")
        preview_btn.set_tooltip_text("پیش‌نمایش صدای موذن")
        set_accessible_name(preview_btn, "پیش‌نمایش صدای موذن")
        preview_btn.connect("clicked", self._preview_adhan)
        self.adhan_preset_row.add_suffix(preview_btn)
        self.adhan_preview_btn = preview_btn
        self.adhan_custom_path_row = Adw.EntryRow(title="فایل سفارشی")
        browse_btn = Gtk.Button(icon_name="folder-open-symbolic")
        browse_btn.set_tooltip_text("انتخاب فایل صوتی")
        set_accessible_name(browse_btn, "انتخاب فایل اذان")
        browse_btn.connect("clicked", self._pick_adhan_file)
        self.adhan_custom_path_row.add_suffix(browse_btn)
        self.adhan_volume_row = Adw.SpinRow.new_with_range(0, 100, 5)
        self.adhan_volume_row.set_title("حجم صدا")
        self.adhan_volume_row.set_subtitle("۰ تا ۱۰۰")
        self.adhan_notify_row = Adw.SwitchRow(title="اعلان دسکتاپ")
        self.adhan_prayer_rows = {}
        adhan.add(self.adhan_row)
        adhan.add(self.adhan_preset_row)
        adhan.add(self.adhan_custom_path_row)
        adhan.add(self.adhan_volume_row)
        adhan.add(self.adhan_notify_row)
        for key, label in ADHAN_PRAYER_OPTIONS:
            row = Adw.SwitchRow(title=label)
            self.adhan_prayer_rows[key] = row
            adhan.add(row)
        page.add(adhan)

        self.prayer_enabled_row.connect("notify::active", self._sync_prayer_rows)
        self.adhan_row.connect("notify::active", self._sync_adhan_rows)
        self.adhan_preset_row.connect("notify::selected", self._sync_adhan_rows)
        self.adhan_custom_path_row.connect("changed", self._sync_adhan_rows)
        self._sync_prayer_rows()

        for row in (
            self.prayer_enabled_row,
            self.city_row,
            self.method_row,
            self.madhab_row,
            self.prayer_display_row,
            self.prayer_top_bar_row,
            self.prayer_popup_row,
            self.notification_row,
            self.adhan_row,
            self.adhan_preset_row,
            self.adhan_custom_path_row,
            self.adhan_volume_row,
            self.adhan_notify_row,
            *self.adhan_prayer_rows.values(),
        ):
            self._wire_change(row)

        self.win.add(page)

    def _build_preview_page(self):
        page = Adw.PreferencesPage()
        page.set_title("پیش‌نمایش")
        page.set_icon_name("view-preview-symbolic")

        top = Adw.PreferencesGroup(
            title="نوار بالا",
            description="نمایش فشرده تاریخ و اوقات شرعی در مرکز نوار بالا",
        )
        top_frame = Gtk.Frame()
        top_frame.add_css_class("card")
        top_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        top_box.add_css_class("boloot-preview-topbar")
        self.preview_label = Gtk.Label(label="…")
        self.preview_label.set_wrap(True)
        self.preview_label.set_selectable(True)
        self.preview_label.add_css_class("title-3")
        self.preview_label.set_halign(Gtk.Align.CENTER)
        set_accessible_name(self.preview_label, "پیش‌نمایش نوار بالا")
        top_box.append(self.preview_label)
        top_frame.set_child(top_box)
        top.add(top_frame)
        page.add(top)

        month = Adw.PreferencesGroup(
            title="نمای ماهانه",
            description="پیش‌نمایش زنده با رنگ‌های تم سیستم یا سفارشی",
        )
        month_frame = Gtk.Frame()
        month_frame.add_css_class("card")
        self.month_preview_frame = month_frame
        month_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        month_box.add_css_class("boloot-preview-calendar")
        self.month_preview_box = month_box

        nav = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        nav.add_css_class("boloot-preview-nav")
        nav.set_halign(Gtk.Align.CENTER)
        self.preview_year_nav = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        self.preview_year_nav.add_css_class("boloot-preview-nav")
        self.preview_year_nav.set_halign(Gtk.Align.CENTER)
        self.preview_prev_year_btn = Gtk.Button(label="")
        self.preview_prev_year_btn.add_css_class("boloot-preview-nav-btn")
        self.preview_year_label = Gtk.Label(label="")
        self.preview_year_label.add_css_class("title-4")
        self.preview_next_year_btn = Gtk.Button(label="")
        self.preview_next_year_btn.add_css_class("boloot-preview-nav-btn")
        self.preview_year_nav.append(self.preview_prev_year_btn)
        self.preview_year_nav.append(self.preview_year_label)
        self.preview_year_nav.append(self.preview_next_year_btn)
        self.preview_prev_year_btn.connect("clicked", lambda *_: self._shift_preview_year(-1))
        self.preview_next_year_btn.connect("clicked", lambda *_: self._shift_preview_year(1))

        self.preview_prev_month_btn = Gtk.Button(label="")
        self.preview_prev_month_btn.add_css_class("boloot-preview-nav-btn")
        next_btn = Gtk.Button(label="")
        next_btn.add_css_class("boloot-preview-nav-btn")
        self.preview_next_month_btn = next_btn
        self.month_title_label = Gtk.Label(label="")
        self.month_title_label.add_css_class("title-4")
        self.preview_month_nav = nav
        nav.append(self.preview_prev_month_btn)
        nav.append(self.month_title_label)
        nav.append(next_btn)
        self.preview_prev_month_btn.connect("clicked", lambda *_: self._shift_preview_month(-1))
        next_btn.connect("clicked", lambda *_: self._shift_preview_month(1))

        self.calendar_grid = Gtk.Grid(column_spacing=4, row_spacing=4, halign=Gtk.Align.CENTER)
        month_box.append(self.preview_year_nav)
        month_box.append(nav)
        month_box.append(self.calendar_grid)
        month_frame.set_child(month_box)
        month.add(month_frame)
        page.add(month)

        self.win.add(page)

    def _build_about_page(self):
        page = Adw.PreferencesPage()
        page.set_title("درباره")
        page.set_icon_name("help-about-symbolic")

        group = Adw.PreferencesGroup(
            title=APP_NAME,
            description="تقویم فارسی، اوقات شرعی و تعطیلات رسمی",
        )
        about_row = Adw.ActionRow(title="درباره بولوت")
        about_row.set_subtitle(WEBSITE_LABEL)
        about_row.add_suffix(Gtk.Image.new_from_icon_name("go-next-symbolic"))
        about_row.set_activatable(True)
        about_row.connect("activated", self._show_about_dialog)
        group.add(about_row)
        page.add(group)

        donate_group = Adw.PreferencesGroup(
            title="حمایت",
            description=(
                "اگر بولوت هر روز کنارت بود، با یک حمایت کوچک کمک کن تا تقویم فارسی "
                "رایگان بماند و برای همه بهتر شود. برای USDT فقط شبکه TRC20 را انتخاب کنید."
            ),
        )
        donate_box = Gtk.Box(
            orientation=Gtk.Orientation.VERTICAL,
            spacing=16,
            margin_top=8,
            margin_bottom=8,
            margin_start=12,
            margin_end=12,
        )
        donate_box.add_css_class("boloot-donate")

        cards = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=16, homogeneous=True)
        cards.set_halign(Gtk.Align.CENTER)
        cards.append(
            self._build_donate_card(
                "USDT · TRC20",
                DONATE_USDT_TRC20,
                "usdt-trc20-qr.png",
            )
        )
        cards.append(
            self._build_donate_card(
                "Bitcoin (BTC)",
                DONATE_BTC,
                "btc-qr.png",
            )
        )
        donate_box.append(cards)

        warn = Gtk.Label(
            label="⚠️ برای USDT فقط TRC20 ارسال کنید؛ شبکه دیگر ممکن است باعث از دست رفتن دارایی شود.",
            wrap=True,
            xalign=0,
        )
        warn.add_css_class("dim-label")
        warn.add_css_class("boloot-donate-warn")
        donate_box.append(warn)

        donate_row = Adw.PreferencesRow()
        donate_row.set_child(donate_box)
        donate_group.add(donate_row)
        page.add(donate_group)

        service_group = Adw.PreferencesGroup(
            title="سرویس",
            description="راه‌اندازی مجدد دیمون پس از به‌روزرسانی یا مشکل اتصال",
        )
        self.service_status_row = Adw.ActionRow(title="وضعیت سرویس")
        self.service_status_row.set_selectable(False)
        self.restart_service_btn = Adw.ButtonRow(
            title="راه‌اندازی مجدد سرویس",
        )
        service_group.add(self.service_status_row)
        service_group.add(self.restart_service_btn)
        page.add(service_group)

        self.restart_service_btn.connect("activated", self._restart_service)
        self._refresh_service_status()

        self.win.add(page)

    def _build_donate_card(self, title, address, qr_filename):
        card = Gtk.Box(
            orientation=Gtk.Orientation.VERTICAL,
            spacing=8,
            halign=Gtk.Align.CENTER,
        )
        card.add_css_class("boloot-donate-card")

        title_lbl = Gtk.Label(label=title, xalign=0.5)
        title_lbl.add_css_class("heading")
        card.append(title_lbl)

        qr = load_donate_qr_texture(qr_filename)
        if qr is not None:
            card.append(qr)
        else:
            missing = Gtk.Label(label="کد QR یافت نشد", xalign=0.5)
            missing.add_css_class("dim-label")
            card.append(missing)

        addr = Gtk.Label(
            label=address,
            wrap=True,
            selectable=True,
            xalign=0.5,
            max_width_chars=28,
        )
        addr.add_css_class("boloot-donate-address")
        card.append(addr)

        copy_btn = Gtk.Button(label="کپی آدرس")
        copy_btn.add_css_class("pill")
        copy_btn.set_halign(Gtk.Align.CENTER)
        copy_btn.connect("clicked", self._copy_donate_address, address)
        card.append(copy_btn)
        return card

    def _copy_donate_address(self, _btn, address):
        display = Gdk.Display.get_default()
        if display is None:
            self.show_toast("کپی آدرس ناموفق بود")
            return
        display.get_clipboard().set(address)
        self.show_toast("آدرس کپی شد")

    def _show_about_dialog(self, *_args):
        dialog = Adw.AboutWindow(
            transient_for=self.win,
            application_name=APP_NAME,
            developer_name=WEBSITE_LABEL,
            website=WEBSITE,
            application_icon="preferences-system-time",
            comments=(
                "تقویم شمسی، اوقات شرعی و تعطیلات برای ایران، افغانستان و تاجیکستان\n\n"
                f"حمایت USDT (TRC20):\n{DONATE_USDT_TRC20}\n\n"
                f"حمایت Bitcoin:\n{DONATE_BTC}"
            ),
        )
        dialog.present(self.win)

    def _service_is_active(self):
        try:
            result = subprocess.run(
                ["systemctl", "--user", "is-active", SERVICE_UNIT],
                capture_output=True,
                text=True,
                check=False,
            )
            return result.stdout.strip() == "active"
        except FileNotFoundError:
            return dbus_call("GetSettings", None, GLib.VariantType.new("(s)")) is not None

    def _refresh_service_status(self):
        if not hasattr(self, "service_status_row"):
            return
        if self._service_is_active():
            self.service_status_row.set_subtitle("در حال اجرا")
        else:
            self.service_status_row.set_subtitle("متوقف یا در دسترس نیست")

    def _after_service_restart(self):
        self.restart_service_btn.set_sensitive(True)
        self._refresh_service_status()
        self.load_settings()
        return False

    def _restart_service(self, *_args):
        self.restart_service_btn.set_sensitive(False)
        try:
            subprocess.run(
                ["systemctl", "--user", "restart", SERVICE_UNIT],
                capture_output=True,
                text=True,
                check=True,
            )
            self.show_toast("سرویس راه‌اندازی مجدد شد")
            GLib.timeout_add_seconds(1, self._after_service_restart)
        except (subprocess.CalledProcessError, FileNotFoundError):
            self.show_toast("راه‌اندازی مجدد سرویس ناموفق بود")
            self.restart_service_btn.set_sensitive(True)
            self._refresh_service_status()

    def _color_row(self, title, subtitle=None):
        if HAS_ADW_COLOR_ROW:
            row = Adw.ColorRow(title=title)
            if subtitle:
                row.set_subtitle(subtitle)
            return row

        rgba = Gdk.RGBA()
        rgba.parse(DEFAULT_TEXT_COLOR)
        row = Adw.ActionRow(title=title)
        if subtitle:
            row.set_subtitle(subtitle)
        button = Gtk.ColorDialogButton()
        button.set_dialog(Gtk.ColorDialog())
        button.set_rgba(rgba)
        row.add_suffix(button)
        row._boloot_color_widget = button
        return row

    def _is_color_row(self, widget):
        if HAS_ADW_COLOR_ROW and isinstance(widget, Adw.ColorRow):
            return True
        return getattr(widget, "_boloot_color_widget", None) is not None

    def _combo_row(self, title, options):
        row = Adw.ComboRow(title=title)
        row._values = [value for value, _label in options]
        model = Gtk.StringList.new([label for _value, label in options])
        row.set_model(model)
        if row._values:
            row.set_selected(0)
        return row

    def _combo_get(self, row):
        idx = row.get_selected()
        if idx < 0 or idx >= len(row._values):
            return row._values[0] if row._values else ""
        return row._values[idx]

    def _combo_set(self, row, value):
        for idx, item in enumerate(row._values):
            if item == value:
                row.set_selected(idx)
                return
        if row._values:
            row.set_selected(0)

    def _wire_change(self, widget):
        if isinstance(widget, Adw.ComboRow):
            widget.connect("notify::selected", self._on_setting_changed)
        elif isinstance(widget, Adw.SwitchRow):
            widget.connect("notify::active", self._on_setting_changed)
        elif isinstance(widget, Adw.SpinRow):
            widget.connect("notify::value", self._on_setting_changed)
        elif self._is_color_row(widget):
            if HAS_ADW_COLOR_ROW:
                widget.connect("notify::rgba", self._on_setting_changed)
            else:
                widget._boloot_color_widget.connect(
                    "notify::rgba", self._on_setting_changed
                )
        elif isinstance(widget, Adw.EntryRow):
            widget.connect("changed", self._on_setting_changed)
        elif isinstance(widget, Gtk.Entry):
            widget.connect("changed", self._on_setting_changed)

    def _on_setting_changed(self, *_args):
        if self._loading:
            return
        self._dirty = True
        self.schedule_preview()
        self.schedule_save()

    def schedule_preview(self):
        if self._preview_source:
            GLib.source_remove(self._preview_source)
        self._preview_source = GLib.timeout_add(
            PREVIEW_DEBOUNCE_MS, self._debounced_preview
        )

    def _debounced_preview(self):
        self._preview_source = 0
        self.refresh_preview()
        return False

    def _on_country_changed(self, *_args):
        if self._loading:
            return
        if self.follow_system_locale_row.get_active():
            self._loading = True
            self.follow_system_locale_row.set_active(False)
            self._loading = False
        country = self._combo_get(self.country_row)
        self._refresh_language_options(country)
        self._refresh_city_options(country, preserve=False)
        self._on_setting_changed()

    def _refresh_language_options(self, country):
        allowed = LANGUAGES_BY_COUNTRY.get(country, ["persian"])
        options = [opt for opt in LANGUAGE_OPTIONS if opt[0] in allowed]
        current = self._combo_get(self.language_row)
        self.language_row._values = [value for value, _label in options]
        self.language_row.set_model(Gtk.StringList.new([label for _v, label in options]))
        if current in self.language_row._values:
            self._combo_set(self.language_row, current)
        else:
            self._combo_set(self.language_row, self.language_row._values[0])

    def _load_city_catalog(self):
        catalog = {}
        locations_dir = resolve_data_dir() / "locations"
        if not locations_dir.is_dir():
            return catalog
        for path in sorted(locations_dir.glob("*.json")):
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                continue
            for city in data.get("cities", []):
                city_id = city.get("id")
                if city_id:
                    catalog[city_id] = city
        return catalog

    def _refresh_city_options(self, country, preserve=True):
        previous = self._combo_get(self.city_row) if preserve else None
        options = []
        for city_id, city in sorted(
            self._city_catalog.items(),
            key=lambda item: item[1].get("name_fa", item[0]),
        ):
            if city.get("country", country) == country:
                label = city.get("name_fa") or city.get("name") or city_id
                options.append((city_id, label))
        if not options:
            options = [("tehran", "تهران")]
        self.city_row._values = [value for value, _label in options]
        self.city_row.set_model(Gtk.StringList.new([label for _v, label in options]))
        target = previous if preserve and previous in self.city_row._values else options[0][0]
        self._combo_set(self.city_row, target)

    def _sync_top_bar_rows(self, *_args):
        enabled = self.show_in_top_bar_row.get_active()
        self.show_clock_row.set_sensitive(enabled)

    def _sync_prayer_rows(self, *_args):
        enabled = self.prayer_enabled_row.get_active()
        for row in (
            self.city_row,
            self.method_row,
            self.madhab_row,
            self.prayer_display_row,
            self.prayer_top_bar_row,
            self.prayer_popup_row,
            self.notification_row,
            self.adhan_row,
            self.adhan_preset_row,
            self.adhan_custom_path_row,
            self.adhan_volume_row,
            self.adhan_notify_row,
            *self.adhan_prayer_rows.values(),
        ):
            row.set_sensitive(enabled)
        self._sync_adhan_rows()

    def _sync_adhan_rows(self, *_args):
        prayer_on = self.prayer_enabled_row.get_active()
        adhan_on = self.adhan_row.get_active()
        custom = self._combo_get(self.adhan_preset_row) == "custom"
        self.adhan_preset_row.set_sensitive(prayer_on)
        self.adhan_custom_path_row.set_sensitive(prayer_on and custom)
        resolved = self._resolve_adhan_path()
        self.adhan_preview_btn.set_sensitive(prayer_on and resolved is not None)
        for row in (
            self.adhan_volume_row,
            self.adhan_notify_row,
            *self.adhan_prayer_rows.values(),
        ):
            row.set_sensitive(prayer_on and adhan_on)

    def _resolve_adhan_path(self):
        preset = self._combo_get(self.adhan_preset_row)
        if preset == "custom":
            path = self.adhan_custom_path_row.get_text().strip()
            return Path(path) if path else None
        filename = ADHAN_PRESET_FILES.get(preset, "mansouri.ogg")
        return resolve_data_dir() / "sounds" / filename

    def _preview_adhan(self, *_args):
        path = self._resolve_adhan_path()
        if path is None or not path.is_file():
            self.show_toast("فایل صوتی موذن یافت نشد")
            return
        self._stop_adhan_preview()
        volume = int(self.adhan_volume_row.get_value())
        paplay_volume = volume * 65536 // 100
        for cmd in (
            ["paplay", "--volume", str(paplay_volume), str(path)],
            ["pw-play", str(path)],
            ["gst-play-1.0", "--no-interactive", str(path)],
        ):
            try:
                proc = subprocess.Popen(
                    cmd,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
                time.sleep(0.15)
                exit_code = proc.poll()
                if exit_code is None or exit_code == 0:
                    if exit_code is None:
                        self._adhan_preview_proc = proc
                    return
            except FileNotFoundError:
                continue
        self.show_toast("پخش‌کننده صوتی در دسترس نیست")

    def _pick_adhan_file(self, *_args):
        dialog = Gtk.FileDialog.new()
        dialog.set_title("انتخاب فایل اذان")
        filters = Gio.ListStore.new(Gtk.FileFilter)
        for pattern, name in (
            ("audio/x-vorbis, audio/ogg", "OGG"),
            ("audio/wav, audio/x-wav", "WAV"),
            ("audio/mpeg", "MP3"),
            ("audio/flac", "FLAC"),
        ):
            filt = Gtk.FileFilter()
            filt.add_mime_type(pattern.split(",")[0].strip())
            if "," in pattern:
                filt.add_mime_type(pattern.split(",")[1].strip())
            filt.set_name(name)
            filters.append(filt)
        dialog.set_filters(filters)

        def on_selected(_dialog, result):
            try:
                file_obj = _dialog.finish(result)
            except GLib.Error:
                return
            if file_obj is None:
                return
            path = file_obj.get_path()
            if path:
                self.adhan_custom_path_row.set_text(path)
                self._on_setting_changed()

        dialog.open(self.win, None, on_selected)

    def _sync_theme_rows(self):
        custom = not self.use_system_theme_row.get_active()
        for row in (
            self.text_color_row,
            self.background_color_row,
            self.holiday_color_row,
            self.today_color_row,
            self.prayer_color_row,
            self.reset_colors_row,
        ):
            row.set_sensitive(custom)

    def _reset_custom_colors(self, *_args):
        self._loading = True
        self._set_color_row(self.text_color_row, DEFAULT_TEXT_COLOR)
        self._set_color_row(self.background_color_row, DEFAULT_BG_COLOR)
        self._set_color_row(self.holiday_color_row, DEFAULT_HOLIDAY_COLOR)
        self._set_color_row(self.today_color_row, DEFAULT_TODAY_COLOR)
        self._set_color_row(self.prayer_color_row, DEFAULT_PRAYER_COLOR)
        self._loading = False
        self._on_setting_changed()
        self.show_toast("رنگ‌ها به پالت پیش‌فرض بولوت بازنشانی شد")

    def _sync_locale_rows(self):
        follow = self.follow_system_locale_row.get_active()
        for row in (self.country_row, self.language_row, self.numerals_row):
            row.set_sensitive(not follow)
        self.sync_locale_btn.set_sensitive(follow)

    def _on_follow_system_locale_changed(self, *_args):
        if self._loading:
            return
        self._sync_locale_rows()
        if self.follow_system_locale_row.get_active():
            self._sync_system_locale_now()
        else:
            self._on_setting_changed()

    def _on_locale_manual_change(self, *_args):
        if self._loading:
            return
        if self.follow_system_locale_row.get_active():
            self._loading = True
            self.follow_system_locale_row.set_active(False)
            self._loading = False
        self._on_setting_changed()

    def _on_calendar_changed(self, *_args):
        if self._loading:
            return
        if self._combo_get(self.calendar_row) == "gregorian":
            self._loading = True
            self._combo_set(self.numerals_row, "latin")
            self._loading = False
        self._on_setting_changed()

    def _sync_system_locale_now(self, *_args):
        raw = self.run_ctl("detect-locale")
        if not raw:
            self.show_toast(_("همگام‌سازی locale ناموفق بود"))
            return
        try:
            detected = json.loads(raw)
        except json.JSONDecodeError:
            self.show_toast(_("همگام‌سازی locale ناموفق بود"))
            return

        self._loading = True
        country = detected.get("country", "iran")
        self._combo_set(self.country_row, country)
        self._refresh_language_options(country)
        self._combo_set(self.language_row, detected.get("language", "persian"))
        self._combo_set(self.numerals_row, detected.get("numerals", "persian"))
        self._loading = False
        self._sync_locale_rows()
        self._apply_text_direction()
        self.schedule_save()

    def _apply_text_direction(self):
        language = self._combo_get(self.language_row)
        if language == "tajik":
            Gtk.Widget.set_default_direction(Gtk.TextDirection.LTR)
        else:
            Gtk.Widget.set_default_direction(Gtk.TextDirection.RTL)

    def _sync_color_rows(self):
        self._sync_theme_rows()

    def _set_color_row(self, row, hex_color):
        rgba = hex_to_rgba(hex_color)
        if HAS_ADW_COLOR_ROW and isinstance(row, Adw.ColorRow):
            row.set_rgba(rgba)
        else:
            row._boloot_color_widget.set_rgba(rgba)

    def _get_color_row(self, row):
        if HAS_ADW_COLOR_ROW and isinstance(row, Adw.ColorRow):
            return rgba_to_hex(row.get_rgba())
        return rgba_to_hex(row._boloot_color_widget.get_rgba())

    def run_ctl(self, *args):
        ctl = resolve_ctl()
        try:
            out = subprocess.check_output([ctl, *args], text=True, stderr=subprocess.PIPE)
            return out.strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            return ""

    def import_config(self, config):
        payload = json.dumps(config, ensure_ascii=False)
        if dbus_call_void("SetSettings", GLib.Variant("(s)", [payload])):
            return True

        tmp = Path.home() / ".config/boloot-calendar/settings-import.json"
        tmp.parent.mkdir(parents=True, exist_ok=True)
        tmp.write_text(payload, encoding="utf-8")
        try:
            subprocess.check_output(
                [resolve_ctl(), "import", str(tmp)],
                text=True,
                stderr=subprocess.PIPE,
            )
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False
        if dbus_call_void("SetSettings", GLib.Variant("(s)", [payload])):
            return True
        return dbus_call_void("Reload", None)

    def show_toast(self, message):
        toast = Adw.Toast.new(message)
        if hasattr(self.win, "add_toast"):
            self.win.add_toast(toast)
            return
        overlay = getattr(self, "_toast_overlay", None)
        if overlay is None:
            content = self.win
            overlay = Adw.ToastOverlay.new()
            self._toast_overlay = overlay
            parent = content.get_parent()
            if parent:
                parent.remove(content)
                overlay.set_child(content)
                parent.append(overlay)
        overlay.add_toast(toast)

    def fetch_settings_json(self):
        reply = dbus_call("GetSettings", None, GLib.VariantType.new("(s)"))
        if reply:
            return reply.get_child_value(0).get_string()
        return self.run_ctl("export")

    def schedule_save(self):
        if self._save_source:
            GLib.source_remove(self._save_source)
        self._save_source = GLib.timeout_add(SAVE_DEBOUNCE_MS, self._debounced_save)

    def _debounced_save(self):
        self._save_source = 0
        self.save_settings(show_toast=True)
        return False

    def build_config(self, base=None):
        config = json.loads(json.dumps(base)) if base else {}
        config.setdefault("calendar", {})
        config.setdefault("appearance", {})
        config.setdefault("prayer", {})

        cal = config["calendar"]
        cal["country"] = self._combo_get(self.country_row)
        cal["language"] = self._combo_get(self.language_row)
        cal["calendar_type"] = self._combo_get(self.calendar_row)
        cal["week_start"] = self._combo_get(self.week_start_row)
        cal["numerals"] = self._combo_get(self.numerals_row)
        cal["date_style"] = self._combo_get(self.date_style_row)
        cal["timezone"] = self.timezone_row.get_text().strip() or "Asia/Tehran"
        cal["show_holidays"] = self.show_holidays_row.get_active()
        cal["holiday_notifications"] = self.holiday_notifications_row.get_active()
        cal["follow_system_locale"] = self.follow_system_locale_row.get_active()

        app = config["appearance"]
        app["font_family"] = self.font_family_row.get_text().strip() or "Vazirmatn"
        app["font_size_pt"] = int(self.font_size_row.get_value())
        app["text_color"] = self._get_color_row(self.text_color_row)
        app["background_color"] = self._get_color_row(self.background_color_row)
        app["holiday_color"] = self._get_color_row(self.holiday_color_row)
        app["today_color"] = self._get_color_row(self.today_color_row)
        app["prayer_color"] = self._get_color_row(self.prayer_color_row)
        app["use_system_theme"] = self.use_system_theme_row.get_active()
        app["show_in_top_bar"] = self.show_in_top_bar_row.get_active()
        app["show_clock"] = self.show_clock_row.get_active()
        app["show_in_popup"] = self.show_in_popup_row.get_active()

        prayer = config["prayer"]
        prayer["enabled"] = self.prayer_enabled_row.get_active()
        prayer["city"] = self._combo_get(self.city_row)
        prayer["method"] = self._combo_get(self.method_row)
        prayer["madhab"] = self._combo_get(self.madhab_row)
        prayer["display_mode"] = self._combo_get(self.prayer_display_row)
        prayer["show_in_top_bar"] = self.prayer_top_bar_row.get_active()
        prayer["show_in_popup"] = self.prayer_popup_row.get_active()
        prayer["adhan_enabled"] = self.adhan_row.get_active()
        prayer["adhan_preset"] = self._combo_get(self.adhan_preset_row)
        custom_path = self.adhan_custom_path_row.get_text().strip()
        prayer["adhan_custom_path"] = custom_path or None
        prayer["adhan_volume"] = int(self.adhan_volume_row.get_value())
        prayer["adhan_show_notification"] = self.adhan_notify_row.get_active()
        prayer["adhan_prayers"] = {
            key: row.get_active() for key, row in self.adhan_prayer_rows.items()
        }
        raw_minutes = self.notification_row.get_text().strip()
        minutes = []
        for part in raw_minutes.replace("،", ",").split(","):
            part = part.strip()
            if part.isdigit():
                minutes.append(int(part))
        prayer["notification_minutes"] = minutes or [10]
        return config

    def load_settings(self):
        raw = self.fetch_settings_json()
        if not raw:
            return
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return

        self._loading = True
        self._city_catalog = self._load_city_catalog()

        cal = data.get("calendar", {})
        app = data.get("appearance", {})
        prayer = data.get("prayer", {})

        country = cal.get("country", "iran")
        self._combo_set(self.country_row, country)
        self._refresh_language_options(country)
        self._combo_set(self.language_row, cal.get("language", "persian"))
        self._combo_set(self.calendar_row, cal.get("calendar_type", "jalali"))
        self._combo_set(self.week_start_row, cal.get("week_start", "saturday"))
        self._combo_set(self.numerals_row, cal.get("numerals", "persian"))
        self._combo_set(self.date_style_row, cal.get("date_style", "long_named"))
        self.timezone_row.set_text(cal.get("timezone", "Asia/Tehran"))
        self.show_holidays_row.set_active(cal.get("show_holidays", True))
        self.holiday_notifications_row.set_active(cal.get("holiday_notifications", True))
        self.follow_system_locale_row.set_active(cal.get("follow_system_locale", True))
        self._sync_locale_rows()

        self.show_in_top_bar_row.set_active(app.get("show_in_top_bar", True))
        self.show_clock_row.set_active(app.get("show_clock", True))
        self.show_in_popup_row.set_active(app.get("show_in_popup", True))
        self.font_family_row.set_text(app.get("font_family", "Vazirmatn"))
        self.font_size_row.set_value(app.get("font_size_pt", 11))
        self._set_color_row(self.text_color_row, app.get("text_color", DEFAULT_TEXT_COLOR))
        self._set_color_row(self.background_color_row, app.get("background_color", DEFAULT_BG_COLOR))
        self._set_color_row(self.holiday_color_row, app.get("holiday_color", DEFAULT_HOLIDAY_COLOR))
        self._set_color_row(self.today_color_row, app.get("today_color", DEFAULT_TODAY_COLOR))
        self._set_color_row(self.prayer_color_row, app.get("prayer_color", DEFAULT_PRAYER_COLOR))
        self.use_system_theme_row.set_active(app.get("use_system_theme", True))
        self._sync_color_rows()
        self._sync_top_bar_rows()

        self._refresh_city_options(country, preserve=True)
        self._combo_set(self.city_row, prayer.get("city", "tehran"))
        self.prayer_enabled_row.set_active(prayer.get("enabled", True))
        self._combo_set(self.method_row, prayer.get("method", "tehran"))
        self._combo_set(self.madhab_row, prayer.get("madhab", "jafari"))
        self._combo_set(self.prayer_display_row, prayer.get("display_mode", "next_prayer"))
        self.prayer_top_bar_row.set_active(prayer.get("show_in_top_bar", True))
        self.prayer_popup_row.set_active(prayer.get("show_in_popup", True))
        self.adhan_row.set_active(prayer.get("adhan_enabled", False))
        adhan_preset = prayer.get("adhan_preset", "mansouri")
        if adhan_preset == "default":
            adhan_preset = "mansouri"
        self._combo_set(self.adhan_preset_row, adhan_preset)
        self.adhan_custom_path_row.set_text(prayer.get("adhan_custom_path") or "")
        self.adhan_volume_row.set_value(prayer.get("adhan_volume", 80))
        self.adhan_notify_row.set_active(prayer.get("adhan_show_notification", True))
        adhan_prayers = prayer.get("adhan_prayers", {})
        defaults = {
            "fajr": True,
            "sunrise": False,
            "dhuhr": True,
            "asr": True,
            "maghrib": True,
            "isha": True,
        }
        for key, row in self.adhan_prayer_rows.items():
            row.set_active(adhan_prayers.get(key, defaults.get(key, True)))
        minutes = prayer.get("notification_minutes", [10])
        self.notification_row.set_text(", ".join(str(m) for m in minutes))

        self._sync_prayer_rows()
        self._loading = False
        self._refresh_service_status()

    def save_settings(self, show_toast=True):
        raw = self.fetch_settings_json()
        try:
            base = json.loads(raw) if raw else {}
        except json.JSONDecodeError:
            base = {}
        config = self.build_config(base)
        payload = json.dumps(config, ensure_ascii=False)
        self._own_change = True
        saved = dbus_call_void("SetSettings", GLib.Variant("(s)", [payload]))
        if not saved:
            saved = self.import_config(config)
        self._own_change = False
        if saved:
            self._dirty = False
            if show_toast:
                self.show_toast("تنظیمات ذخیره شد")
            self.refresh_preview()
        elif show_toast:
            self.show_toast("ذخیره تنظیمات ناموفق بود")
        return saved

    def fetch_month_view(self, year=0, month=0):
        reply = dbus_call(
            "GetMonthView",
            GLib.Variant("(ii)", [year, month]),
            GLib.VariantType.new("(s)"),
        )
        if reply:
            try:
                return json.loads(reply.get_child_value(0).get_string())
            except json.JSONDecodeError:
                return None
        raw = self.run_ctl("month", f"--year={year}", f"--month={month}")
        if not raw:
            return None
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            return None

    def fetch_top_bar_preview(self):
        reply = dbus_call("GetTopBarText", None, GLib.VariantType.new("(s)"))
        if reply:
            return reply.get_child_value(0).get_string()
        return self.run_ctl("preview")

    def fetch_calendar_view(self):
        reply = dbus_call("GetCalendarView", None, GLib.VariantType.new("(s)"))
        if not reply:
            return None
        raw = reply.get_child_value(0).get_string()
        if not raw:
            return None
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            return None

    @staticmethod
    def _clear_container(container):
        while True:
            child = container.get_first_child()
            if child is None:
                break
            container.remove(child)

    def _live_appearance(self, view_appearance=None):
        appearance = dict(view_appearance or {})
        use_system = self.use_system_theme_row.get_active()
        appearance["use_system_theme"] = use_system
        appearance["apply_custom_appearance"] = not use_system
        if not use_system:
            appearance["text_color"] = self._get_color_row(self.text_color_row)
            appearance["background_color"] = self._get_color_row(self.background_color_row)
            appearance["holiday_color"] = self._get_color_row(self.holiday_color_row)
            appearance["today_color"] = self._get_color_row(self.today_color_row)
            appearance["prayer_color"] = self._get_color_row(self.prayer_color_row)
            appearance["today_background_color"] = rgba_tint_css(
                appearance["today_color"], 0.22
            )
            appearance["holiday_background_color"] = rgba_tint_css(
                appearance["holiday_color"], 0.14
            )
        return appearance

    def _update_preview_theme_css(self, appearance):
        use_system = appearance.get("use_system_theme", True)
        apply_custom = appearance.get("apply_custom_appearance", not use_system)
        if not apply_custom:
            self._preview_css_provider.load_from_data("/* boloot preview: system theme */", -1)
            return

        text = safe_hex_color(appearance.get("text_color")) or DEFAULT_TEXT_COLOR
        bg = safe_hex_color(appearance.get("background_color")) or DEFAULT_BG_COLOR
        today = safe_hex_color(appearance.get("today_color")) or DEFAULT_TODAY_COLOR
        holiday = safe_hex_color(appearance.get("holiday_color")) or DEFAULT_HOLIDAY_COLOR
        today_bg = appearance.get("today_background_color") or rgba_tint_css(today, 0.22)

        css = f"""
        .boloot-preview-calendar-custom {{
            background-color: {bg};
            color: {text};
            border-radius: 12px;
        }}
        .boloot-preview-day-custom-today {{
            background-color: {today_bg};
            border-radius: 22px;
        }}
        """
        self._preview_css_provider.load_from_data(css.encode("utf-8"))

    def _make_day_cell(self, cell, appearance):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        box.set_halign(Gtk.Align.CENTER)
        box.set_valign(Gtk.Align.CENTER)

        button = Gtk.Button()
        button.add_css_class("flat")
        button.add_css_class("boloot-preview-day")
        button.set_child(box)

        if not cell.get("is_current_month", True):
            button.add_css_class("boloot-preview-day-outside")

        gregorian = cell.get("gregorian_date")
        if gregorian and gregorian == self._preview_selected_gregorian:
            button.add_css_class("boloot-preview-day-selected")

        use_system = appearance.get("use_system_theme", True)
        apply_custom = appearance.get("apply_custom_appearance", not use_system)
        label = cell.get("day_label") or ""
        primary = Gtk.Label()
        primary.add_css_class("title")

        today_color = safe_hex_color(appearance.get("today_color"))
        holiday_color = safe_hex_color(appearance.get("holiday_color"))

        if not apply_custom:
            if cell.get("is_today"):
                button.add_css_class("boloot-preview-day-today")
            if cell.get("is_holiday"):
                button.add_css_class("boloot-preview-day-holiday")
            if cell.get("is_weekend"):
                button.add_css_class("boloot-preview-day-weekend")
            primary.set_label(label)
        elif cell.get("is_today"):
            button.add_css_class("boloot-preview-day-custom-today")
            if cell.get("is_holiday") and holiday_color:
                primary.set_markup(
                    f"<span foreground='{holiday_color}'><b>{label}</b></span>"
                )
            elif today_color:
                primary.set_markup(
                    f"<span foreground='{today_color}'><b>{label}</b></span>"
                )
            else:
                primary.set_label(label)
        elif cell.get("is_holiday"):
            if holiday_color:
                primary.set_markup(
                    f"<span foreground='{holiday_color}'><b>{label}</b></span>"
                )
            else:
                primary.set_label(label)
        elif cell.get("is_weekend"):
            if holiday_color:
                primary.set_markup(
                    f"<span foreground='{holiday_color}'><b>{label}</b></span>"
                )
            else:
                primary.set_label(label)
        else:
            primary.set_label(label)

        box.append(primary)

        secondary = cell.get("secondary_label")
        if secondary:
            sec = Gtk.Label(label=secondary)
            sec.add_css_class("boloot-preview-day-secondary")
            sec.add_css_class("dim-label")
            box.append(sec)

        tooltip = cell.get("tooltip")
        if tooltip:
            button.set_tooltip_text(tooltip)
            set_accessible_name(button, tooltip)
        elif label:
            set_accessible_name(button, label)

        if gregorian:
            button.connect("clicked", lambda *_: self._on_preview_day_clicked(cell))

        return button

    def _on_preview_day_clicked(self, cell):
        gregorian = cell.get("gregorian_date")
        if not gregorian:
            return
        self._preview_selected_gregorian = gregorian
        if not cell.get("is_current_month", True):
            self._preview_year = cell.get("jalali_year", self._preview_year)
            self._preview_month = cell.get("jalali_month", self._preview_month)
        self.refresh_calendar_preview()

    def _shift_preview_month(self, delta):
        view = self.fetch_month_view(self._preview_year, self._preview_month)
        if not view:
            return
        year = view.get("display_year", view.get("jalali_year", self._preview_year))
        month = view.get("display_month", view.get("jalali_month", self._preview_month))
        month += delta
        if month < 1:
            month = 12
            year -= 1
        elif month > 12:
            month = 1
            year += 1
        self._preview_year = year
        self._preview_month = month
        self.refresh_calendar_preview()

    def _shift_preview_year(self, delta):
        view = self.fetch_month_view(self._preview_year, self._preview_month)
        if not view:
            return
        year = view.get("display_year", view.get("jalali_year", self._preview_year))
        month = view.get("display_month", view.get("jalali_month", self._preview_month))
        self._preview_year = year + delta
        self._preview_month = month
        self.refresh_calendar_preview()

    @staticmethod
    def _format_nav_display_text(label, role, is_rtl):
        if is_rtl:
            return f"{label} ›" if role == "prev" else f"‹ {label}"
        return f"‹ {label}" if role == "prev" else f"{label} ›"

    @staticmethod
    def _relayout_nav_box(box, prev_btn, center, next_btn, is_rtl):
        for child in (prev_btn, center, next_btn):
            parent = child.get_parent()
            if parent is box:
                box.remove(child)
        box.set_direction(Gtk.TextDirection.LTR)
        if is_rtl:
            box.append(next_btn)
            box.append(center)
            box.append(prev_btn)
        else:
            box.append(prev_btn)
            box.append(center)
            box.append(next_btn)

    def _apply_preview_nav(self, view):
        is_rtl = view.get("text_direction") == "rtl"
        ui = view.get("ui") or {}

        self._relayout_nav_box(
            self.preview_year_nav,
            self.preview_prev_year_btn,
            self.preview_year_label,
            self.preview_next_year_btn,
            is_rtl,
        )
        self._relayout_nav_box(
            self.preview_month_nav,
            self.preview_prev_month_btn,
            self.month_title_label,
            self.preview_next_month_btn,
            is_rtl,
        )

        prev_month = ui.get("prev_month_label") or ("ماه قبل" if is_rtl else "Previous month")
        next_month = ui.get("next_month_label") or ("ماه بعد" if is_rtl else "Next month")
        prev_year = ui.get("prev_year_label") or ("سال قبل" if is_rtl else "Previous year")
        next_year = ui.get("next_year_label") or ("سال بعد" if is_rtl else "Next year")

        nav_buttons = (
            (self.preview_prev_month_btn, prev_month, "prev"),
            (self.preview_next_month_btn, next_month, "next"),
            (self.preview_prev_year_btn, prev_year, "prev"),
            (self.preview_next_year_btn, next_year, "next"),
        )
        display = {}
        for button, label, role in nav_buttons:
            text = self._format_nav_display_text(label, role, is_rtl)
            button.set_label(text)
            button.set_tooltip_text(label)
            set_accessible_name(button, label)
            display[f"{role}_{label}"] = text

        # #region agent log
        try:
            import json
            import time
            payload = {
                "sessionId": "830e19",
                "location": "boloot-settings.py:_apply_preview_nav",
                "message": "preview nav applied",
                "data": {
                    "isRtl": is_rtl,
                    "childOrder": "next,center,prev" if is_rtl else "prev,center,next",
                    "buttonStyle": "text",
                    "labels": {
                        "prevMonth": prev_month,
                        "nextMonth": next_month,
                        "prevYear": prev_year,
                        "nextYear": next_year,
                    },
                    "display": display,
                },
                "hypothesisId": "F",
                "timestamp": int(time.time() * 1000),
                "runId": "post-fix",
            }
            with open(
                Path.home() / ".config" / "boloot-calendar" / "debug-nav.log",
                "a",
                encoding="utf-8",
            ) as log_file:
                log_file.write(json.dumps(payload, ensure_ascii=False) + "\n")
        except OSError:
            pass
        # #endregion

    def refresh_calendar_preview(self):
        view = self.fetch_month_view(self._preview_year, self._preview_month)
        self._clear_container(self.calendar_grid)
        if not view:
            self.month_title_label.set_text("")
            self.preview_year_label.set_text("")
            return

        if self._preview_year <= 0:
            self._preview_year = view.get("display_year", view.get("jalali_year", 0))
            self._preview_month = view.get("display_month", view.get("jalali_month", 0))

        self.month_title_label.set_text(view.get("title", view.get("month_name", "")))
        self.preview_year_label.set_text(view.get("year_label", ""))
        self._apply_preview_nav(view)
        appearance = self._live_appearance(view.get("appearance", {}))
        apply_custom = appearance.get("apply_custom_appearance", False)
        if apply_custom:
            self.month_preview_box.add_css_class("boloot-preview-calendar-custom")
        else:
            self.month_preview_box.remove_css_class("boloot-preview-calendar-custom")
        self._update_preview_theme_css(appearance)

        if view.get("text_direction") == "rtl":
            self.calendar_grid.set_direction(Gtk.TextDirection.RTL)
        else:
            self.calendar_grid.set_direction(Gtk.TextDirection.LTR)

        if not self._preview_selected_gregorian:
            for cell in view.get("cells", []):
                if cell.get("is_today") and cell.get("gregorian_date"):
                    self._preview_selected_gregorian = cell["gregorian_date"]
                    break

        weekend_headers = set(view.get("weekend_header_indices", []))
        for col, header in enumerate(view.get("weekday_headers", [])):
            header_label = Gtk.Label(label=header)
            header_label.add_css_class("boloot-preview-weekday")
            if col in weekend_headers:
                header_label.add_css_class("boloot-preview-weekday-weekend")
            else:
                header_label.add_css_class("dim-label")
            self.calendar_grid.attach(header_label, col, 0, 1, 1)

        for idx, cell in enumerate(view.get("cells", [])):
            row = idx // 7 + 1
            col = idx % 7
            self.calendar_grid.attach(
                self._make_day_cell(cell, appearance),
                col,
                row,
                1,
                1,
            )

    def refresh_preview(self):
        text = self.fetch_top_bar_preview() or "—"
        self.preview_label.set_text(text)
        view = self.fetch_calendar_view()
        for css_class in ("boloot-preview-topbar-holiday", "boloot-preview-topbar-weekend"):
            self.preview_label.remove_css_class(css_class)
        if view:
            show_holidays = self.show_holidays_row.get_active()
            if show_holidays and view.get("is_holiday"):
                self.preview_label.add_css_class("boloot-preview-topbar-holiday")
            elif view.get("is_weekend"):
                self.preview_label.add_css_class("boloot-preview-topbar-weekend")
        self.refresh_calendar_preview()

    def _subscribe_settings_changed(self):
        try:
            self._dbus_proxy = Gio.DBusProxy.new_for_bus_sync(
                Gio.BusType.SYSTEM,
                Gio.DBusProxyFlags.NONE,
                None,
                DBUS_NAME,
                DBUS_PATH,
                DBUS_IFACE,
                None,
            )
            self._dbus_proxy.connect("g-signal", self._on_dbus_signal)
        except GLib.Error:
            self._dbus_proxy = None

    def _on_dbus_signal(self, _proxy, _sender, signal_name, _params):
        if signal_name != "SettingsChanged" or self._own_change:
            return
        self.load_settings()
        self.refresh_preview()


if __name__ == "__main__":
    BolootSettingsApp().run(sys.argv)
