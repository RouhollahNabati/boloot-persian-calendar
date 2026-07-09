import Clutter from 'gi://Clutter';
import GObject from 'gi://GObject';
import Meta from 'gi://Meta';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import Pango from 'gi://Pango';

import { Extension, gettext as _ } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as Util from 'resource:///org/gnome/shell/misc/util.js';

const APP_NAME = 'BOLOOT Persian Calendar';
const WEBSITE_LABEL = 'boloot.ir';
const DBUS_NAME = 'org.boloot.Calendar';
const DBUS_PATH = '/org/boloot/Calendar';
const DBUS_IFACE = 'org.boloot.Calendar';

function dbusBus() {
    return Gio.DBus.system;
}

const REFRESH_NORMAL_SEC = 60;
const REFRESH_COUNTDOWN_SEC = 1;
const ENABLE_RETRY_NORMAL = 40;
const ENABLE_RETRY_GREETER = 120;
const ENABLE_RETRY_INTERVAL_MS = 250;
const GREETER_POLL_SEC = 2;
const GREETER_DELAYED_SEC = [2, 5, 10];
const GRID_ROWS = 6;
const GRID_COLS = 7;
const BASE_CELL = 44;
const BASE_CELL_GAP = 4;
const BASE_SECTION_PADDING = 24;
const BASE_HEADER_HEIGHT = 36;
const BASE_NAV_YEAR_HEIGHT = 36;
const BASE_NAV_MONTH_HEIGHT = 44;
const BASE_TODAY_HEIGHT = 36;
const BASE_WEEKDAY_HEIGHT = 32;
const BASE_SECTION_SPACING = 40;
const BASE_HORIZONTAL_PADDING = 32;
const BASE_FOOTER_HEIGHT = 36;
const BASE_HOLIDAYS_MAX_HEIGHT = 58;
const BASE_PRAYER_MAX_HEIGHT = 108;
// Cap the combined footer so multi-line occasions + prayer overflow into ScrollView.
const BASE_EXTRAS_MAX_HEIGHT = 128;
const PRAYER_TABLE_COLS = 3;
const BASE_ICON_SIZE = 18;
const MIN_CELL = 28;
const MAX_CELL = 48;
const DEFAULT_HOLIDAY_COLOR = '#a61e2e';
const DEFAULT_TODAY_COLOR = '#1c71d8';
/** Boloot calendar width as a fraction of the native GNOME dateMenu calendar. */
const NATIVE_CALENDAR_WIDTH_PCT = 0.92;
/** Fallback when native width is unknown: fraction of work-area width. */
const WORK_AREA_WIDTH_PCT = 0.36;
const SELECTED_DAY_FILL_ALPHA = 0.72;
const SELECTED_TODAY_FILL_ALPHA = 0.82;

function setDayCellSelected(button, selected) {
    if (!(button instanceof St.Button))
        return;
    const inner = button._bolootDayBox || button.get_child();
    const baseStyle = button._bolootBaseStyle || '';
    if (selected) {
        button.add_style_pseudo_class('selected');
        button.add_style_class_name('boloot-day-selected');
        if (inner)
            inner.add_style_class_name('boloot-day-inner-selected');
        // Inline fill wins over theme / custom today backgrounds.
        const accent = button._bolootSelectColor || DEFAULT_TODAY_COLOR;
        const alpha = button._bolootSelectAlpha ?? SELECTED_DAY_FILL_ALPHA;
        const fill = rgbaFromHex(accent, alpha);
        button.style = fill
            ? `${baseStyle} background-color: ${fill};`
            : baseStyle;
    } else {
        button.remove_style_pseudo_class('selected');
        button.remove_style_class_name('boloot-day-selected');
        if (inner)
            inner.remove_style_class_name('boloot-day-inner-selected');
        button.style = baseStyle;
    }
}

function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
}

function isGreeter() {
    try {
        return Boolean(Main.sessionMode?.isGreeter);
    } catch (_e) {
        return false;
    }
}

function scaleDimension(base, scale) {
    return Math.round(base * scale);
}

function getTextScalingFactor() {
    try {
        const settings = new Gio.Settings({ schema_id: 'org.gnome.desktop.interface' });
        return settings.get_double('text-scaling-factor') || 1;
    } catch (_e) {
        return 1;
    }
}

function getWorkArea() {
    try {
        const monitor = Main.layoutManager.primaryMonitor;
        return Main.layoutManager.getWorkAreaForMonitor(monitor);
    } catch (_e) {
        return { x: 0, y: 0, width: 1920, height: 1080 };
    }
}

function computeGridHeight(cellSize, cellGap) {
    return GRID_ROWS * cellSize + (GRID_ROWS - 1) * cellGap;
}

function computePopupWidth(cellSize, cellGap) {
    const scale = cellSize / BASE_CELL;
    return GRID_COLS * cellSize + (GRID_COLS - 1) * cellGap +
        scaleDimension(BASE_HORIZONTAL_PADDING, scale);
}

function computeCalendarHeight(cellSize, cellGap) {
    const scale = cellSize / BASE_CELL;
    return scaleDimension(BASE_SECTION_PADDING, scale) +
        scaleDimension(BASE_HEADER_HEIGHT, scale) +
        scaleDimension(BASE_NAV_YEAR_HEIGHT, scale) +
        scaleDimension(BASE_NAV_MONTH_HEIGHT, scale) +
        scaleDimension(BASE_TODAY_HEIGHT, scale) +
        scaleDimension(BASE_WEEKDAY_HEIGHT, scale) +
        scaleDimension(BASE_SECTION_SPACING, scale) +
        computeGridHeight(cellSize, cellGap);
}

function computeDesignDimensions() {
    const calendarHeight = computeCalendarHeight(BASE_CELL, BASE_CELL_GAP);
    const popupWidth = computePopupWidth(BASE_CELL, BASE_CELL_GAP);
    const extrasHeight = BASE_EXTRAS_MAX_HEIGHT + BASE_FOOTER_HEIGHT;
    return { calendarHeight, popupWidth, totalHeight: calendarHeight + extrasHeight };
}

function measureActorWidth(actor) {
    if (!actor)
        return 0;
    // Preferred width works even when the actor is hidden (allocation may be 0).
    try {
        const [, natWidth] = actor.get_preferred_width(-1);
        if (natWidth > 1)
            return Math.round(natWidth);
    } catch (_e) {
        // fall through
    }
    try {
        const width = actor.width;
        if (width > 1)
            return Math.round(width);
    } catch (_e) {
        // ignore
    }
    return 0;
}

function getNativeCalendarWidth(dateMenu) {
    const calendar = dateMenu?._calendar;
    if (!calendar)
        return 0;
    // Only measure the GNOME Calendar widget — not its parent column, which
    // expands to Boloot once the native calendar is hidden.
    return measureActorWidth(calendar);
}

function connectMonitorsChanged(callback) {
    const sources = [];
    try {
        sources.push(Meta.MonitorManager.get());
    } catch (_e) {
        // GNOME 49+: MonitorManager may be unavailable at extension load time.
    }
    if (Main.layoutManager)
        sources.push(Main.layoutManager);

    for (const source of sources) {
        try {
            const id = source.connect('monitors-changed', callback);
            return { source, id };
        } catch (_e) {
            // Try the next source (e.g. MetaDisplay no longer has monitors-changed).
        }
    }
    return null;
}

function computePopupLayout(options = {}) {
    const design = computeDesignDimensions();
    const area = getWorkArea();
    const textScale = getTextScalingFactor();
    const maxH = Math.floor(area.height * 0.88 / textScale);

    const nativeWidth = Math.max(0, Math.round(options.nativeCalendarWidth || 0));
    const workAreaCap = Math.floor(area.width * WORK_AREA_WIDTH_PCT);
    // Size relative to the native GNOME calendar when known so a narrower
    // date-menu column compresses Boloot cells more tightly.
    const maxW = nativeWidth > 0
        ? Math.min(Math.floor(nativeWidth * NATIVE_CALENDAR_WIDTH_PCT), workAreaCap || nativeWidth)
        : workAreaCap;

    const scale = Math.min(1, maxH / design.totalHeight, maxW / design.popupWidth);
    const cellSize = clamp(Math.round(BASE_CELL * scale), MIN_CELL, MAX_CELL);
    const dimScale = cellSize / BASE_CELL;
    const cellGap = Math.max(2, scaleDimension(BASE_CELL_GAP, dimScale));
    let popupWidth = computePopupWidth(cellSize, cellGap);
    if (nativeWidth > 0)
        popupWidth = Math.min(popupWidth, Math.floor(nativeWidth * NATIVE_CALENDAR_WIDTH_PCT));

    const extrasMaxHeight = Math.round(scaleDimension(BASE_EXTRAS_MAX_HEIGHT, dimScale) * textScale);

    const layout = {
        cellSize,
        cellGap,
        gridHeight: computeGridHeight(cellSize, cellGap),
        popupWidth,
        calendarHeight: computeCalendarHeight(cellSize, cellGap),
        headerHeight: scaleDimension(BASE_HEADER_HEIGHT, dimScale),
        navYearHeight: scaleDimension(BASE_NAV_YEAR_HEIGHT, dimScale),
        navMonthHeight: scaleDimension(BASE_NAV_MONTH_HEIGHT, dimScale),
        todayHeight: scaleDimension(BASE_TODAY_HEIGHT, dimScale),
        weekdayHeight: scaleDimension(BASE_WEEKDAY_HEIGHT, dimScale),
        iconSize: clamp(scaleDimension(BASE_ICON_SIZE, dimScale), 14, 22),
        todayIconSize: clamp(scaleDimension(14, dimScale), 12, 18),
        footerHeight: scaleDimension(BASE_FOOTER_HEIGHT, dimScale),
        holidaysMaxHeight: Math.round(scaleDimension(BASE_HOLIDAYS_MAX_HEIGHT, dimScale) * textScale),
        prayerMaxHeight: Math.round(scaleDimension(BASE_PRAYER_MAX_HEIGHT, dimScale) * textScale),
        extrasMaxHeight,
        navButtonSize: cellSize,
        navYearButtonSize: scaleDimension(BASE_NAV_YEAR_HEIGHT, dimScale),
        settingsButtonSize: scaleDimension(BASE_HEADER_HEIGHT, dimScale),
        settingsIconSize: clamp(scaleDimension(16, dimScale), 14, 18),
        scale,
        designTotalHeight: design.totalHeight,
        workAreaHeight: area.height,
        nativeCalendarWidth: nativeWidth,
        textScale,
    };
    return layout;
}

