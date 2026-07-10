use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::calendar::CalendarDate;
use crate::error::Result;
use crate::holidays::data_dir;
use crate::locale::LanguageVariant;

#[derive(Debug, Clone, Deserialize)]
struct WisdomFile {
    attribution: HashMap<String, String>,
    quotes: Vec<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct WisdomStore {
    attribution: HashMap<String, String>,
    quotes: Vec<HashMap<String, String>>,
}

impl WisdomStore {
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let path = dir.join("imam_ali.json");
        let raw = fs::read_to_string(&path)?;
        let file: WisdomFile = serde_json::from_str(&raw)?;
        Ok(Self {
            attribution: file.attribution,
            quotes: file.quotes,
        })
    }

    pub fn embedded() -> Result<Self> {
        Self::load_from_dir(&wisdom_dir())
    }

    pub fn quote_for_date(&self, date: &CalendarDate, lang: LanguageVariant) -> Option<String> {
        if self.quotes.is_empty() {
            return None;
        }
        let index = jalali_day_index(date.jalali_year, date.jalali_month, date.jalali_day)
            % self.quotes.len();
        let quote = localized_text(&self.quotes[index], lang)?;
        let attribution = localized_text(&self.attribution, lang)?;
        Some(format!("{attribution}:\n{quote}"))
    }

    pub fn len(&self) -> usize {
        self.quotes.len()
    }
}

pub fn wisdom_dir() -> std::path::PathBuf {
    data_dir().join("wisdom")
}

fn lang_key(lang: LanguageVariant) -> &'static str {
    match lang {
        LanguageVariant::English => "en",
        LanguageVariant::Persian => "fa",
        LanguageVariant::Dari => "fa_af",
        LanguageVariant::Pashto => "ps",
        LanguageVariant::Tajik => "tg",
    }
}

fn localized_text(map: &HashMap<String, String>, lang: LanguageVariant) -> Option<String> {
    let key = lang_key(lang);
    map.get(key)
        .or_else(|| map.get("fa"))
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            map.values()
                .find(|value| !value.trim().is_empty())
                .cloned()
        })
}

fn is_jalali_leap(year: i32) -> bool {
    matches!(
        year.rem_euclid(33),
        1 | 5 | 9 | 13 | 17 | 22 | 26 | 30
    )
}

fn jalali_month_lengths(year: i32) -> [u8; 12] {
    if is_jalali_leap(year) {
        [31, 31, 31, 31, 31, 31, 30, 30, 30, 30, 30, 30]
    } else {
        [31, 31, 31, 31, 31, 31, 30, 30, 30, 30, 30, 29]
    }
}

fn jalali_day_index(year: i32, month: u8, day: u8) -> usize {
    let lengths = jalali_month_lengths(year);
    let mut total = day as u32;
    for length in lengths.iter().take((month as usize).saturating_sub(1)) {
        total += *length as u32;
    }
    total.saturating_sub(1) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::calendar::CalendarEngine;
    use crate::locale::CountryProfile;

    fn store() -> WisdomStore {
        WisdomStore::embedded().expect("wisdom data should load in tests")
    }

    fn date_for(gregorian: NaiveDate) -> CalendarDate {
        CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian)
            .on_date(gregorian)
            .unwrap()
    }

    #[test]
    fn wisdom_store_loads_quotes() {
        let store = store();
        assert!(store.len() >= 100);
        assert!(store.attribution.contains_key("fa"));
        assert!(store.attribution.contains_key("en"));
    }

    fn assert_all_locales_present(store: &WisdomStore) {
        const REQUIRED: &[&str] = &["fa", "fa_af", "ps", "tg", "en"];
        for key in REQUIRED {
            let value = store.attribution.get(*key).map(String::as_str).unwrap_or("");
            assert!(!value.trim().is_empty(), "attribution missing {key}");
        }
        for (index, quote) in store.quotes.iter().enumerate() {
            for key in REQUIRED {
                let value = quote.get(*key).map(String::as_str).unwrap_or("");
                assert!(
                    !value.trim().is_empty(),
                    "quote {index} missing {key}"
                );
            }
        }
    }

    #[test]
    fn all_quotes_have_required_locales() {
        let store = store();
        assert_all_locales_present(&store);
    }

    #[test]
    fn quote_selection_is_stable_for_same_date() {
        let store = store();
        let date = date_for(NaiveDate::from_ymd_opt(2026, 3, 20).unwrap());
        let first = store.quote_for_date(&date, LanguageVariant::Persian);
        let second = store.quote_for_date(&date, LanguageVariant::Persian);
        assert_eq!(first, second);
        assert!(first.unwrap().starts_with("امیرالمومنین علی علیه السلام:"));
    }

    #[test]
    fn quote_includes_attribution() {
        let store = store();
        let date = date_for(NaiveDate::from_ymd_opt(2026, 7, 10).unwrap());
        let quote = store
            .quote_for_date(&date, LanguageVariant::Persian)
            .expect("quote should exist");
        assert!(quote.starts_with("امیرالمومنین علی علیه السلام:"));
        let lines: Vec<&str> = quote.splitn(2, '\n').collect();
        assert_eq!(lines.len(), 2);
        assert!(!lines[1].trim().is_empty());
    }

    #[test]
    fn english_quote_uses_latin_attribution() {
        let store = store();
        let date = date_for(NaiveDate::from_ymd_opt(2026, 7, 10).unwrap());
        let quote = store
            .quote_for_date(&date, LanguageVariant::English)
            .expect("quote should exist");
        assert!(quote.starts_with("Imam Ali (AS):"));
    }

    #[test]
    fn tajik_quote_uses_cyrillic() {
        let store = store();
        let date = date_for(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        let quote = store
            .quote_for_date(&date, LanguageVariant::Tajik)
            .expect("quote should exist");
        assert!(quote.contains("Имом"));
    }

    #[test]
    fn language_fallback_to_persian() {
        let store = store();
        let date = date_for(NaiveDate::from_ymd_opt(2026, 5, 5).unwrap());
        let quote = store
            .quote_for_date(&date, LanguageVariant::Dari)
            .expect("quote should exist");
        assert!(!quote.is_empty());
    }
}
