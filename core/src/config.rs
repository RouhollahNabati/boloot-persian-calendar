use std::fs;
use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::calendar::CalendarSystem;
use crate::colors::{
    is_valid_hex, DEFAULT_BG_COLOR, DEFAULT_HOLIDAY_COLOR, DEFAULT_PRAYER_COLOR,
    DEFAULT_TEXT_COLOR, DEFAULT_TODAY_COLOR,
};
use crate::error::{BolootError, Result};
use crate::format::{effective_numerals, DateFormatStyle, NumeralStyle};
use crate::locale::{CountryProfile, LanguageVariant, Weekday};
use crate::prayer::{
    LocationStore, PrayerCalculationMethod, PrayerMadhab, PrayerName,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BolootConfig {
    #[serde(default)]
    pub calendar: CalendarSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub prayer: PrayerSettings,
}

impl Default for BolootConfig {
    fn default() -> Self {
        Self {
            calendar: CalendarSettings::default(),
            appearance: AppearanceSettings::default(),
            prayer: PrayerSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSettings {
    #[serde(default)]
    pub country: CountryProfile,
    #[serde(default)]
    pub language: LanguageVariant,
    #[serde(default)]
    pub calendar_type: CalendarSystem,
    #[serde(default)]
    pub week_start: Weekday,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default)]
    pub numerals: NumeralStyle,
    #[serde(default)]
    pub date_style: DateFormatStyle,
    #[serde(default)]
    pub show_holidays: bool,
    #[serde(default)]
    pub holiday_notifications: bool,
    #[serde(default = "default_true")]
    pub follow_system_locale: bool,
}

fn default_timezone() -> String {
    "Asia/Tehran".into()
}

impl CalendarSettings {
    pub fn effective_numerals(&self) -> NumeralStyle {
        effective_numerals(self.calendar_type, self.numerals)
    }
}

impl Default for CalendarSettings {
    fn default() -> Self {
        let locale = crate::locale::LocaleProfile::resolve(
            CountryProfile::Iran,
            LanguageVariant::Persian,
        );
        Self {
            country: CountryProfile::Iran,
            language: LanguageVariant::Persian,
            calendar_type: CalendarSystem::Jalali,
            week_start: locale.default_week_start,
            timezone: locale.default_timezone,
            numerals: NumeralStyle::Persian,
            date_style: DateFormatStyle::LongNamed,
            show_holidays: true,
            holiday_notifications: true,
            follow_system_locale: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_font")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size_pt: u8,
    #[serde(default = "default_text_color")]
    pub text_color: String,
    #[serde(default = "default_bg_color")]
    pub background_color: String,
    #[serde(default = "default_holiday_color")]
    pub holiday_color: String,
    #[serde(default = "default_today_color")]
    pub today_color: String,
    #[serde(default = "default_prayer_color")]
    pub prayer_color: String,
    #[serde(default = "default_true")]
    pub use_system_theme: bool,
    #[serde(default = "default_true")]
    pub show_in_top_bar: bool,
    #[serde(default = "default_true")]
    pub show_clock: bool,
    #[serde(default = "default_true")]
    pub show_in_popup: bool,
}

fn default_font() -> String {
    "Vazirmatn".into()
}
fn default_font_size() -> u8 {
    11
}
fn default_text_color() -> String {
    DEFAULT_TEXT_COLOR.into()
}
fn default_bg_color() -> String {
    DEFAULT_BG_COLOR.into()
}
fn default_holiday_color() -> String {
    DEFAULT_HOLIDAY_COLOR.into()
}
fn default_today_color() -> String {
    DEFAULT_TODAY_COLOR.into()
}
fn default_prayer_color() -> String {
    DEFAULT_PRAYER_COLOR.into()
}
fn default_true() -> bool {
    true
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            font_family: default_font(),
            font_size_pt: default_font_size(),
            text_color: default_text_color(),
            background_color: default_bg_color(),
            holiday_color: default_holiday_color(),
            today_color: default_today_color(),
            prayer_color: default_prayer_color(),
            use_system_theme: true,
            show_in_top_bar: true,
            show_clock: true,
            show_in_popup: true,
        }
    }
}

impl AppearanceSettings {
    /// True when appearance colors were never customized from install defaults.
    pub fn has_factory_colors(&self) -> bool {
        self.text_color == default_text_color()
            && self.background_color == default_bg_color()
            && self.holiday_color == default_holiday_color()
            && self.today_color == default_today_color()
            && self.prayer_color == default_prayer_color()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_city")]
    pub city: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(default)]
    pub method: PrayerCalculationMethod,
    #[serde(default)]
    pub madhab: PrayerMadhab,
    #[serde(default)]
    pub display_mode: PrayerDisplayMode,
    #[serde(default = "default_true")]
    pub show_in_top_bar: bool,
    #[serde(default = "default_true")]
    pub show_in_popup: bool,
    #[serde(default)]
    pub notification_minutes: Vec<u32>,
    #[serde(default)]
    pub adhan_enabled: bool,
    #[serde(default)]
    pub adhan_preset: AdhanPreset,
    pub adhan_custom_path: Option<String>,
    #[serde(default = "default_adhan_volume")]
    pub adhan_volume: u8,
    #[serde(default = "default_true")]
    pub adhan_show_notification: bool,
    #[serde(default)]
    pub adhan_prayers: AdhanPrayerToggles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdhanPreset {
    #[default]
    #[serde(alias = "default")]
    Mansouri,
    Makkah,
    Madinah,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhanPrayerToggles {
    #[serde(default = "default_true")]
    pub fajr: bool,
    #[serde(default)]
    pub sunrise: bool,
    #[serde(default = "default_true")]
    pub dhuhr: bool,
    #[serde(default = "default_true")]
    pub asr: bool,
    #[serde(default = "default_true")]
    pub maghrib: bool,
    #[serde(default = "default_true")]
    pub isha: bool,
}

impl AdhanPrayerToggles {
    pub fn is_enabled(&self, prayer: PrayerName) -> bool {
        match prayer {
            PrayerName::Fajr => self.fajr,
            PrayerName::Sunrise => self.sunrise,
            PrayerName::Dhuhr => self.dhuhr,
            PrayerName::Asr => self.asr,
            PrayerName::Maghrib => self.maghrib,
            PrayerName::Isha => self.isha,
        }
    }
}

impl Default for AdhanPrayerToggles {
    fn default() -> Self {
        Self {
            fajr: true,
            sunrise: false,
            dhuhr: true,
            asr: true,
            maghrib: true,
            isha: true,
        }
    }
}

fn default_adhan_volume() -> u8 {
    80
}

fn is_valid_audio_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "ogg" | "wav" | "mp3" | "flac"))
}

fn default_city() -> String {
    "tehran".into()
}

impl Default for PrayerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            city: default_city(),
            latitude: None,
            longitude: None,
            method: PrayerCalculationMethod::Tehran,
            madhab: PrayerMadhab::Jafari,
            display_mode: PrayerDisplayMode::NextPrayer,
            show_in_top_bar: true,
            show_in_popup: true,
            notification_minutes: vec![10],
            adhan_enabled: false,
            adhan_preset: AdhanPreset::Mansouri,
            adhan_custom_path: None,
            adhan_volume: default_adhan_volume(),
            adhan_show_notification: true,
            adhan_prayers: AdhanPrayerToggles::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrayerDisplayMode {
    #[default]
    NextPrayer,
    AllTimes,
    Countdown,
    Hidden,
}

impl BolootConfig {
    pub const SYSTEM_CONFIG_DIR: &'static str = "/etc/boloot-calendar";
    pub const SYSTEM_CONFIG_FILE: &'static str = "/etc/boloot-calendar/config.toml";

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("boloot-calendar")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn system_config_path() -> PathBuf {
        if let Ok(path) = std::env::var("BOLOOT_SYSTEM_CONFIG") {
            if !path.is_empty() {
                return PathBuf::from(path);
            }
        }
        PathBuf::from(Self::SYSTEM_CONFIG_FILE)
    }

    pub fn config_path_for_uid(uid: u32) -> Option<PathBuf> {
        home_dir_for_uid(uid).map(|home| home.join(".config/boloot-calendar/config.toml"))
    }

    pub fn config_dir_for_uid(uid: u32) -> Option<PathBuf> {
        home_dir_for_uid(uid).map(|home| home.join(".config/boloot-calendar"))
    }

    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::config_path())
    }

    pub fn load_for_uid(uid: u32) -> Result<Self> {
        Self::load_from_path(&Self::resolve_config_path(uid))
    }

    fn resolve_config_path(uid: u32) -> PathBuf {
        if let Some(user_path) = Self::config_path_for_uid(uid) {
            if user_path.exists() {
                return user_path;
            }
        }
        let system = Self::system_config_path();
        if system.exists() {
            return system;
        }
        if let Some(user_path) = Self::config_path_for_uid(uid) {
            return user_path;
        }
        system
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            let mut config = Self::default();
            config.apply_system_locale();
            return Ok(config);
        }
        let raw = fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&raw)
            .map_err(|e| BolootError::InvalidConfig(format!("{path:?}: {e}")))?;
        if config.calendar.follow_system_locale {
            config.apply_system_locale();
        }
        if config.migrate_appearance() {
            config.save_to_path(path)?;
        }
        Ok(config)
    }

    /// Returns `true` when config was updated and should be persisted.
    fn migrate_appearance(&mut self) -> bool {
        let mut changed = false;
        if !self.appearance.use_system_theme && self.appearance.has_factory_colors() {
            self.appearance.use_system_theme = true;
            changed = true;
        }
        changed
    }

    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::config_path())
    }

    pub fn save_for_uid(&self, uid: u32) -> Result<()> {
        let path = Self::config_path_for_uid(uid).ok_or_else(|| {
            BolootError::InvalidConfig(format!("no home directory for uid {uid}"))
        })?;
        self.save_to_path(&path)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let raw = toml::to_string_pretty(self)
            .map_err(|e| BolootError::InvalidConfig(e.to_string()))?;
        fs::write(path, raw)?;
        Ok(())
    }

    /// Apply country-specific defaults only when country or language changes.
    pub fn apply_country_defaults_on_change(
        &mut self,
        old_country: CountryProfile,
        old_language: LanguageVariant,
    ) {
        let locale = crate::locale::LocaleProfile::resolve(
            self.calendar.country,
            self.calendar.language,
        );
        if self.calendar.country != old_country || self.calendar.language != old_language {
            self.calendar.week_start = locale.default_week_start;
            self.calendar.timezone = locale.default_timezone.clone();
            self.prayer.method = PrayerCalculationMethod::for_country(self.calendar.country);
            self.prayer.madhab = PrayerMadhab::for_country(self.calendar.country);
        } else if self.calendar.timezone.is_empty() {
            self.calendar.timezone = locale.default_timezone.clone();
        }
    }

    pub fn export_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn import_json(raw: &str) -> Result<Self> {
        Ok(serde_json::from_str(raw)?)
    }

    pub fn apply_system_locale(&mut self) {
        let detected = crate::system_locale::detect_from_env();
        let old_country = self.calendar.country;
        let old_language = self.calendar.language;
        self.calendar.country = detected.country;
        self.calendar.language = detected.language;
        self.calendar.numerals = detected.numerals;
        self.apply_country_defaults_on_change(old_country, old_language);
    }

    pub fn validate(&self) -> Result<()> {
        if self.calendar.timezone.parse::<Tz>().is_err() {
            return Err(BolootError::InvalidConfig(format!(
                "invalid timezone: {}",
                self.calendar.timezone
            )));
        }

        if !(8..=24).contains(&self.appearance.font_size_pt) {
            return Err(BolootError::InvalidConfig(format!(
                "font_size_pt must be 8–24, got {}",
                self.appearance.font_size_pt
            )));
        }

        for (name, color) in [
            ("text_color", &self.appearance.text_color),
            ("background_color", &self.appearance.background_color),
            ("holiday_color", &self.appearance.holiday_color),
            ("today_color", &self.appearance.today_color),
            ("prayer_color", &self.appearance.prayer_color),
        ] {
            if !is_valid_hex(color) {
                return Err(BolootError::InvalidConfig(format!(
                    "invalid {name}: {color}"
                )));
            }
        }

        if self.appearance.font_family.is_empty()
            || self.appearance.font_family.contains(['"', ';', '<', '>'])
        {
            return Err(BolootError::InvalidConfig(
                "invalid font_family".into(),
            ));
        }

        if self.prayer.enabled {
            let has_coords = self.prayer.latitude.is_some() && self.prayer.longitude.is_some();
            if !has_coords && LocationStore::with_fallback().get(&self.prayer.city).is_none() {
                return Err(BolootError::InvalidConfig(format!(
                    "unknown prayer city: {}",
                    self.prayer.city
                )));
            }
            for minutes in &self.prayer.notification_minutes {
                if *minutes == 0 || *minutes > 24 * 60 {
                    return Err(BolootError::InvalidConfig(format!(
                        "invalid notification_minutes: {minutes}"
                    )));
                }
            }
        }

        if self.prayer.adhan_volume > 100 {
            return Err(BolootError::InvalidConfig(format!(
                "adhan_volume must be 0–100, got {}",
                self.prayer.adhan_volume
            )));
        }

        if self.prayer.adhan_enabled && self.prayer.adhan_preset == AdhanPreset::Custom {
            let path = self
                .prayer
                .adhan_custom_path
                .as_deref()
                .filter(|p| !p.is_empty())
                .ok_or_else(|| {
                    BolootError::InvalidConfig("adhan_custom_path required for custom preset".into())
                })?;
            let path = Path::new(path);
            if !path.is_file() {
                return Err(BolootError::InvalidConfig(format!(
                    "adhan file not found: {path:?}"
                )));
            }
            if !is_valid_audio_extension(path) {
                return Err(BolootError::InvalidConfig(
                    "adhan_custom_path must be .ogg, .wav, .mp3, or .flac".into(),
                ));
            }
        }

        Ok(())
    }
}

