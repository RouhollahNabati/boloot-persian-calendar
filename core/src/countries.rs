use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{BolootError, Result};
use crate::holidays::data_dir;
use crate::locale::Weekday;

static REGISTRY: OnceLock<CountryRegistry> = OnceLock::new();

/// Calendar country identifier (`snake_case`, e.g. `"iran"`, `"germany"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CountryProfile(String);

impl Default for CountryProfile {
    fn default() -> Self {
        Self::iran()
    }
}

impl CountryProfile {
    pub fn iran() -> Self {
        Self("iran".into())
    }

    pub fn afghanistan() -> Self {
        Self("afghanistan".into())
    }

    pub fn tajikistan() -> Self {
        Self("tajikistan".into())
    }

    pub fn new(id: impl Into<String>) -> Self {
        Self(normalize_id(&id.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_iran(&self) -> bool {
        self.0 == "iran"
    }

    pub fn is_afghanistan(&self) -> bool {
        self.0 == "afghanistan"
    }

    pub fn is_tajikistan(&self) -> bool {
        self.0 == "tajikistan"
    }

    pub fn record(&self) -> Option<&CountryRecord> {
        registry().get(self.as_str())
    }

    pub fn default_timezone(&self) -> String {
        self.record()
            .map(|r| r.default_timezone.clone())
            .unwrap_or_else(|| "Asia/Tehran".into())
    }

    pub fn default_week_start(&self) -> Weekday {
        self.record()
            .and_then(|r| Weekday::from_str(&r.week_start).ok())
            .unwrap_or(Weekday::Saturday)
    }

    pub fn weekend_days(&self) -> Vec<Weekday> {
        self.record()
            .map(|r| {
                r.weekend_days
                    .iter()
                    .filter_map(|day| Weekday::from_str(day).ok())
                    .collect()
            })
            .unwrap_or_else(|| vec![Weekday::Friday])
    }

    pub fn prayer_method_id(&self) -> &str {
        self.record()
            .map(|r| r.prayer_method.as_str())
            .unwrap_or("mwl")
    }

    pub fn prayer_madhab_id(&self) -> &str {
        self.record()
            .map(|r| r.prayer_madhab.as_str())
            .unwrap_or("jafari")
    }
}

impl FromStr for CountryProfile {
    type Err = BolootError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(normalize_id(s)))
    }
}

impl Serialize for CountryProfile {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CountryProfile {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self(normalize_id(&raw)))
    }
}

fn normalize_id(raw: &str) -> String {
    let id = raw.trim().to_ascii_lowercase().replace('-', "_");
    if id.is_empty() {
        "iran".into()
    } else {
        id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountryRecord {
    pub id: String,
    pub iso_alpha2: String,
    pub name_en: String,
    pub name_fa: String,
    pub region: String,
    pub default_timezone: String,
    pub week_start: String,
    pub weekend_days: Vec<String>,
    pub default_language: String,
    pub languages: Vec<String>,
    pub prayer_method: String,
    pub prayer_madhab: String,
    pub capital_city_id: String,
}

#[derive(Debug, Deserialize)]
struct CountriesFile {
    countries: Vec<CountryRecord>,
}

#[derive(Debug, Default)]
pub struct CountryRegistry {
    by_id: HashMap<String, CountryRecord>,
}

impl CountryRegistry {
    pub fn load_from_dir(data_root: &Path) -> Result<Self> {
        let path = data_root.join("countries.json");
        if !path.exists() {
            return Ok(Self::fallback());
        }
        let raw = fs::read_to_string(path)?;
        let file: CountriesFile = serde_json::from_str(&raw)?;
        let mut by_id = HashMap::new();
        for record in file.countries {
            by_id.insert(record.id.clone(), record);
        }
        Ok(Self { by_id })
    }

    pub fn embedded() -> Result<Self> {
        Self::load_from_dir(&data_dir())
    }

    fn fallback() -> Self {
        let mut by_id = HashMap::new();
        for record in [
            CountryRecord {
                id: "iran".into(),
                iso_alpha2: "IR".into(),
                name_en: "Iran".into(),
                name_fa: "ایران".into(),
                region: "middle_east".into(),
                default_timezone: "Asia/Tehran".into(),
                week_start: "saturday".into(),
                weekend_days: vec!["friday".into()],
                default_language: "persian".into(),
                languages: vec!["persian".into(), "english".into()],
                prayer_method: "tehran".into(),
                prayer_madhab: "jafari".into(),
                capital_city_id: "tehran".into(),
            },
            CountryRecord {
                id: "afghanistan".into(),
                iso_alpha2: "AF".into(),
                name_en: "Afghanistan".into(),
                name_fa: "افغانستان".into(),
                region: "south_asia".into(),
                default_timezone: "Asia/Kabul".into(),
                week_start: "saturday".into(),
                weekend_days: vec!["thursday".into(), "friday".into()],
                default_language: "dari".into(),
                languages: vec!["dari".into(), "pashto".into(), "english".into()],
                prayer_method: "karachi".into(),
                prayer_madhab: "hanafi".into(),
                capital_city_id: "kabul".into(),
            },
            CountryRecord {
                id: "tajikistan".into(),
                iso_alpha2: "TJ".into(),
                name_en: "Tajikistan".into(),
                name_fa: "تاجیکستان".into(),
                region: "russia_central_asia".into(),
                default_timezone: "Asia/Dushanbe".into(),
                week_start: "monday".into(),
                weekend_days: vec!["saturday".into(), "sunday".into()],
                default_language: "tajik".into(),
                languages: vec!["tajik".into(), "english".into()],
                prayer_method: "mwl".into(),
                prayer_madhab: "hanafi".into(),
                capital_city_id: "dushanbe".into(),
            },
        ] {
            by_id.insert(record.id.clone(), record);
        }
        Self { by_id }
    }

    pub fn get(&self, id: &str) -> Option<&CountryRecord> {
        self.by_id.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }

    pub fn all_sorted(&self) -> Vec<&CountryRecord> {
        let mut records: Vec<_> = self.by_id.values().collect();
        records.sort_by(|a, b| a.name_fa.cmp(&b.name_fa));
        records
    }

    pub fn iso_to_id(&self, iso_alpha2: &str) -> Option<&str> {
        let upper = iso_alpha2.to_ascii_uppercase();
        self.by_id
            .values()
            .find(|r| r.iso_alpha2.eq_ignore_ascii_case(&upper))
            .map(|r| r.id.as_str())
    }
}

pub fn registry() -> &'static CountryRegistry {
    REGISTRY.get_or_init(|| CountryRegistry::embedded().unwrap_or_default())
}

pub fn countries_dir() -> std::path::PathBuf {
    data_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_country_registry_if_present() {
        let registry = CountryRegistry::embedded().unwrap();
        if registry.get("iran").is_none() {
            return;
        }
        assert!(registry.get("germany").is_some());
        assert!(registry.all_sorted().len() >= 190);
    }

    #[test]
    fn deserializes_country_profile_from_string() {
        let profile: CountryProfile = serde_json::from_str("\"germany\"").unwrap();
        assert_eq!(profile.as_str(), "germany");
    }
}
