use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once, OnceLock};

use chrono::{NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use salah::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::PrayerDisplayMode;
use crate::countries::CountryProfile;
use crate::error::{BolootError, Result};
use crate::format::{format_time, NumeralStyle};
use crate::holidays::locations_dir;
use crate::locale::LanguageVariant;
use crate::system_time::local_now;

thread_local! {
    static SUPPRESS_SALAH_PANIC: Cell<bool> = const { Cell::new(false) };
}

static SUPPRESS_SALAH_PANIC_GLOBAL: AtomicBool = AtomicBool::new(false);
static SALAH_PANIC_HOOK: Once = Once::new();
static COORD_CACHE: OnceLock<Mutex<HashMap<(String, NaiveDate), (f64, f64)>>> = OnceLock::new();
static FAILED_COORD_CACHE: OnceLock<Mutex<HashSet<(String, NaiveDate, i64, i64)>>> = OnceLock::new();

fn coord_cache() -> &'static Mutex<HashMap<(String, NaiveDate), (f64, f64)>> {
    COORD_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn failed_coord_cache() -> &'static Mutex<HashSet<(String, NaiveDate, i64, i64)>> {
    FAILED_COORD_CACHE.get_or_init(|| Mutex::new(HashSet::new()))
}

fn coord_failure_key(city_id: &str, date: NaiveDate, lat: f64, lng: f64) -> (String, NaiveDate, i64, i64) {
    (
        city_id.to_string(),
        date,
        (lat * 1_000_000.0) as i64,
        (lng * 1_000_000.0) as i64,
    )
}

fn salah_panic_suppressed() -> bool {
    SUPPRESS_SALAH_PANIC.with(|flag| flag.get()) || SUPPRESS_SALAH_PANIC_GLOBAL.load(Ordering::Relaxed)
}

/// Install a panic hook that silences expected `salah` edge-case panics.
pub fn install_salah_panic_hook() {
    ensure_salah_panic_hook();
}

fn ensure_salah_panic_hook() {
    SALAH_PANIC_HOOK.call_once(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if salah_panic_suppressed() {
                return;
            }
            default_hook(info);
        }));
    });
}

fn with_suppressed_salah_panic<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> std::thread::Result<R> {
    ensure_salah_panic_hook();
    SUPPRESS_SALAH_PANIC.with(|flag| flag.set(true));
    SUPPRESS_SALAH_PANIC_GLOBAL.store(true, Ordering::Relaxed);
    let result = catch_unwind(f);
    SUPPRESS_SALAH_PANIC.with(|flag| flag.set(false));
    SUPPRESS_SALAH_PANIC_GLOBAL.store(false, Ordering::Relaxed);
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrayerCalculationMethod {
    #[default]
    Tehran,
    Mwl,
    Karachi,
    Isna,
    Egypt,
    MoonsightingCommittee,
    UmmAlQura,
    Turkey,
    Singapore,
    Dubai,
}

impl PrayerCalculationMethod {
    pub fn for_country(country: &CountryProfile) -> Self {
        Self::from_str(country.prayer_method_id()).unwrap_or(Self::Mwl)
    }

    pub fn suggested_for_city(city: &CityLocation) -> Self {
        if city.latitude.abs() > 55.0 {
            if matches!(city.country.as_str(), "usa" | "canada") {
                return Self::MoonsightingCommittee;
            }
            if city.region.as_deref() == Some("europe") {
                return Self::MoonsightingCommittee;
            }
        }

        Self::from_str(
            CountryProfile::new(&city.country)
                .prayer_method_id(),
        )
        .unwrap_or_else(|_| match city.country.as_str() {
            "iran" => Self::Tehran,
            "afghanistan" | "pakistan" => Self::Karachi,
            "tajikistan" => Self::Mwl,
            "usa" | "canada" => Self::Isna,
            "saudi_arabia" => Self::UmmAlQura,
            "uae" => Self::Dubai,
            "qatar" | "kuwait" | "bahrain" => Self::UmmAlQura,
            "turkey" => Self::Turkey,
            "malaysia" | "indonesia" => Self::Singapore,
            _ if city.latitude.abs() > 55.0 => Self::MoonsightingCommittee,
            _ => Self::Mwl,
        })
    }

    fn to_salah(self) -> Method {
        match self {
            Self::Tehran => Method::Tehran,
            Self::Mwl => Method::MuslimWorldLeague,
            Self::Karachi => Method::Karachi,
            Self::Isna => Method::NorthAmerica,
            Self::Egypt => Method::Egyptian,
            Self::MoonsightingCommittee => Method::MoonsightingCommittee,
            Self::UmmAlQura => Method::UmmAlQura,
            Self::Turkey => Method::Turkey,
            Self::Singapore => Method::Singapore,
            Self::Dubai => Method::Dubai,
        }
    }
}

impl FromStr for PrayerCalculationMethod {
    type Err = BolootError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "tehran" => Ok(Self::Tehran),
            "mwl" => Ok(Self::Mwl),
            "karachi" => Ok(Self::Karachi),
            "isna" => Ok(Self::Isna),
            "egypt" => Ok(Self::Egypt),
            "moonsighting_committee" => Ok(Self::MoonsightingCommittee),
            "umm_al_qura" => Ok(Self::UmmAlQura),
            "turkey" => Ok(Self::Turkey),
            "singapore" => Ok(Self::Singapore),
            "dubai" => Ok(Self::Dubai),
            _ => Err(BolootError::InvalidConfig(format!(
                "unknown prayer method: {s}"
            ))),
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
    pub fn for_country(country: &CountryProfile) -> Self {
        Self::from_str(country.prayer_madhab_id()).unwrap_or(Self::Jafari)
    }

    fn to_salah(self) -> Madhab {
        match self {
            Self::Jafari | Self::Shafi => Madhab::Shafi,
            Self::Hanafi => Madhab::Hanafi,
        }
    }
}

impl FromStr for PrayerMadhab {
    type Err = BolootError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "jafari" => Ok(Self::Jafari),
            "shafi" => Ok(Self::Shafi),
            "hanafi" => Ok(Self::Hanafi),
            _ => Err(BolootError::InvalidConfig(format!(
                "unknown prayer madhab: {s}"
            ))),
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
            LanguageVariant::English => self.label_en(),
            LanguageVariant::Tajik => self.label_tg(),
            LanguageVariant::Pashto => self.label_ps(),
            LanguageVariant::Dari | LanguageVariant::Persian => self.label_fa(),
        }
    }

    fn label_en(self) -> &'static str {
        match self {
            Self::Fajr => "Fajr",
            Self::Sunrise => "Sunrise",
            Self::Dhuhr => "Dhuhr",
            Self::Asr => "Asr",
            Self::Maghrib => "Maghrib",
            Self::Isha => "Isha",
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
        Some(
            utc.signed_duration_since(local_now().with_timezone(&Utc))
                .num_seconds(),
        )
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
    #[serde(default)]
    pub region: Option<String>,
}

