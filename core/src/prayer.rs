use std::collections::HashMap;
use std::fs;

use chrono::{NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use salah::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::PrayerDisplayMode;
use crate::error::{BolootError, Result};
use crate::format::{format_time, NumeralStyle};
use crate::holidays::locations_dir;
use crate::locale::{CountryProfile, LanguageVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrayerCalculationMethod {
    #[default]
    Tehran,
    Mwl,
    Karachi,
    Isna,
    Egypt,
}

impl PrayerCalculationMethod {
    pub fn for_country(country: CountryProfile) -> Self {
        match country {
            CountryProfile::Iran => Self::Tehran,
            CountryProfile::Afghanistan => Self::Karachi,
            CountryProfile::Tajikistan => Self::Mwl,
        }
    }

    fn to_salah(self) -> Method {
        match self {
            Self::Tehran => Method::Tehran,
            Self::Mwl => Method::MuslimWorldLeague,
            Self::Karachi => Method::Karachi,
            Self::Isna => Method::NorthAmerica,
            Self::Egypt => Method::Egyptian,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrayerMadhab {
    #[default]
    Jafari,
    Shafi,
    Hanafi,
}

impl PrayerMadhab {
    pub fn for_country(country: CountryProfile) -> Self {
        match country {
            CountryProfile::Afghanistan => Self::Hanafi,
            _ => Self::Jafari,
        }
    }

    fn to_salah(self) -> Madhab {
        match self {
            Self::Jafari | Self::Shafi => Madhab::Shafi,
            Self::Hanafi => Madhab::Hanafi,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrayerName {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

impl PrayerName {
    pub fn label(self, language: LanguageVariant) -> &'static str {
        match language {
            LanguageVariant::Tajik => self.label_tg(),
            LanguageVariant::Pashto => self.label_ps(),
            LanguageVariant::Dari | LanguageVariant::Persian => self.label_fa(),
        }
    }

    pub fn label_fa(self) -> &'static str {
        match self {
            Self::Fajr => "فجر",
            Self::Sunrise => "طلوع",
            Self::Dhuhr => "ظهر",
            Self::Asr => "عصر",
            Self::Maghrib => "مغرب",
            Self::Isha => "عشا",
        }
    }

    fn label_ps(self) -> &'static str {
        match self {
            Self::Fajr => "فجر",
            Self::Sunrise => "لمر ختلو",
            Self::Dhuhr => "غرمه",
            Self::Asr => "مازدیګر",
            Self::Maghrib => "ماښام",
            Self::Isha => "ماسختنه",
        }
    }

    fn label_tg(self) -> &'static str {
        match self {
            Self::Fajr => "Бомдод",
            Self::Sunrise => "Офтоб",
            Self::Dhuhr => "Пешин",
            Self::Asr => "Аср",
            Self::Maghrib => "Шом",
            Self::Isha => "Хуфтан",
        }
    }

    fn to_salah(self) -> Prayer {
        match self {
            Self::Fajr => Prayer::Fajr,
            Self::Sunrise => Prayer::Sunrise,
            Self::Dhuhr => Prayer::Dhuhr,
            Self::Asr => Prayer::Asr,
            Self::Maghrib => Prayer::Maghrib,
            Self::Isha => Prayer::Isha,
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerTimeEntry {
    pub name: PrayerName,
    pub label: String,
    pub time: String,
    pub hour: u32,
    pub minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerTimes {
    pub date: NaiveDate,
    pub timezone: String,
    pub city: String,
    pub entries: Vec<PrayerTimeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextPrayer {
    pub name: PrayerName,
    pub label: String,
    pub time: String,
    pub remaining_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerDayStatus {
    pub times: PrayerTimes,
    pub next: Option<NextPrayer>,
    pub current: Option<PrayerName>,
}

impl PrayerTimes {
    /// Seconds from now until the prayer entry (positive = future, zero = now).
    pub fn seconds_until_entry(&self, entry: &PrayerTimeEntry) -> Option<i64> {
        let tz: Tz = self.timezone.parse().ok()?;
        let time = NaiveTime::from_hms_opt(entry.hour, entry.minute, 0)?;
        let utc = tz
            .from_local_datetime(&self.date.and_time(time))
            .earliest()?
            .with_timezone(&Utc);
        Some(utc.signed_duration_since(Utc::now()).num_seconds())
    }

    /// Minimum seconds until any enabled prayer entry (for adaptive polling).
    pub fn min_seconds_until_enabled(
        &self,
        is_enabled: impl Fn(PrayerName) -> bool,
    ) -> Option<i64> {
        self.entries
            .iter()
            .filter(|e| is_enabled(e.name))
            .filter_map(|e| self.seconds_until_entry(e))
            .filter(|&s| s >= 0)
            .min()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CityLocation {
    pub id: String,
    pub name: String,
    pub name_fa: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub country: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocationFile {
    cities: Vec<CityLocation>,
}

pub struct LocationStore {
    cities: HashMap<String, CityLocation>,
}

fn tehran_fallback_city() -> CityLocation {
    CityLocation {
        id: "tehran".into(),
        name: "Tehran".into(),
        name_fa: "تهران".into(),
        latitude: 35.6892,
        longitude: 51.3890,
        timezone: "Asia/Tehran".into(),
        country: "iran".into(),
    }
}

fn tehran_fallback_map() -> HashMap<String, CityLocation> {
    let mut cities = HashMap::new();
    cities.insert("tehran".into(), tehran_fallback_city());
    cities
}

impl LocationStore {
    pub fn load() -> Result<Self> {
        let mut cities = HashMap::new();
        let dir = locations_dir();
        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let raw = fs::read_to_string(&path)?;
                let file: LocationFile = serde_json::from_str(&raw)?;
                for city in file.cities {
                    cities.insert(city.id.clone(), city);
                }
            }
        }
        if cities.is_empty() {
            cities.insert("tehran".into(), tehran_fallback_city());
        }
        Ok(Self { cities })
    }

    pub fn with_fallback() -> Self {
        Self::load().unwrap_or_else(|_| Self {
            cities: tehran_fallback_map(),
        })
    }

    pub fn get(&self, id: &str) -> Option<&CityLocation> {
        self.cities.get(id)
    }

    pub fn all(&self) -> Vec<&CityLocation> {
        let mut items: Vec<_> = self.cities.values().collect();
        items.sort_by(|a, b| a.name_fa.cmp(&b.name_fa));
        items
    }
}

pub struct PrayerEngine {
    locations: LocationStore,
}

impl Default for PrayerEngine {
    fn default() -> Self {
        Self {
            locations: LocationStore::with_fallback(),
        }
    }
}

impl PrayerEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn locations(&self) -> &LocationStore {
        &self.locations
    }

    pub fn calculate(
        &self,
        date: NaiveDate,
        city_id: &str,
        latitude: Option<f64>,
        longitude: Option<f64>,
        timezone: &str,
        method: PrayerCalculationMethod,
        madhab: PrayerMadhab,
        numerals: NumeralStyle,
        language: LanguageVariant,
    ) -> Result<PrayerTimes> {
        let city = self
            .locations
            .get(city_id)
            .ok_or_else(|| BolootError::LocationNotFound(city_id.to_string()))?;

        let lat = latitude.unwrap_or(city.latitude);
        let lng = longitude.unwrap_or(city.longitude);
        let tz_name = if timezone.is_empty() {
            city.timezone.as_str()
        } else {
            timezone
        };

        let coords = Coordinates::new(lat, lng);
        let params = Configuration::with(method.to_salah(), madhab.to_salah());
        let salah_times = salah::PrayerSchedule::new()
            .on(date)
            .for_location(coords)
            .with_configuration(params)
            .calculate()
            .map_err(|e| BolootError::Prayer(e))?;

        let tz: Tz = tz_name
            .parse()
            .map_err(|_| BolootError::Prayer(format!("invalid timezone: {tz_name}")))?;

        let entries = [
            PrayerName::Fajr,
            PrayerName::Sunrise,
            PrayerName::Dhuhr,
            PrayerName::Asr,
            PrayerName::Maghrib,
            PrayerName::Isha,
        ]
        .into_iter()
        .map(|name| {
            let utc = salah_times.time(name.to_salah());
            let local = utc.with_timezone(&tz);
            let hour = local.hour();
            let minute = local.minute();
            PrayerTimeEntry {
                label: name.label(language).to_string(),
                name,
                time: format_time(hour, minute, numerals),
                hour,
                minute,
            }
        })
        .collect();

        Ok(PrayerTimes {
            date,
            timezone: tz_name.to_string(),
            city: city.name_fa.clone(),
            entries,
        })
    }

    pub fn schedule(
        &self,
        date: NaiveDate,
        city_id: &str,
        latitude: Option<f64>,
        longitude: Option<f64>,
        timezone: &str,
        method: PrayerCalculationMethod,
        madhab: PrayerMadhab,
        numerals: NumeralStyle,
        language: LanguageVariant,
    ) -> Result<PrayerDayStatus> {
        let times = self.calculate(
            date,
            city_id,
            latitude,
            longitude,
            timezone,
            method,
            madhab,
            numerals,
            language,
        )?;

        let city = self
            .locations
            .get(city_id)
            .ok_or_else(|| BolootError::LocationNotFound(city_id.to_string()))?;
        let lat = latitude.unwrap_or(city.latitude);
        let lng = longitude.unwrap_or(city.longitude);
        let coords = Coordinates::new(lat, lng);
        let params = Configuration::with(method.to_salah(), madhab.to_salah());
        let salah_times = salah::PrayerSchedule::new()
            .on(date)
            .for_location(coords)
            .with_configuration(params)
            .calculate()
            .map_err(|e| BolootError::Prayer(e))?;

        let tz: Tz = times
            .timezone
            .parse()
            .map_err(|_| BolootError::Prayer(format!("invalid timezone: {}", times.timezone)))?;

        let (current, next) = Self::resolve_current_and_next(
            date,
            &times.entries,
            tz,
            &salah_times,
            numerals,
            language,
        );

        Ok(PrayerDayStatus {
            times,
            next,
            current,
        })
    }

    fn entry_as_utc(
        date: NaiveDate,
        entry: &PrayerTimeEntry,
        tz: Tz,
    ) -> Option<chrono::DateTime<Utc>> {
        let time = NaiveTime::from_hms_opt(entry.hour, entry.minute, 0)?;
        tz.from_local_datetime(&date.and_time(time))
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn resolve_current_and_next(
        date: NaiveDate,
        entries: &[PrayerTimeEntry],
        tz: Tz,
        salah_times: &salah::PrayerTimes,
        numerals: NumeralStyle,
        language: LanguageVariant,
    ) -> (Option<PrayerName>, Option<NextPrayer>) {
        let now = Utc::now();
        let track = [
            PrayerName::Fajr,
            PrayerName::Dhuhr,
            PrayerName::Asr,
            PrayerName::Maghrib,
            PrayerName::Isha,
        ];

        let mut current: Option<PrayerName> = None;

        for name in track {
            let Some(entry) = entries.iter().find(|e| e.name == name) else {
                continue;
            };
            let Some(utc) = Self::entry_as_utc(date, entry, tz) else {
                continue;
            };
            if utc > now {
                let remaining = utc.signed_duration_since(now).num_seconds();
                return (
                    current,
                    Some(NextPrayer {
                        name,
                        label: entry.label.clone(),
                        time: entry.time.clone(),
                        remaining_seconds: remaining,
                    }),
                );
            }
            current = Some(name);
        }

        let fajr_utc = salah_times.time(Prayer::FajrTomorrow);
        let remaining = fajr_utc.signed_duration_since(now).num_seconds();
        if remaining > 0 {
            let local = fajr_utc.with_timezone(&tz);
            return (
                current.or(Some(PrayerName::Isha)),
                Some(NextPrayer {
                    name: PrayerName::Fajr,
                    label: PrayerName::Fajr.label(language).to_string(),
                    time: format_time(local.hour(), local.minute(), numerals),
                    remaining_seconds: remaining,
                }),
            );
        }

        (current, None)
    }

    pub fn format_top_bar(
        &self,
        schedule: &PrayerDayStatus,
        mode: PrayerDisplayMode,
        numerals: NumeralStyle,
    ) -> Option<String> {
        match mode {
            PrayerDisplayMode::Hidden => None,
            PrayerDisplayMode::AllTimes => Some(
                schedule
                    .times
                    .entries
                    .iter()
                    .map(|e| format!("{} {}", e.label, e.time))
                    .collect::<Vec<_>>()
                    .join("  "),
            ),
            PrayerDisplayMode::Countdown => schedule.next.as_ref().map(|next| {
                let hours = next.remaining_seconds / 3600;
                let minutes = (next.remaining_seconds % 3600) / 60;
                let countdown = match numerals {
                    NumeralStyle::Persian => crate::format::to_persian_digits(&format!(
                        "{hours}س {minutes}د"
                    )),
                    NumeralStyle::Latin => format!("{hours}h {minutes}m"),
                };
                format!("{} {}", next.label, countdown)
            }),
            PrayerDisplayMode::NextPrayer => schedule.next.as_ref().map(|next| {
                format!("{} {}", next.label, next.time)
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn tehran_prayer_times() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 12).unwrap();
        let times = engine
            .calculate(
                date,
                "tehran",
                None,
                None,
                "Asia/Tehran",
                PrayerCalculationMethod::Tehran,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .unwrap();
        assert_eq!(times.entries.len(), 6);
        assert!(times.entries[0].hour < 6);
    }

    #[test]
    fn schedule_does_not_panic_when_salah_current_would_fail() {
        let engine = PrayerEngine::new();
        // All prayers on this date are in the past; salah::PrayerTimes::current() panics here.
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let _status = engine
            .schedule(
                date,
                "tehran",
                None,
                None,
                "Asia/Tehran",
                PrayerCalculationMethod::Tehran,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .unwrap();
    }

    #[test]
    fn schedule_next_is_fajr_before_sunrise() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2099, 6, 12).unwrap();
        let status = engine
            .schedule(
                date,
                "tehran",
                None,
                None,
                "Asia/Tehran",
                PrayerCalculationMethod::Tehran,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .unwrap();
        assert_eq!(status.next.as_ref().map(|n| n.name), Some(PrayerName::Fajr));
    }
}