/// Home directory for a UNIX user id (`/etc/passwd` via `getpwuid_r`).
pub fn home_dir_for_uid(uid: u32) -> Option<PathBuf> {
    username_for_uid(uid).and_then(|name| {
        if name == "gdm" {
            return None;
        }
        passwd_entry(uid).map(|(_, home, _)| home)
    })
}

/// Login name for a UNIX user id.
pub fn username_for_uid(uid: u32) -> Option<String> {
    passwd_entry(uid).map(|(name, _, _)| name)
}

fn passwd_entry(uid: u32) -> Option<(String, PathBuf, String)> {
    let mut buffer = vec![0u8; 16_384];
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    let rc = unsafe {
        libc::getpwuid_r(
            uid as libc::uid_t,
            &mut entry,
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() {
        return None;
    }
    let name = unsafe { std::ffi::CStr::from_ptr(entry.pw_name) }
        .to_str()
        .ok()?
        .to_string();
    let home = unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) }
        .to_str()
        .ok()
        .map(PathBuf::from)?;
    let shell = unsafe { std::ffi::CStr::from_ptr(entry.pw_shell) }
        .to_str()
        .ok()?
        .to_string();
    Some((name, home, shell))
}

/// UIDs with an active systemd user session (`/run/user/<uid>`).
pub fn active_session_uids() -> Vec<u32> {
    let mut uids = Vec::new();
    let Ok(entries) = fs::read_dir("/run/user") else {
        return uids;
    };
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if let Ok(uid) = name.parse::<u32>() {
            if uid > 0 {
                uids.push(uid);
            }
        }
    }
    uids.sort_unstable();
    uids.dedup();
    uids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AdhanPreset;
    use crate::locale::{CountryProfile, LanguageVariant, Weekday};

    #[test]
    fn preserves_week_start_when_country_unchanged() {
        let mut config = BolootConfig::default();
        config.calendar.week_start = Weekday::Monday;
        config.apply_country_defaults_on_change(
            CountryProfile::Iran,
            LanguageVariant::Persian,
        );
        assert_eq!(config.calendar.week_start, Weekday::Monday);
    }

    #[test]
    fn resets_week_start_when_country_changes() {
        let mut config = BolootConfig::default();
        config.calendar.country = CountryProfile::Tajikistan;
        config.calendar.week_start = Weekday::Saturday;
        config.apply_country_defaults_on_change(
            CountryProfile::Iran,
            LanguageVariant::Persian,
        );
        assert_eq!(config.calendar.week_start, Weekday::Monday);
    }

    #[test]
    fn imports_snake_case_week_start_from_settings_json() {
        let raw = r#"{"calendar":{"calendar_type":"dual_jalali_gregorian","week_start":"saturday"}}"#;
        let config = BolootConfig::import_json(raw).unwrap();
        assert_eq!(config.calendar.week_start, Weekday::Saturday);
        assert_eq!(
            config.calendar.calendar_type,
            crate::calendar::CalendarSystem::DualJalaliGregorian
        );
    }

    #[test]
    fn accepts_legacy_pascal_case_week_start() {
        let raw = r#"{"calendar":{"week_start":"Saturday"}}"#;
        let config = BolootConfig::import_json(raw).unwrap();
        assert_eq!(config.calendar.week_start, Weekday::Saturday);
    }

    #[test]
    fn migrates_factory_colors_to_system_theme() {
        let mut config = BolootConfig::default();
        config.appearance.use_system_theme = false;
        assert!(config.migrate_appearance());
        assert!(config.appearance.use_system_theme);
    }

    #[test]
    fn keeps_custom_colors_when_system_theme_disabled() {
        let mut config = BolootConfig::default();
        config.appearance.use_system_theme = false;
        config.appearance.today_color = "#ff0000".into();
        assert!(!config.migrate_appearance());
        assert!(!config.appearance.use_system_theme);
    }

    #[test]
    fn use_system_theme_defaults_to_true_when_missing_from_toml() {
        let raw = r#"
[appearance]
font_family = "Vazirmatn"
"#;
        let config: BolootConfig = toml::from_str(raw).unwrap();
        assert!(config.appearance.use_system_theme);
    }

    #[test]
    fn show_clock_defaults_to_true_when_missing_from_toml() {
        let raw = r#"
[appearance]
font_family = "Vazirmatn"
"#;
        let config: BolootConfig = toml::from_str(raw).unwrap();
        assert!(config.appearance.show_clock);
    }

    #[test]
    fn validate_rejects_invalid_timezone() {
        let mut config = BolootConfig::default();
        config.calendar.timezone = "Not/A/Timezone".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_color() {
        let mut config = BolootConfig::default();
        config.appearance.today_color = "red".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_accepts_default_config() {
        assert!(BolootConfig::default().validate().is_ok());
    }

    #[test]
    fn validate_rejects_adhan_volume_over_100() {
        let mut config = BolootConfig::default();
        config.prayer.adhan_volume = 101;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_custom_adhan_without_path() {
        let mut config = BolootConfig::default();
        config.prayer.adhan_enabled = true;
        config.prayer.adhan_preset = AdhanPreset::Custom;
        assert!(config.validate().is_err());
    }

    #[test]
    fn system_config_example_parses() {
        let raw = include_str!("../../data/system-config/config.toml");
        let config: BolootConfig = toml::from_str(raw).unwrap();
        config.validate().unwrap();
        assert_eq!(config.calendar.country, CountryProfile::Iran);
        assert!(!config.calendar.follow_system_locale);
    }

    #[test]
    fn resolve_adhan_path_for_mansouri_preset() {
        let config = BolootConfig::default();
        let path = crate::adhan::resolve_adhan_path(&config).unwrap();
        assert!(path.ends_with("mansouri.ogg"));
        assert!(path.is_file());
    }
}
