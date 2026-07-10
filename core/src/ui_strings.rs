use serde::{Deserialize, Serialize};

use crate::locale::LanguageVariant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStrings {
    pub holidays_prefix: String,
    pub prayer_section_title: String,
    pub next_prayer_prefix: String,
    pub today_button: String,
    pub settings_button: String,
    pub holiday_notification_title: String,
    pub holiday_notification_body_prefix: String,
    pub prev_month_label: String,
    pub next_month_label: String,
    pub prev_year_label: String,
    pub next_year_label: String,
    pub adhan_stop_label: String,
    pub adhan_playing_label: String,
}

impl UiStrings {
    pub fn for_language(language: LanguageVariant) -> Self {
        match language {
            LanguageVariant::English => english(),
            LanguageVariant::Persian => persian(),
            LanguageVariant::Dari => dari(),
            LanguageVariant::Pashto => pashto(),
            LanguageVariant::Tajik => tajik(),
        }
    }

    pub fn apply_english_nav_labels(&mut self) {
        self.prev_month_label = "Previous month".into();
        self.next_month_label = "Next month".into();
        self.prev_year_label = "Previous year".into();
        self.next_year_label = "Next year".into();
    }

    pub fn prayer_notification_summary(&self, label: &str, minutes: u32) -> String {
        match self.prayer_section_title.as_str() {
            "Prayer times" => format!("Adhan {label} in {minutes} minutes"),
            "Вақти намоз" => format!("Азон {label} дар {minutes} дақиқа"),
            "د لمانځه وختونه" => format!("اذان {label} تر {minutes} دقیقو"),
            _ => format!("اذان {label} تا {minutes} دقیقه دیگر"),
        }
    }

    pub fn prayer_notification_body(&self, label: &str, time: &str) -> String {
        match self.prayer_section_title.as_str() {
            "Prayer times" => format!("{label} at {time}"),
            "Вақти намоз" => format!("{label} соат {time}"),
            "د لمانځه وختونه" => format!("{label} وخت {time}"),
            _ => format!("{label} ساعت {time}"),
        }
    }

    pub fn prayer_adhan_summary(&self, label: &str) -> String {
        match self.prayer_section_title.as_str() {
            "Prayer times" => format!("{label} time"),
            "Вақти намоз" => format!("Вақти {label}"),
            "د لمانځه وختونه" => format!("د {label} وخت"),
            _ => format!("وقت {label}"),
        }
    }

    pub fn prayer_adhan_body(&self, label: &str) -> String {
        match self.prayer_section_title.as_str() {
            "Prayer times" => format!("It is now time for {label} prayer"),
            "Вақти намоз" => format!("Акнун вақти намози {label}"),
            "د لمانځه وختونه" => format!("اوس د {label} لمانځه وخت دی"),
            _ => format!("اکنون وقت نماز {label}"),
        }
    }

    pub fn holiday_notification_body(&self, names: &str) -> String {
        format!("{} {names}", self.holiday_notification_body_prefix)
    }
}

fn persian() -> UiStrings {
    UiStrings {
        holidays_prefix: "تعطیلات:".into(),
        prayer_section_title: "اوقات شرعی".into(),
        next_prayer_prefix: "بعدی:".into(),
        today_button: "امروز".into(),
        settings_button: "تنظیمات بولوت".into(),
        holiday_notification_title: "تعطیلی فردا".into(),
        holiday_notification_body_prefix: "فردا:".into(),
        prev_month_label: "ماه قبل".into(),
        next_month_label: "ماه بعد".into(),
        prev_year_label: "سال قبل".into(),
        next_year_label: "سال بعد".into(),
        adhan_stop_label: "قطع اذان".into(),
        adhan_playing_label: "در حال پخش اذان".into(),
    }
}

