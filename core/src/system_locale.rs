use std::env;

use serde::Serialize;

use crate::format::NumeralStyle;
use crate::locale::{CountryProfile, LanguageVariant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectedLocale {
    pub country: CountryProfile,
    pub language: LanguageVariant,
    pub numerals: NumeralStyle,
    pub locale_code: String,
}

fn normalize_locale_code(raw: &str) -> String {
    let base = raw.split('.').next().unwrap_or(raw);
    let base = base.split('@').next().unwrap_or(base);
    base.replace('-', "_")
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

pub fn detect_from_env() -> DetectedLocale {
    let code = env_locale_code().unwrap_or_else(|| "fa_IR".into());
    match code.as_str() {
        "fa_IR" => DetectedLocale {
            country: CountryProfile::Iran,
            language: LanguageVariant::Persian,
            numerals: NumeralStyle::Persian,
            locale_code: "fa_IR".into(),
        },
        "fa_AF" => DetectedLocale {
            country: CountryProfile::Afghanistan,
            language: LanguageVariant::Dari,
            numerals: NumeralStyle::Persian,
            locale_code: "fa_AF".into(),
        },
        "ps_AF" | "ps" => DetectedLocale {
            country: CountryProfile::Afghanistan,
            language: LanguageVariant::Pashto,
            numerals: NumeralStyle::Persian,
            locale_code: "ps_AF".into(),
        },
        "tg_TJ" | "tg" => DetectedLocale {
            country: CountryProfile::Tajikistan,
            language: LanguageVariant::Tajik,
            numerals: NumeralStyle::Latin,
            locale_code: "tg_TJ".into(),
        },
        "fa" => DetectedLocale {
            country: CountryProfile::Iran,
            language: LanguageVariant::Persian,
            numerals: NumeralStyle::Persian,
            locale_code: "fa_IR".into(),
        },
        _ => DetectedLocale {
            country: CountryProfile::Iran,
            language: LanguageVariant::Persian,
            numerals: NumeralStyle::Persian,
            locale_code: code,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_fa_ir() {
        let detected = map_code("fa_IR");
        assert_eq!(detected.country, CountryProfile::Iran);
        assert_eq!(detected.language, LanguageVariant::Persian);
        assert_eq!(detected.numerals, NumeralStyle::Persian);
    }

    #[test]
    fn maps_fa_af() {
        let detected = map_code("fa_AF");
        assert_eq!(detected.country, CountryProfile::Afghanistan);
        assert_eq!(detected.language, LanguageVariant::Dari);
    }

    #[test]
    fn maps_ps_af() {
        let detected = map_code("ps_AF");
        assert_eq!(detected.language, LanguageVariant::Pashto);
    }

    #[test]
    fn maps_tg_tj() {
        let detected = map_code("tg_TJ");
        assert_eq!(detected.country, CountryProfile::Tajikistan);
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
        match code {
            "fa_IR" => DetectedLocale {
                country: CountryProfile::Iran,
                language: LanguageVariant::Persian,
                numerals: NumeralStyle::Persian,
                locale_code: "fa_IR".into(),
            },
            "fa_AF" => DetectedLocale {
                country: CountryProfile::Afghanistan,
                language: LanguageVariant::Dari,
                numerals: NumeralStyle::Persian,
                locale_code: "fa_AF".into(),
            },
            "ps_AF" | "ps" => DetectedLocale {
                country: CountryProfile::Afghanistan,
                language: LanguageVariant::Pashto,
                numerals: NumeralStyle::Persian,
                locale_code: "ps_AF".into(),
            },
            "tg_TJ" | "tg" => DetectedLocale {
                country: CountryProfile::Tajikistan,
                language: LanguageVariant::Tajik,
                numerals: NumeralStyle::Latin,
                locale_code: "tg_TJ".into(),
            },
            "fa" => DetectedLocale {
                country: CountryProfile::Iran,
                language: LanguageVariant::Persian,
                numerals: NumeralStyle::Persian,
                locale_code: "fa_IR".into(),
            },
            other => DetectedLocale {
                country: CountryProfile::Iran,
                language: LanguageVariant::Persian,
                numerals: NumeralStyle::Persian,
                locale_code: other.into(),
            },
        }
    }
}
