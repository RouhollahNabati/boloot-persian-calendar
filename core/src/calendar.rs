use chrono::{Datelike, NaiveDate, Utc};
use hijri_date::HijriDate;
use parsidate::ParsiDate;
use serde::{Deserialize, Serialize};

use crate::error::{BolootError, Result};
use crate::locale::{CountryProfile, LanguageVariant, LocaleProfile, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalendarSystem {
    #[default]
    Jalali,
    Hijri,
    Gregorian,
    DualJalaliGregorian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDate {
    pub gregorian: NaiveDate,
    pub jalali_year: i32,
    pub jalali_month: u8,
    pub jalali_day: u8,
    pub hijri_year: i32,
    pub hijri_month: u8,
    pub hijri_day: u8,
    pub weekday: Weekday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarView {
    pub primary: String,
    pub secondary: Option<String>,
    pub weekday: String,
    pub weekday_short: String,
    pub panel_day_label: String,
    pub panel_tooltip: String,
    pub is_holiday: bool,
    pub is_weekend: bool,
    pub holiday_names: Vec<String>,
    pub jalali: CalendarDate,
}

pub struct CalendarEngine {
    locale: LocaleProfile,
}

impl CalendarEngine {
    pub fn new(country: CountryProfile, language: LanguageVariant) -> Self {
        Self {
            locale: LocaleProfile::resolve(country, language),
        }
    }

    pub fn with_locale(locale: LocaleProfile) -> Self {
        Self { locale }
    }

    pub fn locale(&self) -> &LocaleProfile {
        &self.locale
    }

    pub fn today(&self) -> Result<CalendarDate> {
        self.on_date(Utc::now().date_naive())
    }

    pub fn on_date(&self, date: NaiveDate) -> Result<CalendarDate> {
        let parsi = ParsiDate::from_gregorian(date)
            .map_err(|e| BolootError::InvalidDate(e.to_string()))?;

        let hijri = HijriDate::from_gr(
            date.year() as usize,
            date.month() as usize,
            date.day() as usize,
        )
        .map_err(|e| BolootError::InvalidDate(e))?;

        Ok(CalendarDate {
            gregorian: date,
            jalali_year: parsi.year(),
            jalali_month: parsi.month() as u8,
            jalali_day: parsi.day() as u8,
            hijri_year: hijri.year() as i32,
            hijri_month: hijri.month() as u8,
            hijri_day: hijri.day() as u8,
            weekday: Weekday::from_chrono(date.weekday()),
        })
    }

    pub fn jalali_to_gregorian(&self, year: i32, month: u8, day: u8) -> Result<NaiveDate> {
        let parsi = ParsiDate::new(year, month as u32, day as u32)
            .map_err(|e| BolootError::InvalidDate(e.to_string()))?;
        parsi
            .to_gregorian()
            .map_err(|e| BolootError::InvalidDate(e.to_string()))
    }

    pub fn days_in_jalali_month(&self, year: i32, month: u8) -> Result<u8> {
        let first = self.jalali_to_gregorian(year, month, 1)?;
        let next = if month == 12 {
            self.jalali_to_gregorian(year + 1, 1, 1)?
        } else {
            self.jalali_to_gregorian(year, month + 1, 1)?
        };
        Ok((next - first).num_days() as u8)
    }

    pub fn hijri_to_gregorian(&self, year: i32, month: u8, day: u8) -> Result<NaiveDate> {
        let hijri = HijriDate::from_hijri(year as usize, month as usize, day as usize)
            .map_err(|e| BolootError::InvalidDate(e))?;
        NaiveDate::from_ymd_opt(
            hijri.year_gr() as i32,
            hijri.month_gr() as u32,
            hijri.day_gr() as u32,
        )
        .ok_or_else(|| BolootError::InvalidDate("invalid gregorian date from hijri".into()))
    }

    pub fn days_in_hijri_month(&self, year: i32, month: u8) -> Result<u8> {
        let first = HijriDate::from_hijri(year as usize, month as usize, 1)
            .map_err(|e| BolootError::InvalidDate(e))?;
        Ok(first.month_len() as u8)
    }

    pub fn hijri_month_grid_padded(
        &self,
        hijri_year: i32,
        hijri_month: u8,
        week_start: Weekday,
    ) -> Result<Vec<CalendarDate>> {
        let days = self.days_in_hijri_month(hijri_year, hijri_month)?;
        let first = self.on_date(self.hijri_to_gregorian(hijri_year, hijri_month, 1)?)?;
        let leading = first.weekday.index_from(week_start) as usize;
        let total_cells = leading + days as usize;
        const MIN_ROWS: usize = 6;
        let rows = total_cells.div_ceil(7).max(MIN_ROWS);
        let padded_len = rows * 7;
        let trailing = padded_len - total_cells;

        let (prev_year, prev_month) = if hijri_month == 1 {
            (hijri_year - 1, 12)
        } else {
            (hijri_year, hijri_month - 1)
        };
        let prev_days = self.days_in_hijri_month(prev_year, prev_month)?;

        let mut grid = Vec::with_capacity(padded_len);

        for i in 0..leading {
            let day = prev_days - leading as u8 + i as u8 + 1;
            grid.push(self.on_date(self.hijri_to_gregorian(prev_year, prev_month, day)?)?);
        }

        for day in 1..=days {
            grid.push(self.on_date(self.hijri_to_gregorian(hijri_year, hijri_month, day)?)?);
        }

        let (next_year, next_month) = if hijri_month == 12 {
            (hijri_year + 1, 1)
        } else {
            (hijri_year, hijri_month + 1)
        };

        for day in 1..=trailing as u8 {
            grid.push(self.on_date(self.hijri_to_gregorian(next_year, next_month, day)?)?);
        }

        Ok(grid)
    }

    pub fn month_grid(
        &self,
        jalali_year: i32,
        jalali_month: u8,
        week_start: Weekday,
    ) -> Result<Vec<Option<CalendarDate>>> {
        let days = self.days_in_jalali_month(jalali_year, jalali_month)?;
        let first = self.on_date(self.jalali_to_gregorian(jalali_year, jalali_month, 1)?)?;
        let leading = first.weekday.index_from(week_start) as usize;
        let total_cells = leading + days as usize;
        let rows = total_cells.div_ceil(7);
        let mut grid = vec![None; rows * 7];

        for day in 1..=days {
            let date = self.on_date(self.jalali_to_gregorian(jalali_year, jalali_month, day)?)?;
            let index = leading + (day as usize - 1);
            grid[index] = Some(date);
        }

        Ok(grid)
    }

    /// Full month grid with leading/trailing days from adjacent months; length is always a multiple of 7.
    pub fn month_grid_padded(
        &self,
        jalali_year: i32,
        jalali_month: u8,
        week_start: Weekday,
    ) -> Result<Vec<CalendarDate>> {
        let days = self.days_in_jalali_month(jalali_year, jalali_month)?;
        let first = self.on_date(self.jalali_to_gregorian(jalali_year, jalali_month, 1)?)?;
        let leading = first.weekday.index_from(week_start) as usize;
        let total_cells = leading + days as usize;
        const MIN_ROWS: usize = 6;
        let rows = total_cells.div_ceil(7).max(MIN_ROWS);
        let padded_len = rows * 7;
        let trailing = padded_len - total_cells;

        let (prev_year, prev_month) = if jalali_month == 1 {
            (jalali_year - 1, 12)
        } else {
            (jalali_year, jalali_month - 1)
        };
        let prev_days = self.days_in_jalali_month(prev_year, prev_month)?;

        let mut grid = Vec::with_capacity(padded_len);

        for i in 0..leading {
            let day = prev_days - leading as u8 + i as u8 + 1;
            grid.push(self.on_date(self.jalali_to_gregorian(prev_year, prev_month, day)?)?);
        }

        for day in 1..=days {
            grid.push(self.on_date(self.jalali_to_gregorian(jalali_year, jalali_month, day)?)?);
        }

        let (next_year, next_month) = if jalali_month == 12 {
            (jalali_year + 1, 1)
        } else {
            (jalali_year, jalali_month + 1)
        };

        for day in 1..=trailing as u8 {
            grid.push(self.on_date(self.jalali_to_gregorian(next_year, next_month, day)?)?);
        }

        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jalali_gregorian_roundtrip() {
        let engine = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let gregorian = engine.jalali_to_gregorian(1404, 1, 1).unwrap();
        let cal = engine.on_date(gregorian).unwrap();
        assert_eq!(cal.jalali_year, 1404);
        assert_eq!(cal.jalali_month, 1);
        assert_eq!(cal.jalali_day, 1);
    }

    #[test]
    fn nowruz_1404_is_march_21() {
        let engine = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let gregorian = engine.jalali_to_gregorian(1404, 1, 1).unwrap();
        assert_eq!(gregorian, NaiveDate::from_ymd_opt(2025, 3, 21).unwrap());
    }

    #[test]
    fn nowruz_1403_is_march_20() {
        let engine = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let gregorian = engine.jalali_to_gregorian(1403, 1, 1).unwrap();
        assert_eq!(gregorian, NaiveDate::from_ymd_opt(2024, 3, 20).unwrap());
    }

    #[test]
    fn day_before_nowruz_1404_is_esfand_30() {
        let engine = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let eve = NaiveDate::from_ymd_opt(2025, 3, 20).unwrap();
        let cal = engine.on_date(eve).unwrap();
        assert_eq!(cal.jalali_year, 1403);
        assert_eq!(cal.jalali_month, 12);
        assert_eq!(cal.jalali_day, 30);
    }

    #[test]
    fn hijri_gregorian_roundtrip() {
        let engine = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let gregorian = engine.hijri_to_gregorian(1447, 9, 15).unwrap();
        let cal = engine.on_date(gregorian).unwrap();
        assert_eq!(cal.hijri_year, 1447);
        assert_eq!(cal.hijri_month, 9);
        assert_eq!(cal.hijri_day, 15);
    }
}
