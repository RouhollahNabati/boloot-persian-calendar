use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDate};
use hijri_date::HijriDate;
use serde::{Deserialize, Serialize};

use crate::calendar::{CalendarDate, CalendarEngine};
use crate::countries::CountryProfile;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub name: String,
    pub jalali_month: u8,
    pub jalali_day: u8,
    pub jalali_year: i32,
    pub is_lunar: bool,
    pub hijri_month: Option<u8>,
    pub hijri_day: Option<u8>,
    pub kind: HolidayKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HolidayKind {
    National,
    Religious,
    Cultural,
}

#[derive(Debug, Clone)]
struct HolidayTemplate {
    name: String,
    month: u8,
    day: u8,
    lunar: bool,
    fixed_jalali_year: Option<i32>,
    kind: HolidayKind,
}

#[derive(Debug, Clone, Deserialize)]
struct HolidayFile {
    #[allow(dead_code)]
    country: String,
    holidays: Vec<HolidayEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct HolidayEntry {
    name: String,
    month: u8,
    day: u8,
    #[serde(default)]
    year: Option<i32>,
    #[serde(default)]
    lunar: bool,
    #[serde(default = "default_kind")]
    kind: HolidayKind,
}

fn default_kind() -> HolidayKind {
    HolidayKind::National
}

#[derive(Debug, Default)]
pub struct HolidayStore {
    templates: HashMap<String, Vec<HolidayTemplate>>,
}

impl HolidayStore {
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let mut store = Self::default();
        if !dir.is_dir() {
            return Ok(store);
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let country_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if country_id.is_empty() {
                continue;
            }
            store.load_file(&country_id, &path)?;
        }
        Ok(store)
    }

    pub fn embedded() -> Result<Self> {
        Self::load_from_dir(&holidays_dir())
    }

    fn load_file(&mut self, country_id: &str, path: &Path) -> Result<()> {
        let raw = fs::read_to_string(path)?;
        let file: HolidayFile = serde_json::from_str(&raw)?;
        let holidays = file
            .holidays
            .into_iter()
            .map(|entry| HolidayTemplate {
                name: entry.name,
                month: entry.month,
                day: entry.day,
                lunar: entry.lunar,
                fixed_jalali_year: entry.year,
                kind: entry.kind,
            })
            .collect();
        self.templates.insert(country_id.to_string(), holidays);
        Ok(())
    }