impl CityLocation {
    pub fn matches_calendar_country(&self, country: &CountryProfile) -> bool {
        self.country == country.as_str()
    }
}

struct ResolvedPrayerLocation {
    lat: f64,
    lng: f64,
    timezone: String,
    display_name: String,
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
        region: None,
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

fn calculate_salah_schedule(
    date: NaiveDate,
    coords: Coordinates,
    params: Parameters,
    city_id: &str,
) -> Result<salah::PrayerTimes> {
    let result = with_suppressed_salah_panic(AssertUnwindSafe(|| {
        PrayerSchedule::new()
            .on(date)
            .for_location(coords)
            .with_configuration(params)
            .calculate()
    }));

    match result {
        Ok(Ok(schedule)) => Ok(schedule),
        Ok(Err(e)) => Err(BolootError::Prayer(e.to_string())),
        Err(_) => Err(BolootError::Prayer(format!(
            "prayer calculation failed for {city_id} on {date} (astronomical edge case)"
        ))),
    }
}

struct SalahComputation {
    location: ResolvedPrayerLocation,
    salah: salah::PrayerTimes,
}

impl PrayerEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn locations(&self) -> &LocationStore {
        &self.locations
    }

    fn fallback_coords_for(&self, city_id: &str) -> Vec<(f64, f64)> {
        let Some(city) = self.locations.get(city_id) else {
            return Vec::new();
        };
        self.locations
            .all()
            .into_iter()
            .filter(|candidate| {
                candidate.id != city_id && candidate.timezone == city.timezone
            })
            .map(|candidate| (candidate.latitude, candidate.longitude))
            .collect()
    }

