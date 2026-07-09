pub mod adhan;
pub mod brand;
pub mod calendar;
pub mod colors;
pub mod config;
pub mod error;
pub mod format;
pub mod holidays;
pub mod locale;
pub mod month_view;
pub mod prayer;
pub mod service;
pub mod system_locale;
pub mod ui_strings;

pub use brand::{
    APP_NAME, APP_NAME_SHORT, DONATE_BTC, DONATE_USDT_TRC20, WEBSITE, WEBSITE_LABEL,
};
pub use colors::{
    appearance_tints, is_valid_hex, AppearanceTints, DEFAULT_BG_COLOR, DEFAULT_HOLIDAY_COLOR,
    DEFAULT_PRAYER_COLOR, DEFAULT_TEXT_COLOR, DEFAULT_TODAY_COLOR,
};
pub use calendar::{CalendarDate, CalendarSystem, CalendarView};
pub use config::{
    active_session_uids, home_dir_for_uid, username_for_uid, AdhanPreset, AdhanPrayerToggles,
    AppearanceSettings, BolootConfig, CalendarSettings, PrayerSettings,
};
pub use error::{BolootError, Result};
pub use format::DateFormatter;
pub use adhan::{is_adhan_enabled_for, resolve_adhan_path, should_trigger_adhan, sounds_dir};
pub use holidays::{Holiday, HolidayStore};
pub use month_view::MonthView;
pub use locale::{CountryProfile, LanguageVariant, LocaleProfile, Weekday};
pub use prayer::{PrayerDayStatus, PrayerName, PrayerTimes};
pub use service::BolootService;
pub use system_locale::{detect_from_env, DetectedLocale};
pub use ui_strings::UiStrings;