    pub fn for_month(
        &self,
        country: &CountryProfile,
        jalali_year: i32,
        jalali_month: u8,
        calendar: &CalendarEngine,
    ) -> Vec<Holiday> {
        self.templates
            .get(country.as_str())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|template| {
                resolve_template(&template, jalali_year, calendar).and_then(|(month, day)| {
                    if month == jalali_month {
                        Some(Holiday {
                            name: template.name,
                            jalali_month: month,
                            jalali_day: day,
                            jalali_year,
                            is_lunar: template.lunar,
                            hijri_month: template.lunar.then_some(template.month),
                            hijri_day: template.lunar.then_some(template.day),
                            kind: template.kind,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn for_hijri_month(
        &self,
        country: &CountryProfile,
        hijri_year: i32,
        hijri_month: u8,
        calendar: &CalendarEngine,
    ) -> Vec<Holiday> {
        let days = calendar.days_in_hijri_month(hijri_year, hijri_month).ok();
        let Some(days) = days else {
            return Vec::new();
        };

        let mut holidays = Vec::new();
        for day in 1..=days {
            let Ok(gregorian) = calendar.hijri_to_gregorian(hijri_year, hijri_month, day) else {
                continue;
            };
            let Ok(date) = calendar.on_date(gregorian) else {
                continue;
            };
            holidays.extend(self.for_date(country, &date, calendar));
        }

        holidays.sort_by(|a, b| {
            (a.jalali_month, a.jalali_day, &a.name).cmp(&(b.jalali_month, b.jalali_day, &b.name))
        });
        holidays.dedup_by(|a, b| {
            a.name == b.name
                && a.jalali_year == b.jalali_year
                && a.jalali_month == b.jalali_month
                && a.jalali_day == b.jalali_day
        });
        holidays
    }

    pub fn for_date(
        &self,
        country: &CountryProfile,
        date: &CalendarDate,
        calendar: &CalendarEngine,
    ) -> Vec<Holiday> {
        self.for_month(country, date.jalali_year, date.jalali_month, calendar)
            .into_iter()
            .filter(|h| h.jalali_day == date.jalali_day)
            .collect()
    }

    pub fn for_year(
        &self,
        country: &CountryProfile,
        jalali_year: i32,
        calendar: &CalendarEngine,
    ) -> Vec<Holiday> {
        let mut all = Vec::new();
        for month in 1..=12 {
            all.extend(self.for_month(country, jalali_year, month, calendar));
        }
        all
    }

    pub fn tomorrow_holidays(
        &self,
        country: &CountryProfile,
        today: &CalendarDate,
        calendar: &CalendarEngine,
    ) -> Vec<Holiday> {
        let tomorrow_gregorian = today.gregorian + chrono::Duration::days(1);
        if let Ok(tomorrow) = calendar.on_date(tomorrow_gregorian) {
            return self.for_date(country, &tomorrow, calendar);
        }
        Vec::new()
    }
}

fn resolve_template(
    template: &HolidayTemplate,
    jalali_year: i32,
    calendar: &CalendarEngine,
) -> Option<(u8, u8)> {
    if template.lunar {
        resolve_lunar_holiday(template.month, template.day, jalali_year, calendar)
    } else if template
        .fixed_jalali_year
        .map(|y| y == jalali_year)
        .unwrap_or(true)
    {
        Some((template.month, template.day))
    } else {
        None
    }
}

fn resolve_lunar_holiday(
    hijri_month: u8,
    hijri_day: u8,
    jalali_year: i32,
    calendar: &CalendarEngine,
) -> Option<(u8, u8)> {
    let (start_hijri, end_hijri) = hijri_year_range_for_jalali(jalali_year, calendar)?;
    for hijri_year in start_hijri..=end_hijri {
        let hijri = HijriDate::from_hijri(
            hijri_year,
            hijri_month as usize,
            hijri_day as usize,
        )
        .ok()?;
        let gregorian = NaiveDate::from_ymd_opt(
            hijri.year_gr() as i32,
            hijri.month_gr() as u32,
            hijri.day_gr() as u32,
        )?;
        let jalali = calendar.on_date(gregorian).ok()?;
        if jalali.jalali_year == jalali_year {
            return Some((jalali.jalali_month, jalali.jalali_day));
        }
    }
    None
}

fn hijri_year_range_for_jalali(
    jalali_year: i32,
    calendar: &CalendarEngine,
) -> Option<(usize, usize)> {
    let start_gregorian = calendar.jalali_to_gregorian(jalali_year, 1, 1).ok()?;
    let end_gregorian = calendar
        .jalali_to_gregorian(jalali_year, 12, calendar.days_in_jalali_month(jalali_year, 12).ok()?)
        .ok()?;

    let start_hijri = HijriDate::from_gr(
        start_gregorian.year() as usize,
        start_gregorian.month() as usize,
        start_gregorian.day() as usize,
    )
    .ok()?
    .year();
    let end_hijri = HijriDate::from_gr(
        end_gregorian.year() as usize,
        end_gregorian.month() as usize,
        end_gregorian.day() as usize,
    )
    .ok()?
    .year();

    Some((start_hijri, end_hijri))
}


pub fn countries_json_path() -> PathBuf {
    data_dir().join("countries.json")
}

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BOLOOT_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(test)]
    {
        let test_data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data");
        if test_data.exists() {
            return test_data;
        }
    }
    ["/usr/share/boloot-calendar/data", "./data"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("./data"))
}

pub fn holidays_dir() -> PathBuf {
    data_dir().join("holidays")
}

pub fn locations_dir() -> PathBuf {
    data_dir().join("locations")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::LanguageVariant;

    #[test]
    fn loads_iran_holidays_if_present() {
        let dir = holidays_dir();
        if !dir.exists() {
            return;
        }
        let store = HolidayStore::load_from_dir(&dir).unwrap();
        let calendar = CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian);
        let holidays = store.for_month(&CountryProfile::iran(), 1404, 1, &calendar);
        assert!(!holidays.is_empty());
    }

    #[test]
    fn resolves_eid_fitr_lunar() {
        let dir = holidays_dir();
        if !dir.exists() {
            return;
        }
        let store = HolidayStore::load_from_dir(&dir).unwrap();
        let calendar = CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian);
        let year_holidays = store.for_year(&CountryProfile::iran(), 1404, &calendar);
        let eid = year_holidays
            .iter()
            .find(|h| h.name.contains("عید فطر"));
        assert!(eid.is_some(), "Eid al-Fitr should resolve for 1404");
        let eid = eid.unwrap();
        assert!(eid.is_lunar);
        assert!(eid.jalali_month >= 1 && eid.jalali_month <= 12);
    }
}