    fn compute_salah(
        &self,
        date: NaiveDate,
        city_id: &str,
        latitude: Option<f64>,
        longitude: Option<f64>,
        timezone: &str,
        method: PrayerCalculationMethod,
        madhab: PrayerMadhab,
    ) -> Result<SalahComputation> {
        let location = self.resolve_location(city_id, latitude, longitude, timezone)?;
        let cache_key = (city_id.to_string(), date);
        let params = Configuration::with(method.to_salah(), madhab.to_salah());

        if let Ok(cache) = coord_cache().lock() {
            if let Some(&(lat, lng)) = cache.get(&cache_key) {
                let coords = Coordinates::new(lat, lng);
                if let Ok(salah) = calculate_salah_schedule(date, coords, params, city_id) {
                    return Ok(SalahComputation { location, salah });
                }
            }
        }

        let mut coord_candidates = vec![(location.lat, location.lng)];
        coord_candidates.extend(self.fallback_coords_for(city_id));

        let mut last_err = BolootError::Prayer(format!(
            "prayer calculation failed for {city_id} on {date}"
        ));
        for (lat, lng) in coord_candidates {
            let failure_key = coord_failure_key(city_id, date, lat, lng);
            if let Ok(failed) = failed_coord_cache().lock() {
                if failed.contains(&failure_key) {
                    continue;
                }
            }

            let coords = Coordinates::new(lat, lng);
            match calculate_salah_schedule(date, coords, params, city_id) {
                Ok(salah) => {
                    if let Ok(mut cache) = coord_cache().lock() {
                        cache.insert(cache_key.clone(), (lat, lng));
                    }
                    return Ok(SalahComputation { location, salah });
                }
                Err(err) => {
                    if let Ok(mut failed) = failed_coord_cache().lock() {
                        failed.insert(failure_key);
                    }
                    last_err = err;
                }
            }
        }
        Err(last_err)
    }

