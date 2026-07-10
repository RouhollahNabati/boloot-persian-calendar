use chrono::{Datelike, NaiveDate};
use crate::format::format_time;
use crate::system_time::{local_time_of_day, local_today};

use crate::calendar::{CalendarEngine, CalendarView};
use crate::config::BolootConfig;
use crate::error::Result;
use crate::format::DateFormatter;
use crate::holidays::{Holiday, HolidayStore};
use crate::locale::LocaleProfile;
use crate::month_view::{build_month_view, format_day, MonthView};
use crate::prayer::{PrayerDayStatus, PrayerEngine};
use crate::ui_strings::UiStrings;
use crate::wisdom::WisdomStore;

pub struct BolootService {
    config: BolootConfig,
    calendar: CalendarEngine,
    holidays: HolidayStore,
    wisdom: WisdomStore,
    prayer: PrayerEngine,
}

impl BolootService {
    pub fn new(config: BolootConfig) -> Result<Self> {
        let calendar = CalendarEngine::new(
            config.calendar.country.clone(),
            config.calendar.language,
        );
        let holidays = HolidayStore::embedded().unwrap_or_default();
        let wisdom = WisdomStore::embedded().unwrap_or_default();
        let prayer = PrayerEngine::new();
        Ok(Self {
            config,
            calendar,
            holidays,
            wisdom,
            prayer,
        })
    }

    pub fn from_config_file() -> Result<Self> {
        Self::new(BolootConfig::load()?)
    }

    pub fn from_config_for_uid(uid: u32) -> Result<Self> {
        Self::new(BolootConfig::load_for_uid(uid)?)
    }

    pub fn config(&self) -> &BolootConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut BolootConfig {
        &mut self.config
    }

    pub fn calendar(&self) -> &CalendarEngine {
        &self.calendar
    }

    pub fn locale(&self) -> &LocaleProfile {
        self.calendar.locale()
    }

    pub fn today_view(&self) -> Result<CalendarView> {
        let date = self.calendar.today()?;
        let mut view = self.format_view(&date)?;
        view.panel_tooltip = self.build_wisdom_tooltip(&view);
        Ok(view)
    }

    pub fn view_for(&self, gregorian: NaiveDate) -> Result<CalendarView> {
        let date = self.calendar.on_date(gregorian)?;
        let mut view = self.format_view(&date)?;
        view.panel_tooltip = self.build_wisdom_tooltip(&view);
        Ok(view)
    }

    fn format_view(&self, date: &crate::calendar::CalendarDate) -> Result<CalendarView> {
        let formatter = DateFormatter {
            calendar: self.config.calendar.calendar_type,
            numerals: self.config.calendar.effective_numerals(),
            style: self.config.calendar.date_style,
            custom_pattern: None,
            show_weekday: true,
        };
        let calendar_type = self.config.calendar.calendar_type;
        let locale = self.calendar.locale();
        let day_holidays = if self.config.calendar.show_holidays {
            self.holidays
                .for_date(&self.config.calendar.country, date, &self.calendar)
        } else {
            Vec::new()
        };
        let holiday_names: Vec<String> = day_holidays.iter().map(|h| h.name.clone()).collect();
        let is_weekend = locale.weekend_days.contains(&date.weekday);
        Ok(CalendarView {
            primary: formatter.format(date, locale),
            secondary: formatter.format_secondary(date, locale),
            weekday: locale
                .weekday_name(date.weekday, calendar_type)
                .unwrap_or("---")
                .to_string(),
            weekday_short: locale
                .weekday_short(date.weekday, calendar_type)
                .unwrap_or("-")
                .to_string(),
            panel_day_label: self.panel_day_label(date),
            panel_tooltip: String::new(),
            is_holiday: !holiday_names.is_empty(),
            is_weekend,
            holiday_names,
            jalali: date.clone(),
        })
    }

    #[allow(dead_code)]
    fn build_panel_tooltip(&self, view: &CalendarView, include_prayer: bool) -> String {
        let formatter = DateFormatter {
            calendar: self.config.calendar.calendar_type,
            numerals: self.config.calendar.effective_numerals(),
            style: self.config.calendar.date_style,
            custom_pattern: None,
            show_weekday: false,
        };
        let locale = self.calendar.locale();
        let date_only = formatter.format(&view.jalali, locale);

        let mut lines = vec![view.weekday.clone(), date_only];
        if let Some(secondary) = &view.secondary {
            lines.push(secondary.clone());
        }

        if include_prayer && self.config.prayer.enabled {
            if let Ok(schedule) = self.prayer_today() {
                if let Some(next) = &schedule.next {
                    let ui = UiStrings::for_language(self.config.calendar.language);
                    lines.push(format!(
                        "{} {} {}",
                        ui.next_prayer_prefix, next.label, next.time
                    ));
                }
            }
        }

        lines.join("\n")
    }

