use std::path::PathBuf;

use crate::config::{AdhanPreset, BolootConfig};
use crate::error::{BolootError, Result};
use crate::holidays::data_dir;
use crate::prayer::PrayerName;

pub fn sounds_dir() -> PathBuf {
    data_dir().join("sounds")
}

pub fn preset_filename(preset: AdhanPreset) -> &'static str {
    match preset {
        AdhanPreset::Mansouri => "mansouri.ogg",
        AdhanPreset::Makkah => "makkah.ogg",
        AdhanPreset::Madinah => "madinah.ogg",
        AdhanPreset::Custom => "",
    }
}

pub fn resolve_adhan_path(config: &BolootConfig) -> Result<PathBuf> {
    let prayer = &config.prayer;
    match prayer.adhan_preset {
        AdhanPreset::Custom => {
            let path = prayer
                .adhan_custom_path
                .as_deref()
                .filter(|p| !p.is_empty())
                .ok_or_else(|| {
                    BolootError::InvalidConfig("adhan_custom_path required for custom preset".into())
                })?;
            let path = PathBuf::from(path);
            if !path.is_file() {
                return Err(BolootError::InvalidConfig(format!(
                    "adhan file not found: {path:?}"
                )));
            }
            Ok(path)
        }
        preset => {
            let path = sounds_dir().join(preset_filename(preset));
            if !path.is_file() {
                return Err(BolootError::InvalidConfig(format!(
                    "bundled adhan file not found: {path:?}"
                )));
            }
            Ok(path)
        }
    }
}

pub fn is_adhan_enabled_for(prayer: PrayerName, config: &BolootConfig) -> bool {
    config.prayer.adhan_enabled && config.prayer.adhan_prayers.is_enabled(prayer)
}

/// Returns true when `delta_seconds` (prayer_time - now) is within the trigger window.
pub fn should_trigger_adhan(delta_seconds: i64, poll_interval_secs: u64) -> bool {
    let window = poll_interval_secs as i64;
    (0..=window).contains(&delta_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AdhanPrayerToggles;

    #[test]
    fn should_trigger_within_poll_window() {
        assert!(should_trigger_adhan(0, 1));
        assert!(should_trigger_adhan(1, 1));
        assert!(!should_trigger_adhan(2, 1));
        assert!(!should_trigger_adhan(-1, 1));
    }

    #[test]
    fn prayer_toggles_default_excludes_sunrise() {
        let toggles = AdhanPrayerToggles::default();
        assert!(toggles.is_enabled(PrayerName::Fajr));
        assert!(!toggles.is_enabled(PrayerName::Sunrise));
        assert!(toggles.is_enabled(PrayerName::Dhuhr));
    }

    #[test]
    fn preset_filenames() {
        assert_eq!(preset_filename(AdhanPreset::Mansouri), "mansouri.ogg");
        assert_eq!(preset_filename(AdhanPreset::Makkah), "makkah.ogg");
        assert_eq!(preset_filename(AdhanPreset::Madinah), "madinah.ogg");
    }
}
