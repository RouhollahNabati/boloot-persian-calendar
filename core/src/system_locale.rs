use std::env;

use serde::Serialize;

use crate::countries::{registry, CountryProfile};
use crate::format::NumeralStyle;
use crate::locale::{LanguageVariant, LocaleProfile};
use crate::system_time::detect_system_timezone;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectedLocale {
    pub country: CountryProfile,
    pub language: LanguageVariant,
    pub numerals: NumeralStyle,
    pub locale_code: String,
    pub timezone: String,
}

fn normalize_locale_code(raw: &str) -> String {
    let base = raw.split('.').next().unwrap_or(raw);
    let base = base.split('@').next().unwrap_or(base);
    base.replace('-', "_")
}

fn is_english_locale(code: &str) -> bool {
    let lower = code.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "en" | "en_us" | "en_gb" | "en_ca" | "en_au"
    ) || lower.starts_with("en_")
}

fn env_locale_code() -> Option<String> {
    for var in ["LC_TIME", "LC_ALL", "LANG"] {
        if let Ok(val) = env::var(var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() && trimmed != "C" && trimmed != "POSIX" {
                return Some(normalize_locale_code(trimmed));
            }
        }
    }
    None
}

fn country_from_locale_code(code: &str) -> CountryProfile {
    let reg = registry();
    if let Some((_lang, region)) = code.split_once('_') {
        if let Some(id) = reg.iso_to_id(region) {
            return CountryProfile::new(id);
        }
    }
    CountryProfile::iran()
}

fn language_from_locale_code(code: &str) -> LanguageVariant {
    match code {
        "fa_IR" | "fa" => LanguageVariant::Persian,
        "fa_AF" => LanguageVariant::Dari,
        "ps_AF" | "ps" => LanguageVariant::Pashto,
        "tg_TJ" | "tg" => LanguageVariant::Tajik,
        other if is_english_locale(other) => LanguageVariant::English,
        _ => LanguageVariant::English,
    }
}

fn map_locale_code(code: &str) -> DetectedLocale {
    let mut detected = match code {
        "fa_IR" => DetectedLocale {
            country: CountryProfile::iran(),
            language: LanguageVariant::Persian,
            numerals: NumeralStyle::Persian,
            locale_code: "fa_IR".into(),
            timezone: String::new(),
        },
        "fa_AF" => DetectedLocale {
            country: CountryProfile::afghanistan(),
            language: LanguageVariant::Dari,
            numerals: NumeralStyle::Persian,
            locale_code: "fa_AF".into(),
            timezone: String::new(),
        },
        "ps_AF" | "ps" => DetectedLocale {
            country: CountryProfile::afghanistan(),
            language: LanguageVariant::Pashto,
            numerals: NumeralStyle::Persian,
            locale_code: "ps_AF".into(),
            timezone: String::new(),
        },
        "tg_TJ" | "tg" => DetectedLocale {
            country: CountryProfile::tajikistan(),
            language: LanguageVariant::Tajik,
            numerals: NumeralStyle::Latin,
            locale_code: "tg_TJ".into(),
            timezone: String::new(),
        },
        "fa" => DetectedLocale {
            country: CountryProfile::iran(),
            language: LanguageVariant::Persian,
            numerals: NumeralStyle::Persian,
            locale_code: "fa_IR".into(),
            timezone: String::new(),
        },
        other if is_english_locale(other) => DetectedLocale {
            country: country_from_locale_code(other),
            language: LanguageVariant::English,
            numerals: NumeralStyle::Latin,
            locale_code: other.into(),
            timezone: String::new(),
        },
        other if other.contains('_') => DetectedLocale {
            country: country_from_locale_code(other),
            language: language_from_locale_code(other),
            numerals: NumeralStyle::Latin,
            locale_code: other.into(),
            timezone: String::new(),
        },
        other => DetectedLocale {
            country: CountryProfile::iran(),
            language: LanguageVariant::Persian,
            numerals: NumeralStyle::Persian,
            locale_code: other.into(),
            timezone: String::new(),
        },
    };
    detected.timezone = detect_system_timezone().unwrap_or_else(|| {
        LocaleProfile::resolve(detected.country.clone(), detected.language).default_timezone
    });
    detected
}

pub fn detect_from_env() -> DetectedLocale {
    let code = env_locale_code().unwrap_or_else(|| "fa_IR".into());
    map_locale_code(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_fa_ir() {
        let detected = map_code("fa_IR");
        assert_eq!(detected.country, CountryProfile::iran());
        assert_eq!(detected.language, LanguageVariant::Persian);
        assert_eq!(detected.numerals, NumeralStyle::Persian);
        assert!(!detected.timezone.is_empty());
    }

    #[test]
    fn maps_fa_af() {
        let detected = map_code("fa_AF");
        assert_eq!(detected.country, CountryProfile::afghanistan());
        assert_eq!(detected.language, LanguageVariant::Dari);
    }

    #[test]
    fn maps_ps_af() {
        let detected = map_code("ps_AF");
        assert_eq!(detected.language, LanguageVariant::Pashto);
    }

    #[test]
    fn maps_en_us() {
        let detected = map_code("en_US");
        assert_eq!(detected.language, LanguageVariant::English);
        assert_eq!(detected.numerals, NumeralStyle::Latin);
        if registry().get("usa").is_some() {
            assert_eq!(detected.country.as_str(), "usa");
        }
    }

    #[test]
    fn detect_from_env_en_us() {
        unsafe {
            std::env::set_var("LANG", "en_US.UTF-8");
            std::env::remove_var("LC_ALL");
            std::env::remove_var("LC_TIME");
        }
        let detected = detect_from_env();
        assert_eq!(detected.language, LanguageVariant::English);
        assert_eq!(detected.locale_code, "en_US");
    }

    #[test]
    fn maps_en_us_lowercase() {
        let detected = map_code("en_us");
        assert_eq!(detected.language, LanguageVariant::English);
    }

    #[test]
    fn maps_tg_tj() {
        let detected = map_code("tg_TJ");
        assert_eq!(detected.country, CountryProfile::tajikistan());
        assert_eq!(detected.language, LanguageVariant::Tajik);
        assert_eq!(detected.numerals, NumeralStyle::Latin);
    }

    #[test]
    fn normalizes_dashed_locale() {
        assert_eq!(normalize_locale_code("fa-IR.UTF-8"), "fa_IR");
        assert_eq!(
            normalize_locale_code("fa_IR@calendar=persian"),
            "fa_IR"
        );
    }

    fn map_code(code: &str) -> DetectedLocale {
        map_locale_code(code)
    }
}