    fn build_wisdom_tooltip(&self, view: &CalendarView) -> String {
        if !self.config.appearance.show_wisdom_tooltip {
            return String::new();
        }
        self.wisdom
            .quote_for_date(&view.jalali, self.config.calendar.language)
            .unwrap_or_default()
    }

    fn panel_day_label(&self, date: &crate::calendar::CalendarDate) -> String {
        use crate::calendar::CalendarSystem;
        let day = match self.config.calendar.calendar_type {
            CalendarSystem::Gregorian => date.gregorian.day() as u8,
            CalendarSystem::Hijri => date.hijri_day,
            _ => date.jalali_day,
        };
        format_day(day, self.config.calendar.effective_numerals())
    }

    pub fn month_view(
        &self,
        year: Option<i32>,
        month: Option<u8>,
    ) -> Result<MonthView> {
        let prayer = if self.config.prayer.enabled && self.config.prayer.show_in_popup {
            self.prayer_today().ok()
        } else {
            None
        };
        build_month_view(
            &self.config,
            &self.calendar,
            &self.holidays,
            prayer,
            year,
            month,
        )
    }

    pub fn month_grid(
        &self,
        jalali_year: i32,
        jalali_month: u8,
    ) -> Result<Vec<Option<crate::calendar::CalendarDate>>> {
        self.calendar.month_grid(
            jalali_year,
            jalali_month,
            self.config.calendar.week_start,
        )
    }

    pub fn holidays_for_today(&self) -> Result<Vec<Holiday>> {
        let date = self.calendar.today()?;
        Ok(self.holidays.for_date(
            &self.config.calendar.country,
            &date,
            &self.calendar,
        ))
    }

    pub fn holidays_for_month(&self, jalali_year: i32, jalali_month: u8) -> Vec<Holiday> {
        self.holidays.for_month(
            &self.config.calendar.country,
            jalali_year,
            jalali_month,
            &self.calendar,
        )
    }

    pub fn holidays_tomorrow(&self) -> Result<Vec<Holiday>> {
        let today = self.calendar.today()?;
        Ok(self.holidays.tomorrow_holidays(
            &self.config.calendar.country,
            &today,
            &self.calendar,
        ))
    }

    pub fn prayer_today(&self) -> Result<PrayerDayStatus> {
        self.prayer_for(local_today())
    }

    pub fn prayer_for(&self, date: NaiveDate) -> Result<PrayerDayStatus> {
        self.prayer.schedule(
            date,
            &self.config.prayer.city,
            self.config.prayer.latitude,
            self.config.prayer.longitude,
            &self.config.calendar.timezone,
            self.config.prayer.method,
            self.config.prayer.madhab,
            self.config.calendar.effective_numerals(),
            self.config.calendar.language,
        )
    }

    pub fn top_bar_text(&self) -> Result<String> {
        let view = self.today_view()?;
        let mut parts = Vec::new();

        if self.config.appearance.show_clock {
            let (hour, minute) = local_time_of_day();
            parts.push(format_time(
                hour,
                minute,
                self.config.calendar.effective_numerals(),
            ));
        }

        parts.push(view.primary);
        if let Some(secondary) = view.secondary {
            parts.push(secondary);
        }

        if self.config.prayer.enabled && self.config.prayer.show_in_top_bar {
            if let Ok(schedule) = self.prayer_today() {
                if let Some(text) = self.prayer.format_top_bar(
                    &schedule,
                    self.config.prayer.display_mode,
                    self.config.calendar.effective_numerals(),
                ) {
                    parts.push(text);
                }
            }
        }

        Ok(parts.join(" · "))
    }

    pub fn apply_settings(&mut self, new_config: BolootConfig) {
        let old_country = self.config.calendar.country.clone();
        let old_language = self.config.calendar.language;
        let old_city = self.config.prayer.city.clone();
        let old_tz = self.config.calendar.timezone.clone();
        self.config = new_config;
        if self.config.prayer.city != old_city {
            self.config
                .apply_prayer_city_change(Some(old_tz.as_str()));
        }
        if self.config.calendar.follow_system_locale {
            self.config.apply_system_locale();
        } else {
            self.config
                .apply_country_defaults_on_change(old_country, old_language);
        }
        self.calendar = CalendarEngine::new(
            self.config.calendar.country.clone(),
            self.config.calendar.language,
        );
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save()
    }

    pub fn save_config_for_uid(&self, uid: u32) -> Result<()> {
        self.config.save_for_uid(uid)
    }

    pub fn reload(&mut self) -> Result<()> {
        *self = Self::from_config_file()?;
        Ok(())
    }

