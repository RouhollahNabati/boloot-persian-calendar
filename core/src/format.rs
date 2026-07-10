use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::calendar::{CalendarDate, CalendarSystem};
use crate::locale::LocaleProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NumeralStyle {
    #[default]
    Persian,
    Latin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DateFormatStyle {
    #[default]
    ShortSlash,
    LongNamed,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFormatter {
    pub calendar: CalendarSystem,
    pub numerals: NumeralStyle,
    pub style: DateFormatStyle,
    pub custom_pattern: Option<String>,
    pub show_weekday: bool,
}

impl Default for DateFormatter {
    fn default() -> Self {
        Self {
            calendar: CalendarSystem::Jalali,
            numerals: NumeralStyle::Persian,
            style: DateFormatStyle::LongNamed,
            custom_pattern: None,
            show_weekday: true,
        }
    }
}

impl DateFormatter {
    pub fn format(&self, date: &CalendarDate, locale: &LocaleProfile) -> String {
        let body = match self.calendar {
            CalendarSystem::Jalali | CalendarSystem::DualJalaliGregorian => {
                self.format_jalali(date, locale)
            }
            CalendarSystem::Hijri => self.format_hijri(date, locale),
            CalendarSystem::Gregorian => self.format_gregorian(date, locale),
        };

        if self.show_weekday {
            let weekday = locale
                .weekday_name(date.weekday, self.calendar)
                .unwrap_or("---")
                .to_string();
            format!("{weekday} {body}")
        } else {
            body
        }
    }

    pub fn format_secondary(&self, date: &CalendarDate, locale: &LocaleProfile) -> Option<String> {
        if self.calendar != CalendarSystem::DualJalaliGregorian {
            return None;
        }
        Some(self.with_calendar(CalendarSystem::Gregorian).format(date, locale))
    }

    fn with_calendar(&self, calendar: CalendarSystem) -> Self {
        let mut clone = self.clone();
        clone.calendar = calendar;
        clone.show_weekday = false;
        clone
    }

    fn format_jalali(&self, date: &CalendarDate, locale: &LocaleProfile) -> String {
        match self.style {
            DateFormatStyle::ShortSlash => self.join_slash(&[
                date.jalali_year,
                date.jalali_month as i32,
                date.jalali_day as i32,
            ]),
            DateFormatStyle::LongNamed => {
                let month = locale
                    .month_name(date.jalali_month)
                    .unwrap_or("---");
                format!(
                    "{} {} {}",
                    self.num(date.jalali_day as i32),
                    month,
                    self.num(date.jalali_year)
                )
            }
            DateFormatStyle::Custom => {
                let pattern = self.custom_pattern.as_deref().unwrap_or("%D %M %Y");
                self.apply_pattern(pattern, date, locale, true)
            }
        }
    }

    fn format_hijri(&self, date: &CalendarDate, locale: &LocaleProfile) -> String {
        match self.style {
            DateFormatStyle::ShortSlash => self.join_slash(&[
                date.hijri_year,
                date.hijri_month as i32,
                date.hijri_day as i32,
            ]),
            DateFormatStyle::LongNamed => {
                let month = locale
                    .hijri_month_name(date.hijri_month)
                    .unwrap_or("---");
                format!(
                    "{} {} {}",
                    self.num(date.hijri_day as i32),
                    month,
                    self.num(date.hijri_year)
                )
            }
            DateFormatStyle::Custom => {
                let pattern = self.custom_pattern.as_deref().unwrap_or("%D %M %Y");
                self.apply_pattern(pattern, date, locale, false)
            }
        }
    }

    fn format_gregorian(&self, date: &CalendarDate, locale: &LocaleProfile) -> String {
        match self.style {
            DateFormatStyle::ShortSlash => self.join_slash(&[
                date.gregorian.year(),
                date.gregorian.month() as i32,
                date.gregorian.day() as i32,
            ]),
            DateFormatStyle::LongNamed => {
                let month = locale
                    .gregorian_month_name(date.gregorian.month())
                    .unwrap_or("---");
                format!(
                    "{} {} {}",
                    self.num(date.gregorian.day() as i32),
                    month,
                    self.num(date.gregorian.year())
                )
            }
            DateFormatStyle::Custom => {
                let pattern = self.custom_pattern.as_deref().unwrap_or("%D %M %Y");
                self.apply_pattern(pattern, date, locale, false)
            }
        }
    }

    fn join_slash(&self, parts: &[i32]) -> String {
        parts
            .iter()
            .map(|p| self.num(*p))
            .collect::<Vec<_>>()
            .join("/")
    }

    fn apply_pattern(
        &self,
        pattern: &str,
        date: &CalendarDate,
        locale: &LocaleProfile,
        jalali: bool,
    ) -> String {
        let mut out = String::new();
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '%' && i + 1 < chars.len() {
                match chars[i + 1] {
                    'Y' => {
                        let y = if jalali {
                            date.jalali_year
                        } else {
                            date.gregorian.year()
                        };
                        out.push_str(&self.num(y));
                    }
                    'M' => {
                        if jalali {
                            if let Some(name) = locale.month_name(date.jalali_month) {
                                out.push_str(name);
                            }
                        } else if self.calendar == CalendarSystem::Hijri {
                            if let Some(name) = locale.hijri_month_name(date.hijri_month) {
                                out.push_str(name);
                            }
                        } else if let Some(name) = locale.gregorian_month_name(date.gregorian.month())
                        {
                            out.push_str(name);
                        } else {
                            out.push_str(&self.num(date.gregorian.month() as i32));
                        }
                    }
                    'D' => {
                        let d = if jalali {
                            date.jalali_day as i32
                        } else {
                            date.gregorian.day() as i32
                        };
                        out.push_str(&self.num(d));
                    }
                    'W' => {
                        if let Some(name) = locale.weekday_name(date.weekday, self.calendar) {
                            out.push_str(name);
                        }
                    }
                    c => {
                        out.push('%');
                        out.push(c);
                    }
                }
                i += 2;
            } else {
                out.push(chars[i]);
                i += 1;
            }
        }
        out
    }

    fn num(&self, value: i32) -> String {
        let raw = value.to_string();
        match effective_numerals(self.calendar, self.numerals) {
            NumeralStyle::Latin => raw,
            NumeralStyle::Persian => to_persian_digits(&raw),
        }
    }
}

/// Gregorian calendar always uses Latin digits; other calendars follow `numerals`.
pub fn effective_numerals(calendar: CalendarSystem, numerals: NumeralStyle) -> NumeralStyle {
    if calendar == CalendarSystem::Gregorian {
        NumeralStyle::Latin
    } else {
        numerals
    }
}

pub fn to_persian_digits(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '0'..='9' => char::from_u32(0x06F0 + (c as u32 - '0' as u32)).unwrap_or(c),
            _ => c,
        })
        .collect()
}