function safeHexColor(color) {
    if (typeof color !== 'string')
        return null;
    const match = color.match(/^#[0-9A-Fa-f]{6}$/);
    return match ? match[0] : null;
}

function hexToRgb(hex) {
    const safe = safeHexColor(hex);
    if (!safe)
        return null;
    return {
        r: parseInt(safe.slice(1, 3), 16),
        g: parseInt(safe.slice(3, 5), 16),
        b: parseInt(safe.slice(5, 7), 16),
    };
}

function rgbaFromHex(hex, alpha) {
    const rgb = hexToRgb(hex);
    if (!rgb)
        return null;
    const a = Math.min(1, Math.max(0, alpha));
    return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${a.toFixed(2)})`;
}

function customDayCellStyle(cell, appearance) {
    if (!cell || !appearance?.apply_custom_appearance)
        return '';

    const todayColor = safeHexColor(appearance.today_color);
    const holidayColor = safeHexColor(appearance.holiday_color);
    const todayBg = appearance.today_background_color
        || (todayColor ? rgbaFromHex(todayColor, 0.22) : null);

    let style = '';
    if (cell.is_today && todayBg)
        style += `background-color: ${todayBg};`;

    if (cell.is_today && cell.is_holiday && holidayColor)
        style += ` color: ${holidayColor}; font-weight: 500;`;
    else if (cell.is_today && todayColor)
        style += ` color: ${todayColor}; font-weight: 600;`;
    else if (cell.is_holiday && holidayColor)
        style += ` color: ${holidayColor}; font-weight: 500;`;
    else if (cell.is_weekend && holidayColor)
        style += ` color: ${holidayColor}; font-weight: 500;`;

    return style;
}

const TOP_BAR_CLOCK_CLASSES = ['boloot-clock-holiday', 'boloot-clock-weekend'];

const WEEKEND_WEEKDAY_NAMES = {
    iran: ['جمعه'],
    afghanistan: ['پنج\u200cشنبه', 'پنجشنبه', 'جمعه'],
    tajikistan: ['شنبه', 'یک\u200cشنبه', 'یکشنبه'],
};

function isWeekendToday(calendarView, settings) {
    if (calendarView?.is_weekend != null)
        return Boolean(calendarView.is_weekend);
    const weekday = calendarView?.weekday || '';
    if (!weekday)
        return false;
    const country = settings?.calendar?.country || 'iran';
    const names = WEEKEND_WEEKDAY_NAMES[country] || WEEKEND_WEEKDAY_NAMES.iran;
    return names.some(name => weekday.includes(name));
}

function isHolidayToday(calendarView, settings, holidaysToday = []) {
    if (!settings || settings?.calendar?.show_holidays === false)
        return false;
    if (calendarView?.is_holiday != null)
        return Boolean(calendarView.is_holiday);
    return Array.isArray(holidaysToday) && holidaysToday.length > 0;
}

function applyTopBarDayStyle(display, calendarView, settings, holidaysToday = []) {
    if (!display)
        return { isHoliday: false, isWeekend: false, appliedClass: null };

    for (const cls of TOP_BAR_CLOCK_CLASSES) {
        if (display.has_style_class_name?.(cls))
            display.remove_style_class_name(cls);
    }
    display.style = '';

    const isHoliday = isHolidayToday(calendarView, settings, holidaysToday);
    const isWeekend = isWeekendToday(calendarView, settings);
    let appliedClass = null;
    const accentColor = accentHolidayColor(null, settings);

    if (isHoliday) {
        appliedClass = 'boloot-clock-holiday';
        display.add_style_class_name('boloot-clock-holiday');
        display.style = `color: ${accentColor}; font-weight: 500;`;
    } else if (isWeekend) {
        appliedClass = 'boloot-clock-weekend';
        display.add_style_class_name('boloot-clock-weekend');
        display.style = `color: ${accentColor}; font-weight: 500;`;
    }

    return { isHoliday, isWeekend, appliedClass, inlineStyle: display.style || null };
}

function safeFontFamily(fontFamily) {
    if (typeof fontFamily !== 'string' || !fontFamily)
        return null;
    if (/[";<>]/.test(fontFamily))
        return null;
    return fontFamily;
}

function dbusCall(method, params = null, replyType = null) {
    try {
        return dbusBus().call_sync(
            DBUS_NAME,
            DBUS_PATH,
            DBUS_IFACE,
            method,
            params,
            replyType,
            Gio.DBusCallFlags.NONE,
            -1,
            null,
        );
    } catch (e) {
        log(`${APP_NAME}: D-Bus ${method} failed: ${e}`);
        return null;
    }
}

function fetchTopBarText() {
    const reply = dbusCall('GetTopBarText', null, new GLib.VariantType('(s)'));
    if (!reply)
        return null;
    return reply.deepUnpack()[0];
}

function fetchJson(method, params = null) {
    const reply = dbusCall(method, params, new GLib.VariantType('(s)'));
    if (!reply)
        return null;
    try {
        return JSON.parse(reply.deepUnpack()[0]);
    } catch (e) {
        log(`${APP_NAME}: JSON parse error for ${method}: ${e}`);
        return null;
    }
}

function fetchSettings() {
    const reply = dbusCall('GetSettings', null, new GLib.VariantType('(s)'));
    if (!reply)
        return null;
    try {
        return JSON.parse(reply.deepUnpack()[0]);
    } catch (e) {
        return null;
    }
}

let _settingsCache = null;
let _settingsCacheTime = 0;

function fetchSettingsCached(maxAgeUs = 5_000_000) {
    const now = GLib.get_monotonic_time();
    if (_settingsCache && (now - _settingsCacheTime) < maxAgeUs)
        return _settingsCache;
    _settingsCache = fetchSettings();
    _settingsCacheTime = now;
    return _settingsCache;
}

function invalidateSettingsCache() {
    _settingsCache = null;
    _settingsCacheTime = 0;
}

function refreshIntervalSec() {
    const settings = fetchSettingsCached();
    if (!settings)
        return REFRESH_NORMAL_SEC;
    const prayer = settings.prayer || {};
    if (prayer.enabled && prayer.show_in_top_bar && prayer.display_mode === 'countdown')
        return REFRESH_COUNTDOWN_SEC;
    return REFRESH_NORMAL_SEC;
}

function prefersReducedMotion() {
    try {
        const settings = new Gio.Settings({ schema_id: 'org.gnome.desktop.interface' });
        return !settings.get_boolean('enable-animations');
    } catch (e) {
        return false;
    }
}

function systemFontFamily() {
    try {
        const settings = new Gio.Settings({ schema_id: 'org.gnome.desktop.interface' });
        const name = settings.get_string('font-name');
        return name.replace(/\s+\d+(\.\d+)?$/, '').trim();
    } catch (e) {
        return '';
    }
}

function effectiveNumerals(settings) {
    const cal = settings?.calendar || {};
    if (cal.calendar_type === 'gregorian')
        return 'latin';
    return cal.numerals || 'persian';
}

function formatCountdownLabel(remainingSeconds, numerals) {
    const clamped = Math.max(0, remainingSeconds);
    const hours = Math.floor(clamped / 3600);
    const minutes = Math.floor((clamped % 3600) / 60);
    const seconds = clamped % 60;
    if (numerals === 'persian') {
        const toFa = n => String(n).replace(/\d/g, d => '۰۱۲۳۴۵۶۷۸۹'[d]);
        return `${toFa(hours)}س ${toFa(minutes)}د ${toFa(seconds)}ث`;
    }
    return `${hours}h ${minutes}m ${seconds}s`;
}

function resolveFontFamily(appearance) {
    const family = appearance?.font_family;
    if (appearance?.use_system_theme && (!family || family === 'system')) {
        const sys = systemFontFamily();
        if (sys)
            return sys;
    }
    if (!family || family === 'system')
        return 'Vazirmatn';
    return family;
}

function resolveSettingsCommand() {
    const home = GLib.getenv('HOME');
    const candidates = [
        `${home}/.local/bin/boloot-calendar-settings`,
        '/usr/local/bin/boloot-calendar-settings',
        '/usr/bin/boloot-calendar-settings',
    ];
    for (const path of candidates) {
        if (GLib.file_test(path, GLib.FileTest.IS_EXECUTABLE))
            return path;
    }
    return null;
}

function openBolootSettings(dateMenu) {
    dateMenu?.menu?.close();
    // GNOME API differs between versions; avoid hard-failing on missing method.
    try {
        if (typeof Main.panel?.closeCalendar === 'function')
            Main.panel.closeCalendar();
    } catch (_e) {
        // Ignore: menu was already closed above.
    }

    const desktopApp = Gio.DesktopAppInfo.new('org.boloot.Calendar.desktop');
    if (desktopApp) {
        try {
            const context = global.create_app_launch_context(0, -1);
            desktopApp.launch([], context);
            return;
        } catch (e) {
            log(`${APP_NAME}: desktop launch failed: ${e.message}`);
        }
    }

    const cmd = resolveSettingsCommand();
    if (cmd) {
        try {
            Util.spawn([cmd]);
            return;
        } catch (e) {
            log(`${APP_NAME}: failed to launch settings from absolute path: ${e.message}`);
        }
    }

    try {
        Util.spawnCommandLine('boloot-calendar-settings');
        return;
    } catch (e) {
        log(`${APP_NAME}: boloot-calendar-settings not found in PATH: ${e.message}`);
    }

    log(`${APP_NAME}: unable to launch settings app`);
}

function makeIconButton(iconName, accessibleName, styleClass = 'boloot-nav', iconSize = BASE_ICON_SIZE) {
    const button = new St.Button({
        style_class: styleClass,
        can_focus: true,
        x_expand: false,
        x_align: Clutter.ActorAlign.CENTER,
        y_align: Clutter.ActorAlign.CENTER,
    });
    button.set_child(new St.Icon({ icon_name: iconName, icon_size: iconSize }));
    button.accessible_name = accessibleName;
    return button;
}

function makeTextNavButton(styleClass = 'boloot-nav boloot-nav-text') {
    const label = new St.Label({ style_class: 'boloot-nav-label' });
    label.clutter_text.ellipsize = Pango.EllipsizeMode.NONE;
    const button = new St.Button({
        style_class: styleClass,
        can_focus: true,
        x_expand: false,
        child: label,
    });
    return button;
}

function navButtonLabel(button) {
    const child = button.get_child();
    return child instanceof St.Label ? child : null;
}

function setTextNavButton(button, displayText, accessibleName) {
    const label = navButtonLabel(button);
    if (label)
        label.text = displayText;
    button.accessible_name = accessibleName;
    button.tooltip_text = accessibleName;
}

function formatNavDisplayText(label, role, isRtl) {
    if (isRtl)
        return role === 'prev' ? `${label} ›` : `‹ ${label}`;
    return role === 'prev' ? `‹ ${label}` : `${label} ›`;
}

function relayoutNavRow(box, prevBtn, centerWidget, nextBtn, isRtl) {
    if (!box || !prevBtn || !centerWidget || !nextBtn)
        return;

    for (const child of [prevBtn, centerWidget, nextBtn]) {
        if (child.get_parent() === box)
            box.remove_child(child);
    }

    box.text_direction = Clutter.TextDirection.LTR;
    if (isRtl) {
        box.add_child(nextBtn);
        box.add_child(centerWidget);
        box.add_child(prevBtn);
    } else {
        box.add_child(prevBtn);
        box.add_child(centerWidget);
        box.add_child(nextBtn);
    }
}

function makeEllipsisLabel(props) {
    const clean = {};
    for (const [key, value] of Object.entries(props)) {
        if (value !== undefined)
            clean[key] = value;
    }
    const label = new St.Label(clean);
    label.clutter_text.ellipsize = Pango.EllipsizeMode.END;
    if ('line_wrap' in label.clutter_text)
        label.clutter_text.line_wrap = false;
    return label;
}

function makeDivider() {
    return new St.Widget({ style_class: 'boloot-divider', x_expand: true });
}

function makePrayerCell(entry, isNext, useSystemTheme, prayerColor) {
    const cell = new St.BoxLayout({
        style_class: isNext ? 'boloot-prayer-cell boloot-prayer-cell-next' : 'boloot-prayer-cell',
        x_expand: true,
    });
    const nameLabel = new St.Label({
        text: entry.label,
        style_class: 'boloot-prayer-name',
        x_expand: true,
        x_align: Clutter.ActorAlign.END,
    });
    nameLabel.clutter_text.ellipsize = Pango.EllipsizeMode.END;
    const timeLabel = new St.Label({
        text: entry.time,
        style_class: 'boloot-prayer-time',
        x_align: Clutter.ActorAlign.START,
    });
    cell.add_child(nameLabel);
    cell.add_child(timeLabel);

    if (!useSystemTheme && prayerColor) {
        const colorStyle = `color: ${prayerColor};`;
        const weight = isNext ? ' font-weight: 600;' : '';
        nameLabel.style = `${colorStyle}${weight} opacity: 0.92;`;
        timeLabel.style = `${colorStyle}${weight}`;
    } else if (isNext) {
        nameLabel.style = 'font-weight: 600;';
        timeLabel.style = 'font-weight: 600;';
    }
    return cell;
}

function buildPrayerTable(entries, cols, nextName, useSystemTheme, prayerColor) {
    const table = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-prayer-table',
        x_expand: true,
    });
    for (let i = 0; i < entries.length; i += cols) {
        const row = new St.BoxLayout({
            style_class: 'boloot-prayer-table-row',
            x_expand: true,
        });
        for (let j = 0; j < cols; j++) {
            const entry = entries[i + j];
            if (entry) {
                const isNext = Boolean(nextName && entry.name === nextName);
                row.add_child(makePrayerCell(entry, isNext, useSystemTheme, prayerColor));
            } else {
                row.add_child(new St.Widget({
                    style_class: 'boloot-prayer-cell boloot-prayer-cell-empty',
                    x_expand: true,
                }));
            }
        }
        table.add_child(row);
    }
    return table;
}

function accentHolidayColor(appearance, settings) {
    return safeHexColor(appearance?.holiday_color)
        || safeHexColor(settings?.appearance?.holiday_color)
        || DEFAULT_HOLIDAY_COLOR;
}

function accentTodayColor(appearance, settings) {
    return safeHexColor(appearance?.today_color)
        || safeHexColor(settings?.appearance?.today_color)
        || DEFAULT_TODAY_COLOR;
}

function systemDayPrimaryStyle(cell, appearance, showHolidays) {
    if (!cell || appearance?.apply_custom_appearance)
        return '';

    const holidayColor = accentHolidayColor(appearance);
    const todayColor = accentTodayColor(appearance);
    const isHoliday = showHolidays && cell.is_holiday;

    if (cell.is_today && isHoliday)
        return `color: ${holidayColor}; font-weight: 500;`;
    if (cell.is_today)
        return `color: ${todayColor}; font-weight: 600;`;
    if (isHoliday || cell.is_weekend)
        return `color: ${holidayColor}; font-weight: 500;`;
    return '';
}

function systemDaySecondaryStyle(cell, appearance, showHolidays) {
    if (!cell || appearance?.apply_custom_appearance)
        return '';

    const holidayColor = accentHolidayColor(appearance);
    const isHoliday = showHolidays && cell.is_holiday;
    if (isHoliday || cell.is_weekend)
        return `color: ${holidayColor}; opacity: 0.75;`;
    return '';
}

function gregorianDateFromIso(iso) {
    if (!iso)
        return null;
    const parts = iso.split('-').map(Number);
    if (parts.length !== 3 || parts.some(n => Number.isNaN(n)))
        return null;
    const [year, month, day] = parts;
    return new Date(year, month - 1, day, 12, 0, 0);
}

function gridColumnOrder(isRtl) {
    const cols = [];
    for (let c = 0; c < GRID_COLS; c++)
        cols.push(c);
    return isRtl ? cols.reverse() : cols;
}

function dayCellClasses(cell, appearance) {
    const applyCustom = Boolean(appearance?.apply_custom_appearance);
    let classes = 'boloot-day-cell';
    if (!cell?.is_current_month)
        classes += ' boloot-day-outside';
    if (cell?.is_weekend)
        classes += ' boloot-weekend';
    if (!applyCustom) {
        if (cell?.is_today)
            classes += ' boloot-today';
        if (cell?.is_holiday)
            classes += ' boloot-holiday';
    } else {
        if (cell?.is_today)
            classes += ' boloot-today-custom';
        if (cell?.is_holiday)
            classes += ' boloot-holiday-custom';
    }
    return classes;
}

function createDayCell(cell, layout, appearance, showHolidays = true, isSelected = false, onClicked = null) {
    const cellSize = layout?.cellSize ?? BASE_CELL;
    if (!cell) {
        const empty = new St.Widget({
            style_class: 'boloot-day-cell boloot-day-empty',
            x_expand: true,
        });
        empty.set_size(cellSize, cellSize);
        return empty;
    }

    const dayBox = new St.BoxLayout({
        vertical: true,
        x_expand: true,
        y_align: Clutter.ActorAlign.CENTER,
        style_class: 'boloot-day-inner',
    });

    let primaryStyle = appearance?.apply_custom_appearance && cell?.is_today
        ? 'font-weight: 600;'
        : appearance?.apply_custom_appearance && cell?.is_holiday
            ? 'font-weight: 500;'
            : '';
    if (!primaryStyle && !appearance?.apply_custom_appearance)
        primaryStyle = systemDayPrimaryStyle(cell, appearance, showHolidays);
    const primaryLabel = new St.Label({
        text: cell?.day_label || '',
        style_class: 'boloot-day-primary',
    });
    if (primaryStyle)
        primaryLabel.style = primaryStyle;
    dayBox.add_child(primaryLabel);

    if (cell?.secondary_label) {
        const secondaryLabel = new St.Label({
            text: cell.secondary_label,
            style_class: 'boloot-day-secondary',
        });
        const secondaryStyle = systemDaySecondaryStyle(cell, appearance, showHolidays);
        if (secondaryStyle)
            secondaryLabel.style = secondaryStyle;
        dayBox.add_child(secondaryLabel);
    }

    const tip = cell?.tooltip
        || (cell?.is_holiday && cell?.holiday_names?.length ? cell.holiday_names.join('، ') : '');

    const button = new St.Button({
        style_class: dayCellClasses(cell, appearance),
        can_focus: true,
        x_expand: true,
        y_align: Clutter.ActorAlign.CENTER,
        reactive: true,
    });
    button.set_size(cellSize, cellSize);

    let cellStyle = customDayCellStyle(cell, appearance);
    button._bolootBaseStyle = cellStyle || '';
    button._bolootSelectColor = appearance?.apply_custom_appearance
        ? (safeHexColor(appearance.today_color) || DEFAULT_TODAY_COLOR)
        : DEFAULT_TODAY_COLOR;
    button._bolootSelectAlpha = cell?.is_today
        ? SELECTED_TODAY_FILL_ALPHA
        : SELECTED_DAY_FILL_ALPHA;
    button.style = button._bolootBaseStyle;

    button.set_child(dayBox);
    button._bolootDayBox = dayBox;

    setDayCellSelected(button, isSelected);

    if (tip) {
        button.tooltip_text = tip;
        button.accessible_name = tip;
    } else if (cell?.day_label) {
        button.accessible_name = cell.day_label;
    }

    if (onClicked) {
        button.connect('clicked', () => {
            onClicked(button);
        });
    }

    return button;
}

/** Jalali calendar body injected into GNOME dateMenu popup. */
const BolootMenuSection = GObject.registerClass(
class BolootMenuSection extends St.BoxLayout {
    _init() {
        super._init({ vertical: true, style_class: 'boloot-section', x_expand: true });

        const settingsRow = new St.BoxLayout({
            style_class: 'boloot-header-row',
            x_expand: true,
        });
        this._settingsBtn = makeIconButton(
            'preferences-system-symbolic',
            _('تنظیمات BOLOOT'),
            'boloot-nav boloot-nav-settings',
        );
        this._settingsClickId = 0;
        settingsRow.add_child(this._settingsBtn);
        settingsRow.add_child(new St.Widget({ x_expand: true }));
        this.add_child(settingsRow);

        const yearNavBox = new St.BoxLayout({
            style_class: 'boloot-nav-row boloot-year-nav',
            x_expand: true,
        });
        this._yearNavBox = yearNavBox;
        this._prevYearBtn = makeTextNavButton('boloot-nav boloot-nav-year boloot-nav-text');
        this._yearLabel = new St.Label({
            style_class: 'boloot-year-title',
            x_expand: true,
            x_align: Clutter.ActorAlign.CENTER,
        });
        this._yearLabel.clutter_text.ellipsize = Pango.EllipsizeMode.END;
        this._nextYearBtn = makeTextNavButton('boloot-nav boloot-nav-year boloot-nav-text');
        this._prevYearBtn.connect('clicked', () => this._changeYear(-1));
        this._nextYearBtn.connect('clicked', () => this._changeYear(1));
        yearNavBox.add_child(this._prevYearBtn);
        yearNavBox.add_child(this._yearLabel);
        yearNavBox.add_child(this._nextYearBtn);
        this.add_child(yearNavBox);

        const navBox = new St.BoxLayout({
            style_class: 'boloot-nav-row boloot-month-nav',
            x_expand: true,
        });
        this._monthNavBox = navBox;
        this._prevBtn = makeTextNavButton('boloot-nav boloot-nav-text');
        this._titleLabel = new St.Label({
            style_class: 'boloot-title',
            x_expand: true,
            x_align: Clutter.ActorAlign.CENTER,
        });
        this._titleLabel.clutter_text.ellipsize = Pango.EllipsizeMode.END;
        this._nextBtn = makeTextNavButton('boloot-nav boloot-nav-text');
        this._prevBtn.connect('clicked', () => this._changeMonth(-1));
        this._nextBtn.connect('clicked', () => this._changeMonth(1));
        navBox.add_child(this._prevBtn);
        navBox.add_child(this._titleLabel);
        navBox.add_child(this._nextBtn);
        this.add_child(navBox);

        this._todayBtn = new St.Button({
            style_class: 'boloot-nav boloot-today-btn',
            can_focus: true,
            x_align: Clutter.ActorAlign.CENTER,
        });
        const todayBox = new St.BoxLayout({ style_class: 'boloot-today-inner' });
        todayBox.spacing = 6;
        this._todayIcon = new St.Icon({ icon_name: 'view-calendar-today-symbolic', icon_size: 14 });
        todayBox.add_child(this._todayIcon);
        this._todayLabel = new St.Label({ text: '' });
        todayBox.add_child(this._todayLabel);
        this._todayBtn.set_child(todayBox);
        this._todayBtn.accessible_name = _('برو به امروز');
        this._todayBtn.connect('clicked', () => this._goToday());
        this.add_child(this._todayBtn);

        this._headerBox = new St.BoxLayout({ style_class: 'boloot-weekdays' });
        this.add_child(this._headerBox);

        this._gridBox = new St.BoxLayout({ vertical: true, style_class: 'boloot-grid' });
        this.add_child(this._gridBox);

        this._dividerHolidays = null;
        this._holidayLabel = null;
        this._dividerPrayer = null;
        this._prayerBox = null;

        this._displayYear = 0;
        this._displayMonth = 0;
        this._monthView = null;
        this._reducedMotion = prefersReducedMotion();
        this._cachedSettings = null;
        this._prayerSchedule = null;
        this._countdownAnchor = null;
        this._nextPrayerLabel = null;
        this._prayerRefreshTimer = 0;
        this._layout = computePopupLayout();
        this._renderGeneration = 0;
        this._selectedGregorian = null;
        this._dateMenu = null;
        this._dayButtons = [];
        this._isRtl = false;
    }

    setDateMenu(dateMenu) {
        this._dateMenu = dateMenu;
    }

    setLayout(layout) {
        this._layout = layout;
        this._applyLayoutToNav();
        if (this._monthView)
            this._renderMonth();
    }

    setSettingsAction(callback) {
        if (this._settingsClickId) {
            this._settingsBtn.disconnect(this._settingsClickId);
            this._settingsClickId = 0;
        }
        this._settingsClickId = this._settingsBtn.connect('clicked', callback);
    }

    updateSettingsAccessibleName(name) {
        if (name && this._settingsBtn)
            this._settingsBtn.accessible_name = name;
    }

    _applyTextDirection(isRtl) {
        this._isRtl = isRtl;
        const dir = isRtl ? Clutter.TextDirection.RTL : Clutter.TextDirection.LTR;
        const actors = [
            this._headerBox,
            this._gridBox,
        ];
        for (const actor of actors) {
            if (actor)
                actor.text_direction = dir;
        }
    }

    _applyNavPresentation(isRtl, ui = {}) {
        relayoutNavRow(
            this._yearNavBox,
            this._prevYearBtn,
            this._yearLabel,
            this._nextYearBtn,
            isRtl,
        );
        relayoutNavRow(
            this._monthNavBox,
            this._prevBtn,
            this._titleLabel,
            this._nextBtn,
            isRtl,
        );

        const labels = {
            prevMonth: ui.prev_month_label || (isRtl ? _('ماه قبل') : 'Previous month'),
            nextMonth: ui.next_month_label || (isRtl ? _('ماه بعد') : 'Next month'),
            prevYear: ui.prev_year_label || (isRtl ? _('سال قبل') : 'Previous year'),
            nextYear: ui.next_year_label || (isRtl ? _('سال بعد') : 'Next year'),
        };

        setTextNavButton(
            this._prevBtn,
            formatNavDisplayText(labels.prevMonth, 'prev', isRtl),
            labels.prevMonth,
        );
        setTextNavButton(
            this._nextBtn,
            formatNavDisplayText(labels.nextMonth, 'next', isRtl),
            labels.nextMonth,
        );
        setTextNavButton(
            this._prevYearBtn,
            formatNavDisplayText(labels.prevYear, 'prev', isRtl),
            labels.prevYear,
        );
        setTextNavButton(
            this._nextYearBtn,
            formatNavDisplayText(labels.nextYear, 'next', isRtl),
            labels.nextYear,
        );

        for (const button of [this._prevBtn, this._nextBtn, this._prevYearBtn, this._nextYearBtn]) {
            button.text_direction = isRtl ? Clutter.TextDirection.RTL : Clutter.TextDirection.LTR;
        }
    }

    _syncGnomeCalendar(date) {
        if (!date || !this._dateMenu?._calendar)
            return;
        try {
            const cal = this._dateMenu._calendar;
            if (typeof cal.setDate === 'function') {
                if (cal.setDate.length >= 2)
                    cal.setDate(date, false);
                else
                    cal.setDate(date);
            }
        } catch (e) {
            log(`${APP_NAME}: sync GNOME calendar failed: ${e}`);
        }
    }

    _cellForSelectedGregorian() {
        if (!this._selectedGregorian || !this._monthView?.cells)
            return null;
        return this._monthView.cells.find(c => c?.gregorian_date === this._selectedGregorian) ?? null;
    }

    _ensureDefaultSelection() {
        if (this._selectedGregorian)
            return;
        const todayCell = this._monthView?.cells?.find(c => c?.is_today);
        if (todayCell?.gregorian_date) {
            this._selectedGregorian = todayCell.gregorian_date;
            this._syncGnomeCalendar(gregorianDateFromIso(todayCell.gregorian_date));
        }
    }

    _updateDaySelection() {
        for (const entry of this._dayButtons) {
            const shouldSelect = entry.gregorian === this._selectedGregorian;
            setDayCellSelected(entry.button, shouldSelect);
        }
    }

    _updateHolidayFooter() {
        if (!this._holidayLabel || !this._monthView)
            return;

        const settings = this._cachedSettings || this._loadSettings();
        const showHolidays = settings?.calendar?.show_holidays !== false;
        const ui = this._monthView.ui || {};
        const holidaysPrefix = ui.holidays_prefix || '';

        if (!showHolidays) {
            this._holidayLabel.text = '';
            if (this._holidaysBar)
                this._holidaysBar.visible = false;
            return;
        }

        let holidayNames = [];
        if (this._selectedGregorian) {
            const cell = this._cellForSelectedGregorian();
            if (cell?.holiday_names?.length)
                holidayNames = [...new Set(cell.holiday_names)];
        }

        const hasHolidays = holidayNames.length > 0;
        if (hasHolidays) {
            // One occasion per line so ScrollView can show many without clipping.
            this._holidayLabel.text = holidaysPrefix
                ? `${holidaysPrefix}\n${holidayNames.join('\n')}`
                : holidayNames.join('\n');
            if ('line_wrap' in this._holidayLabel.clutter_text)
                this._holidayLabel.clutter_text.line_wrap = true;
            this._holidayLabel.clutter_text.ellipsize = Pango.EllipsizeMode.NONE;
        } else {
            this._holidayLabel.text = '';
        }
        if (this._holidaysBar)
            this._holidaysBar.visible = showHolidays && hasHolidays;
    }

    _onDayClicked(cell, button) {
        if (!cell?.gregorian_date)
            return;

        this._selectedGregorian = cell.gregorian_date;

        if (!cell.is_current_month) {
            this._displayYear = cell.jalali_year;
            this._displayMonth = cell.jalali_month;
            this.refresh();
            return;
        }

        this._syncGnomeCalendar(gregorianDateFromIso(cell.gregorian_date));
        this._updateDaySelection();
        this._updateHolidayFooter();
        if (button)
            button.grab_key_focus();
    }

    _applyLayoutToNav() {
        const layout = this._layout;
        if (!layout)
            return;

        const settingsSize = layout.settingsButtonSize ?? layout.navYearButtonSize;
        this._settingsBtn.set_size(settingsSize, settingsSize);
        const setIconSize = (button, size) => {
            const icon = button.get_child();
            if (icon instanceof St.Icon)
                icon.icon_size = size;
        };
        setIconSize(this._settingsBtn, layout.settingsIconSize ?? layout.iconSize);
        this._prevYearBtn.height = layout.navYearHeight;
        this._nextYearBtn.height = layout.navYearHeight;
        this._prevBtn.height = layout.navMonthHeight ?? layout.navButtonSize;
        this._nextBtn.height = layout.navMonthHeight ?? layout.navButtonSize;
        this._todayBtn.height = layout.todayHeight;
        if (this._todayIcon)
            this._todayIcon.icon_size = layout.todayIconSize;

        this._gridBox.height = layout.gridHeight;
    }

    _loadSettings() {
        this._cachedSettings = fetchSettingsCached();
        return this._cachedSettings;
    }

    _prayerScheduleFromView() {
        if (this._monthView?.prayer)
            return this._monthView.prayer;
        return fetchJson('GetPrayerTimes');
    }

    _remainingSeconds() {
        if (!this._countdownAnchor)
            return 0;
        const elapsed = Math.floor(
            (GLib.get_monotonic_time() - this._countdownAnchor.monotonicUs) / 1_000_000,
        );
        return this._countdownAnchor.remaining - elapsed;
    }

    _tickCountdown() {
        if (!this.visible || !this._nextPrayerLabel || !this._countdownAnchor)
            return GLib.SOURCE_CONTINUE;

        const remaining = this._remainingSeconds();
        const settings = this._cachedSettings || this._loadSettings();
        const numerals = effectiveNumerals(settings);
        this._nextPrayerLabel.text =
            `${this._countdownAnchor.prefix} ${this._countdownAnchor.label} ${formatCountdownLabel(remaining, numerals)}`;

        if (remaining <= 0)
            this.refresh();
        return GLib.SOURCE_CONTINUE;
    }

    _stopPrayerTimers() {
        if (this._countdownTimer) {
            GLib.source_remove(this._countdownTimer);
            this._countdownTimer = 0;
        }
        if (this._prayerRefreshTimer) {
            GLib.source_remove(this._prayerRefreshTimer);
            this._prayerRefreshTimer = 0;
        }
        this._countdownAnchor = null;
        this._nextPrayerLabel = null;
    }

    refresh() {
        const year = this._displayYear > 0 ? this._displayYear : 0;
        const month = this._displayMonth > 0 ? this._displayMonth : 0;
        const params = new GLib.Variant('(ii)', [year, month]);
        this._monthView = fetchJson('GetMonthView', params);
        if (!this._monthView)
            return;

        this._displayYear = this._monthView.display_year ?? this._monthView.jalali_year;
        this._displayMonth = this._monthView.display_month ?? this._monthView.jalali_month;
        this._loadSettings();

        this._renderGeneration += 1;
        const generation = this._renderGeneration;
        GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            if (generation !== this._renderGeneration || !this.visible)
                return GLib.SOURCE_REMOVE;
            try {
                this._renderMonth();
                this._renderPrayer(true);
            } catch (e) {
                log(`${APP_NAME}: popup render failed: ${e}`);
            }
            return GLib.SOURCE_REMOVE;
        });
    }

    _renderMonth() {
        const view = this._monthView;
        if (!view)
            return;

        this._yearLabel.text = view.year_label || '';
        this._titleLabel.text = view.month_name || view.title;

        const ui = view.ui || {};
        if (ui.today_button)
            this._todayLabel.text = ui.today_button;

        const settings = this._cachedSettings || this._loadSettings();
        const showHolidays = settings?.calendar?.show_holidays !== false;
        const appearance = view.appearance || {};
        const applyCustom = Boolean(appearance.apply_custom_appearance);

        let sectionStyle = '';
        const fontSize = Number(appearance.font_size_pt);
        if (fontSize >= 8 && fontSize <= 24)
            sectionStyle += `font-size: ${fontSize}pt;`;
        const fontFamily = safeFontFamily(resolveFontFamily(appearance));
        if (fontFamily)
            sectionStyle += ` font-family: "${fontFamily}";`;
        if (applyCustom) {
            const bgColor = safeHexColor(appearance.background_color);
            const textColor = safeHexColor(appearance.text_color);
            if (bgColor)
                sectionStyle += ` background-color: ${bgColor};`;
            if (textColor)
                sectionStyle += ` color: ${textColor};`;
        }
        this.style = sectionStyle;

        const isRtl = view.text_direction === 'rtl';
        this._applyTextDirection(isRtl);
        this._applyNavPresentation(isRtl, ui);
        const colOrder = gridColumnOrder(isRtl);

        this._headerBox.destroy_all_children();
        const weekendHeaders = new Set(view.weekend_header_indices || []);
        for (const col of colOrder) {
            const header = view.weekday_headers[col];
            const isWeekendHeader = weekendHeaders.has(col);
            this._headerBox.add_child(new St.Label({
                text: header,
                style_class: isWeekendHeader
                    ? 'boloot-weekday boloot-weekday-weekend'
                    : 'boloot-weekday',
                x_expand: true,
            }));
        }

        this._gridBox.destroy_all_children();
        this._dayButtons = [];
        const layout = this._layout || computePopupLayout();
        this._applyLayoutToNav();
        for (let row = 0; row < GRID_ROWS; row++) {
            const rowBox = new St.BoxLayout({
                style_class: 'boloot-grid-row',
            });
            rowBox.spacing = layout.cellGap;
            for (const col of colOrder) {
                const cell = view.cells[row * GRID_COLS + col] ?? null;
                const isSelected = Boolean(cell?.gregorian_date &&
                    cell.gregorian_date === this._selectedGregorian);
                const dayWidget = createDayCell(
                    cell,
                    layout,
                    appearance,
                    showHolidays,
                    isSelected,
                    cell ? button => this._onDayClicked(cell, button) : null,
                );
                if (cell?.gregorian_date) {
                    this._dayButtons.push({
                        gregorian: cell.gregorian_date,
                        button: dayWidget,
                    });
                }
                rowBox.add_child(dayWidget);
            }
            this._gridBox.add_child(rowBox);
        }

        this._ensureDefaultSelection();
        this._updateDaySelection();
        this._updateHolidayFooter();

        if (this._selectedGregorian) {
            this._syncGnomeCalendar(gregorianDateFromIso(this._selectedGregorian));
            const selectedEntry = this._dayButtons.find(
                entry => entry.gregorian === this._selectedGregorian,
            );
            if (selectedEntry?.button)
                selectedEntry.button.grab_key_focus();
        }
    }

    attachFooterWidgets(holidaysBar, prayerBar) {
        this._holidaysBar = holidaysBar.bar;
        this._dividerHolidays = holidaysBar.divider;
        this._holidayLabel = holidaysBar.label;
        this._prayerBarContainer = prayerBar.bar;
        this._dividerPrayer = prayerBar.divider;
        this._prayerBox = prayerBar.box;
    }

    _renderPrayer(forceRefresh = false) {
        this._stopPrayerTimers();
        if (!this._prayerBox)
            return;
        this._prayerBox.destroy_all_children();

        const settings = this._cachedSettings || this._loadSettings();
        const prayerCfg = settings?.prayer || {};
        if (prayerCfg.enabled === false || prayerCfg.show_in_popup === false) {
            this._prayerBox.visible = false;
            if (this._dividerPrayer)
                this._dividerPrayer.visible = false;
            if (this._prayerBarContainer)
                this._prayerBarContainer.visible = false;
            this._prayerSchedule = null;
            return;
        }

        if (forceRefresh || !this._prayerSchedule)
            this._prayerSchedule = this._prayerScheduleFromView();
        const schedule = this._prayerSchedule;
        if (!schedule) {
            this._prayerBox.visible = false;
            if (this._dividerPrayer)
                this._dividerPrayer.visible = false;
            if (this._prayerBarContainer)
                this._prayerBarContainer.visible = false;
            return;
        }

        this._prayerBox.visible = true;
        if (this._dividerPrayer)
            this._dividerPrayer.visible = true;
        if (this._prayerBarContainer)
            this._prayerBarContainer.visible = true;

        const appearance = this._monthView?.appearance || {};
        const ui = this._monthView?.ui || {};
        const useSystemTheme = Boolean(appearance.use_system_theme);
        const titleLabel = makeEllipsisLabel({
            text: ui.prayer_section_title || '',
            style_class: 'boloot-section-title',
        });
        const prayerColor = safeHexColor(appearance.prayer_color);
        if (!useSystemTheme && prayerColor) {
            titleLabel.style = `color: ${prayerColor};`;
        }
        this._prayerBox.add_child(titleLabel);

        const isRtl = this._monthView?.text_direction === 'rtl';
        this._prayerBox.text_direction = isRtl
            ? Clutter.TextDirection.RTL
            : Clutter.TextDirection.LTR;

        const nextName = schedule.next?.name ?? null;
        const table = buildPrayerTable(
            schedule.times.entries,
            PRAYER_TABLE_COLS,
            nextName,
            useSystemTheme,
            prayerColor,
        );
        this._prayerBox.add_child(table);
        if (schedule.next) {
            const prefix = ui.next_prayer_prefix || '';
            const nextLabel = makeEllipsisLabel({ style_class: 'boloot-prayer-next' });
            if (!useSystemTheme && prayerColor)
                nextLabel.style = `color: ${prayerColor}; font-weight: 600;`;
            const showCountdown = prayerCfg.display_mode === 'countdown' && !this._reducedMotion;
            if (showCountdown) {
                this._countdownAnchor = {
                    monotonicUs: GLib.get_monotonic_time(),
                    remaining: schedule.next.remaining_seconds,
                    prefix,
                    label: schedule.next.label,
                };
                const numerals = effectiveNumerals(settings);
                nextLabel.text =
                    `${prefix} ${schedule.next.label} ${formatCountdownLabel(schedule.next.remaining_seconds, numerals)}`;
                this._nextPrayerLabel = nextLabel;
                this._countdownTimer = GLib.timeout_add_seconds(
                    GLib.PRIORITY_DEFAULT,
                    1,
                    () => this._tickCountdown(),
                );
            } else {
                nextLabel.text = `${prefix} ${schedule.next.label} ${schedule.next.time}`;
            }
            this._prayerBox.add_child(nextLabel);
        }

        if (prayerCfg.display_mode === 'countdown' && !this._reducedMotion) {
            this._prayerRefreshTimer = GLib.timeout_add_seconds(
                GLib.PRIORITY_DEFAULT,
                REFRESH_NORMAL_SEC,
                () => {
                    if (!this.visible)
                        return GLib.SOURCE_CONTINUE;
                    this._prayerSchedule = this._prayerScheduleFromView();
                    this._renderPrayer(false);
                    return GLib.SOURCE_CONTINUE;
                },
            );
        }
    }

    _goToday() {
        this._displayYear = 0;
        this._displayMonth = 0;
        this._selectedGregorian = null;
        this.refresh();
        this._syncGnomeCalendar(new Date());
    }

    _changeYear(delta) {
        if (!this._monthView) {
            this.refresh();
            return;
        }
        if (this._displayYear <= 0)
            this._displayYear = this._monthView.display_year ?? this._monthView.jalali_year;
        if (this._displayMonth <= 0)
            this._displayMonth = this._monthView.display_month ?? this._monthView.jalali_month;
        this._displayYear += delta;
        this.refresh();
    }

    _changeMonth(delta) {
        if (!this._monthView) {
            this.refresh();
            return;
        }
        let month = this._displayMonth + delta;
        let year = this._displayYear;
        if (month < 1) {
            month = 12;
            year -= 1;
        } else if (month > 12) {
            month = 1;
            year += 1;
        }
        this._displayYear = year;
        this._displayMonth = month;
        this.refresh();
    }

    destroy() {
        this._stopPrayerTimers();
        super.destroy();
    }
});

function buildHolidaysBar() {
    const bar = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-holidays-bar',
        x_expand: true,
    });
    const divider = makeDivider();
    const label = new St.Label({
        style_class: 'boloot-holidays',
        x_expand: true,
    });
    label.clutter_text.ellipsize = Pango.EllipsizeMode.NONE;
    if ('line_wrap' in label.clutter_text)
        label.clutter_text.line_wrap = true;
    bar.add_child(divider);
    bar.add_child(label);
    return { bar, divider, label };
}

function buildPrayerBar() {
    const bar = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-prayer-bar',
        x_expand: true,
    });
    const divider = makeDivider();
    const box = new St.BoxLayout({ vertical: true, style_class: 'boloot-prayer' });
    bar.add_child(divider);
    bar.add_child(box);
    return { bar, divider, box };
}

function buildExtrasScrollArea(holidaysBar, prayerBar) {
    const extras = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-extras',
        x_expand: true,
    });
    extras.add_child(holidaysBar.bar);
    extras.add_child(prayerBar.bar);

    const scroll = new St.ScrollView({
        style_class: 'boloot-extras-scroll',
        x_expand: true,
        y_expand: false,
        overlay_scrollbars: true,
        enable_mouse_scrolling: true,
        hscrollbar_policy: St.PolicyType.NEVER,
        vscrollbar_policy: St.PolicyType.AUTOMATIC,
        clip_to_allocation: true,
    });
    if (typeof scroll.set_policy === 'function')
        scroll.set_policy(St.PolicyType.NEVER, St.PolicyType.AUTOMATIC);

    // GNOME 46+: child property; older Shell used add_actor / set_child.
    if ('child' in scroll)
        scroll.child = extras;
    else if (typeof scroll.set_child === 'function')
        scroll.set_child(extras);
    else if (typeof scroll.add_actor === 'function')
        scroll.add_actor(extras);
    else
        scroll.add_child(extras);

    return { scroll, extras };
}

function buildBolootPopupContent(dateMenu, nativeCalendarWidth = 0) {
    const layout = computePopupLayout({
        nativeCalendarWidth: nativeCalendarWidth || getNativeCalendarWidth(dateMenu),
    });
    const container = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-popup-container',
        x_expand: false,
        width: layout.popupWidth,
    });

    const calendarBody = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-calendar-body',
        x_expand: false,
        width: layout.popupWidth,
        height: layout.calendarHeight,
    });
    const calendarSection = new BolootMenuSection();
    calendarSection.setLayout(layout);
    calendarSection.setDateMenu(dateMenu);
    calendarSection.setSettingsAction(() => openBolootSettings(dateMenu));
    calendarBody.add_child(calendarSection);

    const holidaysBar = buildHolidaysBar();
    const prayerBar = buildPrayerBar();
    calendarSection.attachFooterWidgets(holidaysBar, prayerBar);

    const { scroll: extrasScroll } = buildExtrasScrollArea(holidaysBar, prayerBar);
    extrasScroll.height = layout.extrasMaxHeight;

    const footer = new St.BoxLayout({
        vertical: true,
        style_class: 'boloot-popup-footer',
        x_expand: true,
    });
    footer.height = layout.footerHeight;

    footer.add_child(new St.Label({
        text: `${APP_NAME} — ${WEBSITE_LABEL}`,
        style_class: 'boloot-brand',
        x_expand: true,
        x_align: Clutter.ActorAlign.CENTER,
    }));

    container.add_child(calendarBody);
    container.add_child(extrasScroll);
    container.add_child(footer);

    return {
        container,
        calendarSection,
        calendarBody,
        extrasScroll,
        holidaysBar: holidaysBar.bar,
        prayerBar: prayerBar.bar,
        footer,
        layout,
    };
}

/** Integrate Boloot calendar into GNOME's built-in dateMenu popup. */
const DateMenuIntegrator = GObject.registerClass(
class DateMenuIntegrator extends GObject.Object {
    _init() {
        super._init();
        this._dateMenu = null;
        this._calendarColumn = null;
        this._bolootContainer = null;
        this._calendarSection = null;
        this._calendarBody = null;
        this._extrasScroll = null;
        this._holidaysBar = null;
        this._prayerBar = null;
        this._footer = null;
        this._popupLayout = null;
        this._cachedNativeCalendarWidth = 0;
        this._openStateId = 0;
        this._clockNotifyId = 0;
        this._timerId = 0;
        this._ownerWatchId = 0;
        this._retryId = 0;
        this._interfaceSettings = null;
        this._colorSchemeId = 0;
        this._gtkThemeId = 0;
        this._textScalingId = 0;
        this._monitorChangedId = 0;
        this._monitorChangedSource = null;
        this._currentInterval = REFRESH_NORMAL_SEC;
        this._restoringGnomeClock = false;
        this._gnomeWidgetState = null;
        this._sessionModeId = 0;
        this._greeterDelayIds = [];
        this._clockAppliedOnce = false;
    }

    enable() {
        this._tryEnable(0);
    }

    _tryEnable(attempt) {
        this._dateMenu = Main.panel.statusArea?.dateMenu;
        const maxAttempts = isGreeter() ? ENABLE_RETRY_GREETER : ENABLE_RETRY_NORMAL;
        if (!this._dateMenu?._calendar || !this._dateMenu?._clockDisplay) {
            if (attempt < maxAttempts) {
                this._retryId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, ENABLE_RETRY_INTERVAL_MS, () => {
                    this._tryEnable(attempt + 1);
                    return GLib.SOURCE_REMOVE;
                });
                return;
            }
            log(`${APP_NAME}: dateMenu not found — cannot integrate calendar${isGreeter() ? ' (greeter)' : ''}`);
            return;
        }

        this._calendarColumn = this._dateMenu._calendar.get_parent();
        if (!this._calendarColumn) {
            log(`${APP_NAME}: dateMenu calendar column not found`);
            return;
        }

        this._cachedNativeCalendarWidth = getNativeCalendarWidth(this._dateMenu);

        const built = buildBolootPopupContent(this._dateMenu, this._cachedNativeCalendarWidth);
        this._bolootContainer = built.container;
        this._bolootContainer.visible = false;
        this._calendarSection = built.calendarSection;
        this._calendarBody = built.calendarBody;
        this._extrasScroll = built.extrasScroll;
        this._holidaysBar = built.holidaysBar;
        this._prayerBar = built.prayerBar;
        this._footer = built.footer;
        this._popupLayout = built.layout;

        const monitorListener = connectMonitorsChanged(() => {
            if (this._dateMenu?.menu.isOpen && this._bolootContainer?.visible)
                this._applyPopupLayout();
        });
        if (monitorListener) {
            this._monitorChangedSource = monitorListener.source;
            this._monitorChangedId = monitorListener.id;
        }

        try {
            this._calendarColumn.insert_child_above(this._bolootContainer, this._dateMenu._calendar);
        } catch (e) {
            log(`${APP_NAME}: insert_child_above failed, using add_child: ${e}`);
            this._calendarColumn.add_child(this._bolootContainer);
        }
        this._dateMenu.menu.box.add_style_class_name('boloot-datemenu-popup');

        this._openStateId = this._dateMenu.menu.connect('open-state-changed', (_menu, isOpen) => {
            if (isOpen)
                this._onMenuOpen();
            else
                this._calendarSection?._stopPrayerTimers();
        });

        const display = this._dateMenu._clockDisplay;
        const clock = this._dateMenu._clock;
        display.clutter_text.ellipsize = Pango.EllipsizeMode.END;

        if (clock) {
            this._clockNotifyId = clock.connect_after('notify::clock', () => {
                this._applyClock();
            });
        }

        this._rearmTimer();

        this._ownerWatchId = dbusBus().watch_name(
            DBUS_NAME,
            Gio.BusNameWatcherFlags.NONE,
            () => this._applyClock(),
            () => this._applyClock(),
        );

        this._watchSystemTheme();

        try {
            this._sessionModeId = Main.sessionMode.connect('updated', () => {
                const settings = fetchSettingsCached();
                const useBoloot = settings?.appearance?.show_in_popup !== false;
                if (useBoloot)
                    this._applyGnomeWidgetVisibility(true);
            });
        } catch (e) {
            log(`${APP_NAME}: sessionMode listener failed: ${e}`);
        }

        this._syncCalendarMode();
        this._applyClock();
        this._scheduleGreeterClockRetries();
        log(`${APP_NAME}: dateMenu integration active${isGreeter() ? ' (greeter)' : ''}`);
    }

    _scheduleGreeterClockRetries() {
        if (!isGreeter())
            return;
        for (const id of this._greeterDelayIds)
            GLib.source_remove(id);
        this._greeterDelayIds = [];
        for (const delaySec of GREETER_DELAYED_SEC) {
            const id = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, delaySec, () => {
                this._applyClock();
                return GLib.SOURCE_REMOVE;
            });
            this._greeterDelayIds.push(id);
        }
    }

    _measureAndCacheNativeCalendarWidth() {
        const measured = getNativeCalendarWidth(this._dateMenu);
        if (measured > 0)
            this._cachedNativeCalendarWidth = measured;
        return this._cachedNativeCalendarWidth;
    }

    _applyPopupLayout() {
        if (!this._bolootContainer || !this._calendarBody)
            return;

        const layout = computePopupLayout({
            nativeCalendarWidth: this._measureAndCacheNativeCalendarWidth(),
        });
        this._popupLayout = layout;

        this._bolootContainer.width = layout.popupWidth;
        this._calendarBody.width = layout.popupWidth;
        this._calendarBody.height = layout.calendarHeight;

        if (this._extrasScroll)
            this._extrasScroll.height = layout.extrasMaxHeight;
        if (this._footer)
            this._footer.height = layout.footerHeight;

        this._calendarSection?.setLayout(layout);
    }

    _onMenuOpen() {
        if (!this._bolootContainer?.visible)
            return;

        GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            if (!this._bolootContainer?.visible)
                return GLib.SOURCE_REMOVE;
            this._applyPopupLayout();
            this._calendarSection._selectedGregorian = null;
            this._calendarSection._displayYear = 0;
            this._calendarSection._displayMonth = 0;
            this._calendarSection.refresh();
            try {
                const cal = this._dateMenu?._calendar;
                if (cal && typeof cal.setDate === 'function') {
                    if (cal.setDate.length >= 2)
                        cal.setDate(new Date(), true);
                    else
                        cal.setDate(new Date());
                }
            } catch (e) {
                log(`${APP_NAME}: reset GNOME calendar on open failed: ${e}`);
            }
            this._updateSettingsAccessibleName();
            return GLib.SOURCE_REMOVE;
        });
    }

    _updateSettingsAccessibleName() {
        const ui = this._calendarSection?._monthView?.ui;
        if (ui?.settings_button)
            this._calendarSection?.updateSettingsAccessibleName(ui.settings_button);
    }

    _watchSystemTheme() {
        try {
            this._interfaceSettings = new Gio.Settings({
                schema_id: 'org.gnome.desktop.interface',
            });
            this._colorSchemeId = this._interfaceSettings.connect(
                'changed::color-scheme',
                () => this._onSystemThemeChanged(),
            );
            this._gtkThemeId = this._interfaceSettings.connect(
                'changed::gtk-theme',
                () => this._onSystemThemeChanged(),
            );
            this._textScalingId = this._interfaceSettings.connect(
                'changed::text-scaling-factor',
                () => this._onLayoutDriversChanged(),
            );
        } catch (e) {
            this._interfaceSettings = null;
        }
    }

    _onLayoutDriversChanged() {
        if (!this._dateMenu?.menu.isOpen || !this._bolootContainer?.visible)
            return;
        this._applyPopupLayout();
    }

    _onSystemThemeChanged() {
        if (!this._dateMenu?.menu.isOpen || !this._bolootContainer?.visible)
            return;
        this._calendarSection.refresh();
    }

    _applyGnomeWidgetVisibility(useBoloot) {
        const dm = this._dateMenu;
        if (!dm)
            return;

        const widgets = [
            ['eventsItem', dm._eventsItem],
            ['displaysSection', dm._displaysSection],
            ['dateButton', dm._date],
        ];

        for (const [key, actor] of widgets) {
            if (!actor)
                continue;
            if (useBoloot) {
                if (!this._gnomeWidgetState)
                    this._gnomeWidgetState = {};
                if (!(key in this._gnomeWidgetState))
                    this._gnomeWidgetState[key] = actor.visible;
                actor.visible = false;
            } else if (this._gnomeWidgetState && key in this._gnomeWidgetState) {
                actor.visible = this._gnomeWidgetState[key];
            }
        }

        if (!useBoloot)
            this._gnomeWidgetState = null;
    }

    _syncCalendarMode() {
        if (!this._dateMenu)
            return;

        const settings = fetchSettingsCached();
        const useBoloot = settings?.appearance?.show_in_popup !== false;

        this._dateMenu._calendar.visible = !useBoloot;
        if (this._bolootContainer)
            this._bolootContainer.visible = useBoloot;
        this._applyGnomeWidgetVisibility(useBoloot);
    }

    _rearmTimer() {
        if (this._timerId) {
            GLib.source_remove(this._timerId);
            this._timerId = 0;
        }
        const greeterFastPoll = isGreeter() && !this._clockAppliedOnce;
        this._currentInterval = greeterFastPoll ? GREETER_POLL_SEC : refreshIntervalSec();
        this._timerId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, this._currentInterval, () => {
            if (greeterFastPoll && !this._clockAppliedOnce) {
                this._applyClock();
                if (!this._clockAppliedOnce)
                    return GLib.SOURCE_CONTINUE;
                this._rearmTimer();
                return GLib.SOURCE_REMOVE;
            }
            const needed = refreshIntervalSec();
            if (needed !== this._currentInterval) {
                this._rearmTimer();
                return GLib.SOURCE_REMOVE;
            }
            this._applyClock();
            return GLib.SOURCE_CONTINUE;
        });
    }

    _applyClock() {
        if (!this._dateMenu?._clockDisplay)
            return;

        const settings = fetchSettingsCached();
        if (settings?.appearance?.show_in_top_bar === false) {
            if (!this._restoringGnomeClock) {
                this._restoringGnomeClock = true;
                this._dateMenu._clock?.notify('clock');
                this._restoringGnomeClock = false;
            }
            return;
        }

        const text = fetchTopBarText();
        if (!text) {
            if (isGreeter() && !this._clockAppliedOnce)
                log(`${APP_NAME}: greeter waiting for D-Bus service ${DBUS_NAME}`);
            return;
        }

        const display = this._dateMenu._clockDisplay;
        display.set_text(text);
        if (display.clutter_text)
            display.clutter_text.text = text;

        if (!this._clockAppliedOnce) {
            this._clockAppliedOnce = true;
            if (isGreeter())
                log(`${APP_NAME}: greeter top bar date applied`);
            if (isGreeter())
                this._rearmTimer();
        }

        const calendarView = fetchJson('GetCalendarView');
        let holidaysToday = [];
        try {
            const raw = dbusCall('GetHolidaysToday', null, new GLib.VariantType('(s)'));
            if (raw)
                holidaysToday = JSON.parse(raw.deepUnpack()[0]) || [];
        } catch (_e) {
            holidaysToday = [];
        }
        applyTopBarDayStyle(display, calendarView, settings, holidaysToday);
    }

    onSettingsChanged() {
        invalidateSettingsCache();
        this._rearmTimer();
        GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
            this._applyClock();
            return GLib.SOURCE_REMOVE;
        });
        this._syncCalendarMode();
        if (this._dateMenu?.menu.isOpen && this._bolootContainer?.visible) {
            this._applyPopupLayout();
            this._calendarSection._displayYear = 0;
            this._calendarSection._displayMonth = 0;
            this._calendarSection.refresh();
        }
    }

    disable() {
        if (this._retryId) {
            GLib.source_remove(this._retryId);
            this._retryId = 0;
        }
        for (const id of this._greeterDelayIds)
            GLib.source_remove(id);
        this._greeterDelayIds = [];
        this._clockAppliedOnce = false;
        if (this._openStateId && this._dateMenu?.menu) {
            this._dateMenu.menu.disconnect(this._openStateId);
            this._openStateId = 0;
        }
        if (this._clockNotifyId && this._dateMenu?._clock) {
            this._dateMenu._clock.disconnect(this._clockNotifyId);
            this._clockNotifyId = 0;
        }
        if (this._timerId) {
            GLib.source_remove(this._timerId);
            this._timerId = 0;
        }
        if (this._ownerWatchId) {
            dbusBus().unwatch_name(this._ownerWatchId);
            this._ownerWatchId = 0;
        }
        if (this._monitorChangedId && this._monitorChangedSource) {
            this._monitorChangedSource.disconnect(this._monitorChangedId);
            this._monitorChangedId = 0;
            this._monitorChangedSource = null;
        }
        if (this._interfaceSettings) {
            if (this._colorSchemeId) {
                this._interfaceSettings.disconnect(this._colorSchemeId);
                this._colorSchemeId = 0;
            }
            if (this._gtkThemeId) {
                this._interfaceSettings.disconnect(this._gtkThemeId);
                this._gtkThemeId = 0;
            }
            if (this._textScalingId) {
                this._interfaceSettings.disconnect(this._textScalingId);
                this._textScalingId = 0;
            }
            this._interfaceSettings = null;
        }
        if (this._sessionModeId) {
            try {
                Main.sessionMode.disconnect(this._sessionModeId);
            } catch (_e) {
                // ignore
            }
            this._sessionModeId = 0;
        }
        if (this._bolootContainer && this._calendarColumn) {
            this._calendarColumn.remove_child(this._bolootContainer);
            this._bolootContainer.destroy();
            this._bolootContainer = null;
        }
        if (this._dateMenu) {
            this._applyGnomeWidgetVisibility(false);
            this._dateMenu.menu.box.remove_style_class_name('boloot-datemenu-popup');
            if (this._dateMenu._calendar)
                this._dateMenu._calendar.visible = true;
            if (this._dateMenu._clock)
                this._dateMenu._clock.notify('clock');
        }
        this._calendarSection = null;
        this._calendarBody = null;
        this._holidaysBar = null;
        this._prayerBar = null;
        this._footer = null;
        this._popupLayout = null;
        this._calendarColumn = null;
        this._dateMenu = null;
    }
});

export default class BolootCalendarExtension extends Extension {
    enable() {
        this.initTranslations();

        try {
            this._dateMenuIntegrator = new DateMenuIntegrator();
            this._dateMenuIntegrator.enable();
        } catch (e) {
            log(`${APP_NAME}: dateMenu integration failed: ${e}`);
        }

        this._settingsChangedId = dbusBus().signal_subscribe(
            DBUS_NAME,
            DBUS_IFACE,
            'SettingsChanged',
            DBUS_PATH,
            null,
            Gio.DBusSignalFlags.NONE,
            () => {
                if (this._dateMenuIntegrator)
                    this._dateMenuIntegrator.onSettingsChanged();
            },
        );
    }

    disable() {
        if (this._dateMenuIntegrator) {
            this._dateMenuIntegrator.disable();
            this._dateMenuIntegrator = null;
        }
        if (this._settingsChangedId) {
            dbusBus().signal_unsubscribe(this._settingsChangedId);
            this._settingsChangedId = 0;
        }
    }
}
