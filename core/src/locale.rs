use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::calendar::CalendarSystem;

pub use crate::countries::CountryProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LanguageVariant {
    #[default]
    Persian,
    Dari,
    Pashto,
    Tajik,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Weekday {
    #[default]
    #[serde(alias = "Saturday")]
    Saturday = 0,
    #[serde(alias = "Sunday")]
    Sunday = 1,
    #[serde(alias = "Monday")]
    Monday = 2,
    #[serde(alias = "Tuesday")]
    Tuesday = 3,
    #[serde(alias = "Wednesday")]
    Wednesday = 4,
    #[serde(alias = "Thursday")]
    Thursday = 5,
    #[serde(alias = "Friday")]
    Friday = 6,
}

impl Weekday {
    pub fn from_chrono(weekday: chrono::Weekday) -> Self {
        match weekday {
            chrono::Weekday::Mon => Self::Monday,
            chrono::Weekday::Tue => Self::Tuesday,
            chrono::Weekday::Wed => Self::Wednesday,
            chrono::Weekday::Thu => Self::Thursday,
            chrono::Weekday::Fri => Self::Friday,
            chrono::Weekday::Sat => Self::Saturday,
            chrono::Weekday::Sun => Self::Sunday,
        }
    }

    pub fn to_chrono(self) -> chrono::Weekday {
        match self {
            Self::Monday => chrono::Weekday::Mon,
            Self::Tuesday => chrono::Weekday::Tue,
            Self::Wednesday => chrono::Weekday::Wed,
            Self::Thursday => chrono::Weekday::Thu,
            Self::Friday => chrono::Weekday::Fri,
            Self::Saturday => chrono::Weekday::Sat,
            Self::Sunday => chrono::Weekday::Sun,
        }
    }

    pub fn index_from(self, week_start: Self) -> u8 {
        (self as u8 + 7 - week_start as u8) % 7
    }
}

impl FromStr for Weekday {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "saturday" => Ok(Self::Saturday),
            "sunday" => Ok(Self::Sunday),
            "monday" => Ok(Self::Monday),
            "tuesday" => Ok(Self::Tuesday),
            "wednesday" => Ok(Self::Wednesday),
            "thursday" => Ok(Self::Thursday),
            "friday" => Ok(Self::Friday),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleProfile {
    pub country: CountryProfile,
    pub language: LanguageVariant,
    pub month_names: Vec<String>,
    pub hijri_month_names: Vec<String>,
    pub weekday_names: Vec<String>,
    pub weekday_short: Vec<String>,
    pub hijri_weekday_names: Vec<String>,
    pub hijri_weekday_short: Vec<String>,
    pub default_week_start: Weekday,
    pub weekend_days: Vec<Weekday>,
    pub default_timezone: String,
    pub locale_code: String,
}

impl LocaleProfile {
    pub fn resolve(country: CountryProfile, language: LanguageVariant) -> Self {
        match (country.as_str(), language) {
            (_, LanguageVariant::English) => english_profile(&country),
            ("iran", _) => iran(country),
            ("afghanistan", LanguageVariant::Pashto) => afghanistan_pashto(country),
            ("afghanistan", _) => afghanistan_dari(country),
            ("tajikistan", _) => tajikistan(country),
            (_, LanguageVariant::Persian) => persian_diaspora_profile(&country),
            _ => english_profile(&country),
        }
    }

    pub fn month_name(&self, month: u8) -> Option<&str> {
        self.month_names.get(month as usize - 1).map(String::as_str)
    }

    pub fn weekday_name(&self, weekday: Weekday, calendar: CalendarSystem) -> Option<&str> {
        match calendar {
            CalendarSystem::Hijri => self
                .hijri_weekday_names
                .get(weekday as usize)
                .map(String::as_str),
            _ => self.weekday_names.get(weekday as usize).map(String::as_str),
        }
    }

    pub fn weekday_short(&self, weekday: Weekday, calendar: CalendarSystem) -> Option<&str> {
        match calendar {
            CalendarSystem::Hijri => self
                .hijri_weekday_short
                .get(weekday as usize)
                .map(String::as_str),
            _ => self.weekday_short.get(weekday as usize).map(String::as_str),
        }
    }

    pub fn hijri_month_name(&self, month: u8) -> Option<&str> {
        self.hijri_month_names.get(month as usize - 1).map(String::as_str)
    }

    pub fn gregorian_month_name(&self, month: u32) -> Option<&str> {
        gregorian_month_names()
            .get(month as usize - 1)
            .map(String::as_str)
    }
}

fn gregorian_month_names() -> &'static [String] {
    use std::sync::OnceLock;
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        [
            "January", "February", "March", "April", "May", "June", "July", "August",
            "September", "October", "November", "December",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    })
}

