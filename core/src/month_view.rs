use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::calendar::{CalendarEngine, CalendarSystem};
use crate::colors::appearance_tints;
use crate::config::BolootConfig;
use crate::error::Result;
use crate::format::{effective_numerals, to_persian_digits, NumeralStyle};
use crate::holidays::{Holiday, HolidayStore};
use crate::locale::{LanguageVariant, LocaleProfile, Weekday};
use crate::prayer::PrayerDayStatus;
use crate::ui_strings::UiStrings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthCell {
    pub day: Option<u8>,
    pub day_label: Option<String>,
    pub secondary_label: Option<String>,
    pub jalali_year: i32,
    pub jalali_month: u8,
    pub is_current_month: bool,
    pub is_today: bool,
    pub is_holiday: bool,
    pub is_weekend: bool,
    pub holiday_names: Vec<String>,
    pub tooltip: Option<String>,
    /// Gregorian date in ISO 8601 form (`YYYY-MM-DD`) for UI day selection.
    pub gregorian_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthView {
    pub jalali_year: i32,
    pub jalali_month: u8,
    pub display_year: i32,
    pub display_month: u8,
    pub calendar_type: CalendarSystem,
    pub month_name: String,
    pub year_label: String,
    pub title: String,
    pub weekday_headers: Vec<String>,
    /// Column indices (0–6) for weekend weekday headers, relative to `weekday_headers`.
    pub weekend_header_indices: Vec<u8>,
    pub cells: Vec<MonthCell>,
    pub holidays: Vec<Holiday>,
    pub prayer: Option<PrayerDayStatus>,
    pub appearance: MonthAppearance,
    pub ui: UiStrings,
    /// `"rtl"` for Persian/Dari/Pashto; `"ltr"` for Tajik.
    pub text_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthAppearance {
    pub font_family: String,
    pub font_size_pt: u8,
    pub text_color: String,
    pub background_color: String,
    pub holiday_color: String,
    pub today_color: String,
    pub prayer_color: String,
    pub today_background_color: String,
    pub holiday_background_color: String,
    pub use_system_theme: bool,
    pub apply_custom_appearance: bool,
}

pub fn build_month_view(
    config: &BolootConfig,
    calendar: &CalendarEngine,
    holidays: &HolidayStore,
    prayer: Option<PrayerDayStatus>,
    year: Option<i32>,
    month: Option<u8>,
) -> Result<MonthView> {
    match config.calendar.calendar_type {
        CalendarSystem::Hijri => build_hijri_month_view(config, calendar, holidays, prayer, year, month),
        _ => build_jalali_month_view(config, calendar, holidays, prayer, year, month),
    }
}

fn build_jalali_month_view(
    config: &BolootConfig,
    calendar: &CalendarEngine,
    holidays: &HolidayStore,
    prayer: Option<PrayerDayStatus>,
    year: Option<i32>,
    month: Option<u8>,
) -> Result<MonthView> {
    let today = calendar.today()?;
    let year = year.unwrap_or(today.jalali_year);
    let month = month.unwrap_or(today.jalali_month);
    let locale = calendar.locale();
    let week_start = config.calendar.week_start;
    let numerals = config.calendar.effective_numerals();
    let calendar_type = config.calendar.calendar_type;
    let dual_calendar = calendar_type == CalendarSystem::DualJalaliGregorian;

    let grid = calendar.month_grid_padded(year, month, week_start)?;
    let month_holidays = if config.calendar.show_holidays {
        holidays.for_month(config.calendar.country, year, month, calendar)
    } else {
        Vec::new()
    };

    let mut cells = Vec::with_capacity(grid.len());
    for date in grid {
        let is_current_month = date.jalali_year == year && date.jalali_month == month;
        let day = Some(date.jalali_day);
        let is_today = date.jalali_year == today.jalali_year
            && date.jalali_month == today.jalali_month
            && date.jalali_day == today.jalali_day;
        let day_holidays = if config.calendar.show_holidays {
            holidays.for_date(config.calendar.country, &date, calendar)
        } else {
            Vec::new()
        };
        let holiday_names: Vec<String> = day_holidays.iter().map(|h| h.name.clone()).collect();
        let is_weekend = locale.weekend_days.contains(&date.weekday);
        cells.push(MonthCell {
            day,
            day_label: day.map(|d| format_day(d, numerals)),
            secondary_label: dual_calendar.then(|| {
                format_day(
                    date.gregorian.day() as u8,
                    effective_numerals(CalendarSystem::Gregorian, config.calendar.numerals),
                )
            }),
            jalali_year: date.jalali_year,
            jalali_month: date.jalali_month,
            is_current_month,
            is_today,
            is_holiday: !holiday_names.is_empty(),
            is_weekend,
            holiday_names,
            tooltip: build_tooltip(&day_holidays),
            gregorian_date: format_gregorian_iso(&date.gregorian),
        });
    }

    let month_name = locale.month_name(month).unwrap_or("---").to_string();
    let year_label = format_year_label(year, numerals);
    let title = format_month_title(&month_name, year, numerals);
    let tints = appearance_tints(
        &config.appearance.today_color,
        &config.appearance.holiday_color,
    );

    Ok(MonthView {
        jalali_year: year,
        jalali_month: month,
        display_year: year,
        display_month: month,
        calendar_type,
        month_name,
        year_label,
        title,
        weekday_headers: weekday_headers(locale, week_start, numerals, calendar_type),
        weekend_header_indices: weekend_header_indices(locale, week_start),
        cells,
        holidays: month_holidays,
        prayer,
        appearance: month_appearance(config, &tints),
        ui: ui_strings_for_config(config),
        text_direction: text_direction_for_config(config),
    })
}

fn build_hijri_month_view(
    config: &BolootConfig,
    calendar: &CalendarEngine,
    holidays: &HolidayStore,
    prayer: Option<PrayerDayStatus>,
    year: Option<i32>,
    month: Option<u8>,
) -> Result<MonthView> {
    let today = calendar.today()?;
    let year = year.unwrap_or(today.hijri_year);
    let month = month.unwrap_or(today.hijri_month);
    let locale = calendar.locale();
    let week_start = config.calendar.week_start;
    let numerals = config.calendar.effective_numerals();
    let calendar_type = CalendarSystem::Hijri;

    let grid = calendar.hijri_month_grid_padded(year, month, week_start)?;
    let month_holidays = if config.calendar.show_holidays {
        holidays.for_hijri_month(config.calendar.country, year, month, calendar)
    } else {
        Vec::new()
    };

    let mut cells = Vec::with_capacity(grid.len());
    for date in grid {
        let is_current_month = date.hijri_year == year && date.hijri_month == month;
        let day = Some(date.hijri_day);
        let is_today = date.hijri_year == today.hijri_year
            && date.hijri_month == today.hijri_month
            && date.hijri_day == today.hijri_day;
        let day_holidays = if config.calendar.show_holidays {
            holidays.for_date(config.calendar.country, &date, calendar)
        } else {
            Vec::new()
        };
        let holiday_names: Vec<String> = day_holidays.iter().map(|h| h.name.clone()).collect();
        let is_weekend = locale.weekend_days.contains(&date.weekday);
        cells.push(MonthCell {
            day,
            day_label: day.map(|d| format_day(d, numerals)),
            secondary_label: None,
            jalali_year: date.jalali_year,
            jalali_month: date.jalali_month,
            is_current_month,
            is_today,
            is_holiday: !holiday_names.is_empty(),
            is_weekend,
            holiday_names,
            tooltip: build_tooltip(&day_holidays),
            gregorian_date: format_gregorian_iso(&date.gregorian),
        });
    }

    let month_name = locale.hijri_month_name(month).unwrap_or("---").to_string();
    let year_label = format_year_label(year, numerals);
    let title = format_month_title(&month_name, year, numerals);
    let tints = appearance_tints(
        &config.appearance.today_color,
        &config.appearance.holiday_color,
    );

    Ok(MonthView {
        jalali_year: today.jalali_year,
        jalali_month: today.jalali_month,
        display_year: year,
        display_month: month,
        calendar_type,
        month_name,
        year_label,
        title,
        weekday_headers: weekday_headers(locale, week_start, numerals, calendar_type),
        weekend_header_indices: weekend_header_indices(locale, week_start),
        cells,
        holidays: month_holidays,
        prayer,
        appearance: month_appearance(config, &tints),
        ui: ui_strings_for_config(config),
        text_direction: text_direction_for_config(config),
    })
}

fn format_gregorian_iso(date: &chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn text_direction_for_language(language: LanguageVariant) -> String {
    match language {
        LanguageVariant::Tajik => "ltr".to_string(),
        LanguageVariant::Persian | LanguageVariant::Dari | LanguageVariant::Pashto => {
            "rtl".to_string()
        }
    }
}

fn text_direction_for_config(config: &BolootConfig) -> String {
    if config.calendar.calendar_type == CalendarSystem::Gregorian {
        return "ltr".to_string();
    }
    text_direction_for_language(config.calendar.language)
}

fn ui_strings_for_config(config: &BolootConfig) -> UiStrings {
    let mut ui = UiStrings::for_language(config.calendar.language);
    if config.calendar.calendar_type == CalendarSystem::Gregorian {
        ui.apply_english_nav_labels();
    }
    ui
}

fn month_appearance(
    config: &BolootConfig,
    tints: &crate::colors::AppearanceTints,
) -> MonthAppearance {
    MonthAppearance {
        font_family: config.appearance.font_family.clone(),
        font_size_pt: config.appearance.font_size_pt,
        text_color: config.appearance.text_color.clone(),
        background_color: config.appearance.background_color.clone(),
        holiday_color: config.appearance.holiday_color.clone(),
        today_color: config.appearance.today_color.clone(),
        prayer_color: config.appearance.prayer_color.clone(),
        today_background_color: tints.today_background.clone(),
        holiday_background_color: tints.holiday_background.clone(),
        use_system_theme: config.appearance.use_system_theme,
        apply_custom_appearance: !config.appearance.use_system_theme,
    }
}

fn format_year_label(year: i32, numerals: NumeralStyle) -> String {
    match numerals {
        NumeralStyle::Persian => to_persian_digits(&year.to_string()),
        NumeralStyle::Latin => year.to_string(),
    }
}

fn format_month_title(month_name: &str, year: i32, numerals: NumeralStyle) -> String {
    format!("{month_name} {}", format_year_label(year, numerals))
}

fn build_tooltip(holidays: &[Holiday]) -> Option<String> {
    if holidays.is_empty() {
        return None;
    }
    let names: Vec<String> = holidays
        .iter()
        .map(|h| {
            if h.is_lunar {
                format!("≈ {}", h.name)
            } else {
                h.name.clone()
            }
        })
        .collect();
    Some(names.join("، "))
}

fn weekend_header_indices(locale: &LocaleProfile, week_start: Weekday) -> Vec<u8> {
    let mut indices: Vec<u8> = locale
        .weekend_days
        .iter()
        .map(|day| day.index_from(week_start))
        .collect();
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn weekday_headers(
    locale: &LocaleProfile,
    week_start: Weekday,
    numerals: NumeralStyle,
    calendar_type: CalendarSystem,
) -> Vec<String> {
    let order = [
        Weekday::Saturday,
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
    ];
    let start = week_start as usize;
    order
        .iter()
        .cycle()
        .skip(start)
        .take(7)
        .map(|day| {
            let label = locale.weekday_short(*day, calendar_type).unwrap_or("-");
            match numerals {
                NumeralStyle::Persian => label.to_string(),
                NumeralStyle::Latin => label.to_string(),
            }
        })
        .collect()
}

pub fn format_day(day: u8, numerals: NumeralStyle) -> String {
    match numerals {
        NumeralStyle::Persian => to_persian_digits(&day.to_string()),
        NumeralStyle::Latin => day.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{CalendarEngine, CalendarSystem};
    use crate::config::BolootConfig;
    use crate::format::NumeralStyle;
    use crate::holidays::HolidayStore;
    use crate::locale::{CountryProfile, LanguageVariant, Weekday};

    fn test_service_config() -> BolootConfig {
        BolootConfig::default()
    }

    #[test]
    fn format_day_persian_digits() {
        assert_eq!(format_day(15, NumeralStyle::Persian), "۱۵");
        assert_eq!(format_day(15, NumeralStyle::Latin), "15");
    }

    #[test]
    fn padded_grid_has_seven_columns() {
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let grid = calendar
            .month_grid_padded(1404, 4, Weekday::Saturday)
            .unwrap();
        assert_eq!(grid.len() % 7, 0);
        assert_eq!(grid.len(), 42);
    }

    #[test]
    fn hijri_padded_grid_has_seven_columns() {
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let grid = calendar
            .hijri_month_grid_padded(1447, 9, Weekday::Saturday)
            .unwrap();
        assert_eq!(grid.len() % 7, 0);
        assert!(grid.len() >= 42);
    }

    #[test]
    fn padding_days_are_outside_current_month() {
        let config = test_service_config();
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(4),
        )
        .unwrap();

        assert!(view.cells.iter().any(|c| !c.is_current_month));
        assert!(view.cells.iter().any(|c| c.is_current_month));
        for cell in &view.cells {
            if !cell.is_current_month {
                assert!(
                    cell.jalali_month != view.jalali_month || cell.jalali_year != view.jalali_year
                );
            }
        }
    }

    #[test]
    fn dual_calendar_includes_secondary_label() {
        let mut config = test_service_config();
        config.calendar.calendar_type = CalendarSystem::DualJalaliGregorian;
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(1),
        )
        .unwrap();

        let with_secondary: Vec<_> = view
            .cells
            .iter()
            .filter(|c| c.secondary_label.is_some())
            .collect();
        assert!(!with_secondary.is_empty());
    }

    #[test]
    fn title_uses_month_name_and_year() {
        let config = test_service_config();
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(1),
        )
        .unwrap();

        assert!(view.title.contains(&view.month_name));
        assert!(view.title.contains(&view.year_label));
        assert!(view.year_label.contains('۴') || view.year_label.contains("1404"));
    }

    #[test]
    fn hijri_month_view_uses_arabic_weekday_headers() {
        let mut config = test_service_config();
        config.calendar.calendar_type = CalendarSystem::Hijri;
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1447),
            Some(9),
        )
        .unwrap();

        assert_eq!(view.calendar_type, CalendarSystem::Hijri);
        assert!(view.title.contains("رمضان"));
        assert!(view.weekday_headers.iter().any(|h| h.contains('س') || h == "س"));
        assert!(view
            .weekday_headers
            .iter()
            .any(|h| h.contains("الجمعة") || h == "ج"));
    }

    #[test]
    fn weekend_header_indices_serialize_in_month_view() {
        let config = test_service_config();
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(4),
        )
        .unwrap();
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("weekend_header_indices"));
        assert!(json.contains("[6]"));
    }

    #[test]
    fn weekend_header_indices_mark_friday_for_iran() {
        let config = test_service_config();
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(4),
        )
        .unwrap();

        assert_eq!(view.weekend_header_indices, vec![6]);
    }

    #[test]
    fn cells_include_gregorian_date() {
        let config = test_service_config();
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let view = build_month_view(
            &config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(1),
        )
        .unwrap();

        for cell in &view.cells {
            assert!(
                cell.gregorian_date.len() == 10,
                "expected ISO date, got {:?}",
                cell.gregorian_date
            );
            assert_eq!(cell.gregorian_date.as_bytes()[4], b'-');
            assert_eq!(cell.gregorian_date.as_bytes()[7], b'-');
        }
    }

    #[test]
    fn gregorian_calendar_uses_english_nav_and_ltr() {
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();

        let mut gregorian_config = test_service_config();
        gregorian_config.calendar.calendar_type = CalendarSystem::Gregorian;
        let view = build_month_view(
            &gregorian_config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(1),
        )
        .unwrap();
        assert_eq!(view.text_direction, "ltr");
        assert_eq!(view.ui.prev_month_label, "Previous month");
        assert_eq!(view.ui.next_month_label, "Next month");
    }

    #[test]
    fn text_direction_is_rtl_for_persian_and_ltr_for_tajik() {
        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let holidays = HolidayStore::embedded().unwrap_or_default();

        let mut persian_config = test_service_config();
        persian_config.calendar.language = LanguageVariant::Persian;
        let persian_view = build_month_view(
            &persian_config,
            &calendar,
            &holidays,
            None,
            Some(1404),
            Some(1),
        )
        .unwrap();
        assert_eq!(persian_view.text_direction, "rtl");

        let mut tajik_config = test_service_config();
        tajik_config.calendar.language = LanguageVariant::Tajik;
        let tajik_calendar =
            CalendarEngine::new(CountryProfile::Tajikistan, LanguageVariant::Tajik);
        let tajik_view = build_month_view(
            &tajik_config,
            &tajik_calendar,
            &holidays,
            None,
            Some(1404),
            Some(1),
        )
        .unwrap();
        assert_eq!(tajik_view.text_direction, "ltr");
    }

    #[test]
    fn lunar_holiday_tooltip_has_approximate_prefix() {
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let dir = crate::holidays::holidays_dir();
        if !dir.exists() {
            return;
        }

        let calendar = CalendarEngine::new(CountryProfile::Iran, LanguageVariant::Persian);
        let year_holidays = holidays.for_year(CountryProfile::Iran, 1404, &calendar);
        let lunar = year_holidays.iter().find(|h| h.is_lunar);
        if let Some(holiday) = lunar {
            let tooltip = build_tooltip(&[holiday.clone()]).unwrap();
            assert!(tooltip.starts_with('≈'));
        }
    }
}