fn dari() -> UiStrings {
    UiStrings {
        holidays_prefix: "رخصتی‌ها:".into(),
        prayer_section_title: "اوقات شرعی".into(),
        next_prayer_prefix: "بعدی:".into(),
        today_button: "امروز".into(),
        settings_button: "تنظیمات بولوت".into(),
        holiday_notification_title: "رخصتی فردا".into(),
        holiday_notification_body_prefix: "فردا:".into(),
        prev_month_label: "ماه قبل".into(),
        next_month_label: "ماه بعد".into(),
        prev_year_label: "سال قبل".into(),
        next_year_label: "سال بعد".into(),
        adhan_stop_label: "قطع اذان".into(),
        adhan_playing_label: "در حال پخش اذان".into(),
    }
}

fn pashto() -> UiStrings {
    UiStrings {
        holidays_prefix: "رخصتيانې:".into(),
        prayer_section_title: "د لمانځه وختونه".into(),
        next_prayer_prefix: "راتلونکی:".into(),
        today_button: "نن".into(),
        settings_button: "د بولوت تنظیمات".into(),
        holiday_notification_title: "سبا رخصتي".into(),
        holiday_notification_body_prefix: "سبا:".into(),
        prev_month_label: "تیر ماه".into(),
        next_month_label: "راتلونکی ماه".into(),
        prev_year_label: "تیر کال".into(),
        next_year_label: "راتلونکی کال".into(),
        adhan_stop_label: "اذان بند کړئ".into(),
        adhan_playing_label: "اذان غږېږي".into(),
    }
}

fn english() -> UiStrings {
    UiStrings {
        holidays_prefix: "Holidays:".into(),
        prayer_section_title: "Prayer times".into(),
        next_prayer_prefix: "Next:".into(),
        today_button: "Today".into(),
        settings_button: "BOLOOT Settings".into(),
        holiday_notification_title: "Holiday tomorrow".into(),
        holiday_notification_body_prefix: "Tomorrow:".into(),
        prev_month_label: "Previous month".into(),
        next_month_label: "Next month".into(),
        prev_year_label: "Previous year".into(),
        next_year_label: "Next year".into(),
        adhan_stop_label: "Stop adhan".into(),
        adhan_playing_label: "Playing adhan".into(),
    }
}

fn tajik() -> UiStrings {
    UiStrings {
        holidays_prefix: "Идҳо:".into(),
        prayer_section_title: "Вақти намоз".into(),
        next_prayer_prefix: "Бад:".into(),
        today_button: "Имрӯз".into(),
        settings_button: "Танзимоти BOLOOT".into(),
        holiday_notification_title: "Ид пагоҳ".into(),
        holiday_notification_body_prefix: "Пагоҳ:".into(),
        prev_month_label: "Моҳи қаблӣ".into(),
        next_month_label: "Моҳи баъдӣ".into(),
        prev_year_label: "Соли қаблӣ".into(),
        next_year_label: "Соли баъдӣ".into(),
        adhan_stop_label: "Қатъ кардани азон".into(),
        adhan_playing_label: "Азон пахш мешавад".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_labels_localized_for_persian() {
        let ui = UiStrings::for_language(LanguageVariant::Persian);
        assert_eq!(ui.prev_month_label, "ماه قبل");
        assert_eq!(ui.next_year_label, "سال بعد");
    }

    #[test]
    fn english_uses_latin_nav_labels() {
        let ui = UiStrings::for_language(LanguageVariant::English);
        assert_eq!(ui.prev_month_label, "Previous month");
        assert_eq!(ui.today_button, "Today");
    }

    #[test]
    fn tajik_uses_cyrillic() {
        let ui = UiStrings::for_language(LanguageVariant::Tajik);
        assert!(ui.today_button.contains('И'));
    }

    #[test]
    fn prayer_notification_formats() {
        let ui = UiStrings::for_language(LanguageVariant::Persian);
        assert!(ui.prayer_notification_summary("ظهر", 10).contains("ظهر"));
    }
}