fn hijri_month_names() -> Vec<String> {
    vec![
        "محرم", "صفر", "ربيع الأول", "ربيع الثاني", "جمادي الأولى", "جمادي الآخرة", "رجب",
        "شعبان", "رمضان", "شوال", "ذو القعدة", "ذو الحجة",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn hijri_weekday_names() -> Vec<String> {
    vec![
        "السبت", "الاحد", "الاثنين", "الثلاثاء", "الاربعاء", "الخميس", "الجمعة",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn hijri_weekday_short() -> Vec<String> {
    vec!["س", "ح", "ن", "ث", "ر", "خ", "ج"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn iran(country: CountryProfile) -> LocaleProfile {
    LocaleProfile {
        country,
        language: LanguageVariant::Persian,
        month_names: vec![
            "فروردین", "اردیبهشت", "خرداد", "تیر", "مرداد", "شهریور", "مهر", "آبان", "آذر",
            "دی", "بهمن", "اسفند",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        hijri_month_names: hijri_month_names(),
        weekday_names: vec![
            "شنبه", "یکشنبه", "دوشنبه", "سه‌شنبه", "چهارشنبه", "پنج‌شنبه", "جمعه",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        weekday_short: vec!["ش", "ی", "د", "س", "چ", "پ", "ج"]
            .into_iter()
            .map(String::from)
            .collect(),
        hijri_weekday_names: hijri_weekday_names(),
        hijri_weekday_short: hijri_weekday_short(),
        default_week_start: Weekday::Saturday,
        weekend_days: vec![Weekday::Friday],
        default_timezone: "Asia/Tehran".into(),
        locale_code: "fa_IR".into(),
    }
}

fn afghanistan_dari(country: CountryProfile) -> LocaleProfile {
    LocaleProfile {
        country,
        language: LanguageVariant::Dari,
        month_names: vec![
            "حمل", "ثور", "جوزا", "سرطان", "اسد", "سنبله", "میزان", "عقرب", "قوس", "جدی",
            "دلو", "حوت",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        hijri_month_names: hijri_month_names(),
        weekday_names: vec![
            "شنبه", "یکشنبه", "دوشنبه", "سه‌شنبه", "چهارشنبه", "پنج‌شنبه", "جمعه",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        weekday_short: vec!["ش", "ی", "د", "س", "چ", "پ", "ج"]
            .into_iter()
            .map(String::from)
            .collect(),
        hijri_weekday_names: hijri_weekday_names(),
        hijri_weekday_short: hijri_weekday_short(),
        default_week_start: Weekday::Saturday,
        weekend_days: vec![Weekday::Thursday, Weekday::Friday],
        default_timezone: "Asia/Kabul".into(),
        locale_code: "fa_AF".into(),
    }
}

fn afghanistan_pashto(country: CountryProfile) -> LocaleProfile {
    let mut profile = afghanistan_dari(country);
    profile.language = LanguageVariant::Pashto;
    profile.month_names = vec![
        "وری", "غویی", "غبرګولی", "چنګاښ", "زمری", "وږی", "تله", "لړم", "لیندۍ", "مرغومی",
        "سلواغه", "کب",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    profile.locale_code = "ps_AF".into();
    profile
}

fn jalali_month_names_english() -> Vec<String> {
    vec![
        "Farvardin", "Ordibehesht", "Khordad", "Tir", "Mordad", "Shahrivar", "Mehr", "Aban",
        "Azar", "Dey", "Bahman", "Esfand",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn english_weekday_names() -> Vec<String> {
    vec![
        "Saturday", "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn english_weekday_short() -> Vec<String> {
    vec!["Sa", "Su", "Mo", "Tu", "We", "Th", "Fr"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn english_profile(country: &CountryProfile) -> LocaleProfile {
    LocaleProfile {
        country: country.clone(),
        language: LanguageVariant::English,
        month_names: jalali_month_names_english(),
        hijri_month_names: hijri_month_names(),
        weekday_names: english_weekday_names(),
        weekday_short: english_weekday_short(),
        hijri_weekday_names: hijri_weekday_names(),
        hijri_weekday_short: hijri_weekday_short(),
        default_week_start: country.default_week_start(),
        weekend_days: country.weekend_days(),
        default_timezone: country.default_timezone(),
        locale_code: "en".into(),
    }
}

fn persian_diaspora_profile(country: &CountryProfile) -> LocaleProfile {
    let mut profile = iran(country.clone());
    profile.default_week_start = country.default_week_start();
    profile.weekend_days = country.weekend_days();
    profile.default_timezone = country.default_timezone();
    profile
}

fn tajikistan(country: CountryProfile) -> LocaleProfile {
    LocaleProfile {
        country,
        language: LanguageVariant::Tajik,
        month_names: vec![
            "Фарвардин", "Урдибихишт", "Хурдод", "Тир", "Мурдод", "Шаҳривар", "Мехр", "Абон",
            "Азар", "Ди", "Бахман", "Исфанд",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        hijri_month_names: hijri_month_names(),
        weekday_names: vec![
            "Шанбе", "Якшанбе", "Душанбе", "Сешанбе", "Чоршанбе", "Панҷшанбе", "Ҷумъа",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        weekday_short: vec!["Ш", "Я", "Д", "С", "Ч", "П", "Ҷ"]
            .into_iter()
            .map(String::from)
            .collect(),
        hijri_weekday_names: hijri_weekday_names(),
        hijri_weekday_short: hijri_weekday_short(),
        default_week_start: Weekday::Monday,
        weekend_days: vec![Weekday::Saturday, Weekday::Sunday],
        default_timezone: "Asia/Dushanbe".into(),
        locale_code: "tg_TJ".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_locale_uses_latin_months() {
        let profile = LocaleProfile::resolve(CountryProfile::iran(), LanguageVariant::English);
        assert_eq!(profile.month_names[0], "Farvardin");
        assert_eq!(profile.weekday_names[0], "Saturday");
        assert_eq!(profile.locale_code, "en");
    }

    #[test]
    fn afghan_months_differ_from_iran() {
        let ir = iran(CountryProfile::iran());
        let af = afghanistan_dari(CountryProfile::afghanistan());
        assert_ne!(ir.month_names[0], af.month_names[0]);
        assert_eq!(ir.month_names[0], "فروردین");
        assert_eq!(af.month_names[0], "حمل");
    }
}