    pub fn reload_for_uid(&mut self, uid: u32) -> Result<()> {
        *self = Self::from_config_for_uid(uid)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn panel_tooltip_shows_wisdom_when_enabled() {
        let service = BolootService::new(BolootConfig::default()).unwrap();
        let view = service.today_view().unwrap();

        assert!(view.panel_tooltip.starts_with("امیرالمومنین علی علیه السلام:"));
        assert!(view.panel_tooltip.contains('\n'));
    }

    #[test]
    fn panel_tooltip_shows_wisdom_in_english() {
        let mut config = BolootConfig::default();
        config.calendar.language = crate::locale::LanguageVariant::English;
        let service = BolootService::new(config).unwrap();
        let view = service.today_view().unwrap();

        assert!(view.panel_tooltip.starts_with("Imam Ali (AS):"));
        assert!(view.panel_tooltip.contains('\n'));
    }

    #[test]
    fn panel_tooltip_empty_when_wisdom_disabled() {
        let mut config = BolootConfig::default();
        config.appearance.show_wisdom_tooltip = false;
        let service = BolootService::new(config).unwrap();
        let view = service.today_view().unwrap();

        assert!(view.panel_tooltip.is_empty());
    }

    #[test]
    fn build_panel_tooltip_still_formats_date_lines() {
        let mut config = BolootConfig::default();
        config.prayer.enabled = false;
        let service = BolootService::new(config).unwrap();
        let view = service.today_view().unwrap();
        let tooltip = service.build_panel_tooltip(&view, true);

        assert!(!tooltip.contains("بعدی:"));
        assert!(tooltip.starts_with(&view.weekday));
        assert_eq!(tooltip.matches('\n').count(), 1);
    }

    #[test]
    fn panel_tooltip_with_prayer_includes_next_time() {
        let service = BolootService::new(BolootConfig::default()).unwrap();
        let view = service.today_view().unwrap();
        let tooltip = service.build_panel_tooltip(&view, true);

        assert!(tooltip.starts_with(&view.weekday));
        assert!(tooltip.contains('\n'));
        assert!(tooltip.contains("بعدی:"));
    }

    #[test]
    fn panel_tooltip_for_past_date_omits_prayer() {
        let service = BolootService::new(BolootConfig::default()).unwrap();
        let past = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let view = service.view_for(past).unwrap();
        let tooltip = service.build_panel_tooltip(&view, false);

        assert!(!tooltip.contains("بعدی:"));
        assert!(tooltip.starts_with(&view.weekday));
    }

    #[test]
    fn top_bar_text_omits_clock_when_disabled() {
        let mut config = BolootConfig::default();
        config.appearance.show_clock = false;
        config.prayer.enabled = false;
        let service = BolootService::new(config).unwrap();
        let text = service.top_bar_text().unwrap();
        let first = text.split(" · ").next().unwrap_or("");
        assert!(!first.contains(':'));
    }

    #[test]
    fn top_bar_text_includes_clock_when_enabled() {
        let mut config = BolootConfig::default();
        config.prayer.enabled = false;
        let service = BolootService::new(config).unwrap();
        let text = service.top_bar_text().unwrap();
        let first = text.split(" · ").next().unwrap_or("");
        assert!(first.contains(':'));
    }

    #[test]
    fn today_view_marks_weekend_and_holidays() {
        let service = BolootService::new(BolootConfig::default()).unwrap();
        let view = service.today_view().unwrap();
        let holidays = service.holidays_for_today().unwrap();
        assert_eq!(view.is_holiday, !holidays.is_empty());
        if view.is_holiday {
            assert!(!view.holiday_names.is_empty());
        }
    }

    #[test]
    fn view_for_friday_is_weekend_in_iran() {
        let service = BolootService::new(BolootConfig::default()).unwrap();
        let friday = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        let view = service.view_for(friday).unwrap();
        assert!(view.is_weekend);
    }

    #[test]
    fn apply_settings_applies_system_locale_when_enabled() {
        let mut config = BolootConfig::default();
        config.calendar.follow_system_locale = true;
        let mut service = BolootService::new(BolootConfig::default()).unwrap();
        service.apply_settings(config);
        let detected = crate::system_locale::detect_from_env();
        assert_eq!(service.config().calendar.country, detected.country);
        assert_eq!(service.config().calendar.language, detected.language);
        assert_eq!(service.config().calendar.timezone, detected.timezone);
    }

    #[test]
    fn top_bar_clock_uses_system_local_time() {
        let service = BolootService::new(BolootConfig::default()).unwrap();
        let text = service.top_bar_text().unwrap();
        let (hour, minute) = crate::system_time::local_time_of_day();
        let expected = crate::format::format_time(
            hour,
            minute,
            service.config().calendar.effective_numerals(),
        );
        assert!(
            text.starts_with(&expected),
            "top bar should start with system clock, got {text}"
        );
    }
}