    fn entries_from_salah(
        salah_times: &salah::PrayerTimes,
        tz: Tz,
        numerals: NumeralStyle,
        language: LanguageVariant,
    ) -> Vec<PrayerTimeEntry> {
        [
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
        .collect()
    }

    fn resolve_location(
        &self,
        city_id: &str,
        latitude: Option<f64>,
        longitude: Option<f64>,
        timezone: &str,
    ) -> Result<ResolvedPrayerLocation> {
        if let (Some(lat), Some(lng)) = (latitude, longitude) {
            if let Some(city) = self.locations.get(city_id) {
                return Ok(ResolvedPrayerLocation {
                    lat,
                    lng,
                    timezone: if timezone.is_empty() {
                        city.timezone.clone()
                    } else {
                        timezone.to_string()
                    },
                    display_name: city.name_fa.clone(),
                });
            }
            if timezone.is_empty() {
                return Err(BolootError::InvalidConfig(
                    "timezone required for custom prayer coordinates".into(),
                ));
            }
            return Ok(ResolvedPrayerLocation {
                lat,
                lng,
                timezone: timezone.to_string(),
                display_name: "مختصات سفارشی".into(),
            });
        }

        let city = self
            .locations
            .get(city_id)
            .ok_or_else(|| BolootError::LocationNotFound(city_id.to_string()))?;
        Ok(ResolvedPrayerLocation {
            lat: city.latitude,
            lng: city.longitude,
            timezone: if timezone.is_empty() {
                city.timezone.clone()
            } else {
                timezone.to_string()
            },
            display_name: city.name_fa.clone(),
        })
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
        let SalahComputation { location, salah } =
            self.compute_salah(date, city_id, latitude, longitude, timezone, method, madhab)?;

        let tz: Tz = location
            .timezone
            .parse()
            .map_err(|_| BolootError::Prayer(format!("invalid timezone: {}", location.timezone)))?;

        let entries = Self::entries_from_salah(&salah, tz, numerals, language);

        Ok(PrayerTimes {
            date,
            timezone: location.timezone,
            city: location.display_name,
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
        let SalahComputation { location, salah } =
            self.compute_salah(date, city_id, latitude, longitude, timezone, method, madhab)?;

        let tz: Tz = location
            .timezone
            .parse()
            .map_err(|_| BolootError::Prayer(format!("invalid timezone: {}", location.timezone)))?;

        let entries = Self::entries_from_salah(&salah, tz, numerals, language);
        let times = PrayerTimes {
            date,
            timezone: location.timezone.clone(),
            city: location.display_name.clone(),
            entries,
        };

        let (current, next) = Self::resolve_current_and_next(
            date,
            &times.entries,
            tz,
            &salah,
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
        let now = local_now().with_timezone(&Utc);
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
    fn global_city_prayer_times() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        for (city_id, tz, method) in [
            ("dubai", "Asia/Dubai", PrayerCalculationMethod::Dubai),
            ("new_york", "America/New_York", PrayerCalculationMethod::Isna),
            ("tokyo", "Asia/Tokyo", PrayerCalculationMethod::Mwl),
        ] {
            let times = engine
                .calculate(
                    date,
                    city_id,
                    None,
                    None,
                    tz,
                    method,
                    PrayerMadhab::Jafari,
                    NumeralStyle::Latin,
                    LanguageVariant::Persian,
                )
                .unwrap();
            assert_eq!(times.entries.len(), 6, "city {city_id}");
            assert_eq!(times.timezone, tz);
        }
    }

    #[test]
    fn location_store_loads_global_cities() {
        let store = LocationStore::load().expect("load locations");
        assert!(store.get("berlin").is_some());
        assert!(store.get("new_york").is_some());
        assert!(store.get("tokyo").is_some());
        assert!(store.get("tehran").is_some());
    }

    #[test]
    fn suggested_for_city_by_region() {
        let berlin = CityLocation {
            id: "berlin".into(),
            name: "Berlin".into(),
            name_fa: "برلین".into(),
            latitude: 52.52,
            longitude: 13.405,
            timezone: "Europe/Berlin".into(),
            country: "germany".into(),
            region: Some("europe".into()),
        };
        assert_eq!(
            PrayerCalculationMethod::suggested_for_city(&berlin),
            PrayerCalculationMethod::Mwl
        );

        let new_york = CityLocation {
            id: "new_york".into(),
            name: "New York".into(),
            name_fa: "نیویورک".into(),
            latitude: 40.7128,
            longitude: -74.006,
            timezone: "America/New_York".into(),
            country: "usa".into(),
            region: Some("north_america".into()),
        };
        assert_eq!(
            PrayerCalculationMethod::suggested_for_city(&new_york),
            PrayerCalculationMethod::Isna
        );

        let oslo = CityLocation {
            id: "oslo".into(),
            name: "Oslo".into(),
            name_fa: "اسلو".into(),
            latitude: 59.9139,
            longitude: 10.7522,
            timezone: "Europe/Oslo".into(),
            country: "norway".into(),
            region: Some("europe".into()),
        };
        assert_eq!(
            PrayerCalculationMethod::suggested_for_city(&oslo),
            PrayerCalculationMethod::MoonsightingCommittee
        );
    }

    #[test]
    fn custom_coordinates_without_catalog_city() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let times = engine
            .calculate(
                date,
                "custom",
                Some(35.6892),
                Some(51.3890),
                "Asia/Tehran",
                PrayerCalculationMethod::Tehran,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .unwrap();
        assert_eq!(times.city, "مختصات سفارشی");
        assert_eq!(times.entries.len(), 6);
    }

    #[test]
    fn custom_coordinates_require_timezone_without_catalog() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 12).unwrap();
        let err = engine
            .calculate(
                date,
                "custom",
                Some(52.52),
                Some(13.405),
                "",
                PrayerCalculationMethod::Mwl,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .unwrap_err();
        assert!(matches!(err, BolootError::InvalidConfig(_)));
    }

    #[test]
    fn khujand_uses_same_timezone_fallback_on_problem_date() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 7, 9).unwrap();
        let times = engine
            .calculate(
                date,
                "khujand",
                None,
                None,
                "Asia/Dushanbe",
                PrayerCalculationMethod::Mwl,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .expect("khujand should fallback to dushanbe");
        assert_eq!(times.city, "خجند");
        assert_eq!(times.entries.len(), 6);
    }

    #[test]
    fn khujand_schedule_can_be_polled_repeatedly() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 7, 9).unwrap();
        for _ in 0..3 {
            let status = engine
                .schedule(
                    date,
                    "khujand",
                    None,
                    None,
                    "Asia/Dushanbe",
                    PrayerCalculationMethod::Mwl,
                    PrayerMadhab::Jafari,
                    NumeralStyle::Latin,
                    LanguageVariant::Persian,
                )
                .expect("khujand schedule should succeed via fallback");
            assert_eq!(status.times.entries.len(), 6);
        }
    }

    #[test]
    fn salah_panic_is_caught_when_no_fallback_exists() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 7, 9).unwrap();
        let err = engine
            .calculate(
                date,
                "london",
                None,
                None,
                "Europe/London",
                PrayerCalculationMethod::Mwl,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .unwrap_err();
        assert!(matches!(err, BolootError::Prayer(_)));
    }

    #[test]
    fn tbilisi_prayer_times_on_problem_date() {
        let engine = PrayerEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 7, 9).unwrap();
        let times = engine
            .calculate(
                date,
                "tbilisi",
                None,
                None,
                "Asia/Tbilisi",
                PrayerCalculationMethod::Mwl,
                PrayerMadhab::Jafari,
                NumeralStyle::Latin,
                LanguageVariant::Persian,
            )
            .expect("tbilisi prayer times");
        assert_eq!(times.entries.len(), 6);
        assert_eq!(times.timezone, "Asia/Tbilisi");
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