pub fn format_time(hour: u32, minute: u32, numerals: NumeralStyle) -> String {
    let raw = format!("{hour:02}:{minute:02}");
    match numerals {
        NumeralStyle::Latin => raw,
        NumeralStyle::Persian => to_persian_digits(&raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{CalendarEngine, CalendarSystem};
    use crate::locale::{CountryProfile, LanguageVariant};

    #[test]
    fn gregorian_calendar_uses_latin_digits_even_with_persian_numerals() {
        let engine = CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian);
        let gregorian = engine.jalali_to_gregorian(1404, 1, 1).unwrap();
        let date = engine.on_date(gregorian).unwrap();
        let formatter = DateFormatter {
            calendar: CalendarSystem::Gregorian,
            numerals: NumeralStyle::Persian,
            ..DateFormatter::default()
        };
        let formatted = formatter.format(&date, engine.locale());
        assert!(formatted.contains("2025"));
        assert!(!formatted.contains('۲'));
    }

    #[test]
    fn persian_digits() {
        assert_eq!(to_persian_digits("1404"), "۱۴۰۴");
    }

    #[test]
    fn long_format() {
        let engine = CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian);
        let gregorian = engine.jalali_to_gregorian(1404, 1, 1).unwrap();
        let date = engine.on_date(gregorian).unwrap();
        let formatter = DateFormatter::default();
        let formatted = formatter.format(&date, engine.locale());
        assert!(formatted.contains("فروردین"));
        assert!(formatted.contains("۱۴۰۴"));
    }

    #[test]
    fn hijri_long_named_uses_arabic_month() {
        let engine = CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian);
        let gregorian = engine.hijri_to_gregorian(1447, 9, 15).unwrap();
        let date = engine.on_date(gregorian).unwrap();
        let formatter = DateFormatter {
            calendar: CalendarSystem::Hijri,
            ..DateFormatter::default()
        };
        let formatted = formatter.format(&date, engine.locale());
        assert!(formatted.contains("رمضان"));
        assert!(formatted.contains("ال"));
    }

    #[test]
    fn gregorian_long_named_uses_standard_month_name() {
        let engine = CalendarEngine::new(CountryProfile::iran(), LanguageVariant::Persian);
        let gregorian = engine.jalali_to_gregorian(1405, 4, 16).unwrap();
        let date = engine.on_date(gregorian).unwrap();
        let formatter = DateFormatter {
            calendar: CalendarSystem::Gregorian,
            show_weekday: false,
            ..DateFormatter::default()
        };
        let formatted = formatter.format(&date, engine.locale());
        assert!(formatted.contains("July"));
        assert!(formatted.contains("7 July 2026"));
    }
}
