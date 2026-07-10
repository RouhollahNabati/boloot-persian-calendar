use chrono::{DateTime, Local, NaiveDate, Timelike};
use std::path::Path;

/// Current local calendar date from the system clock.
pub fn local_today() -> NaiveDate {
    Local::now().date_naive()
}

/// Current local date-time from the system clock.
pub fn local_now() -> DateTime<Local> {
    Local::now()
}

/// Local wall-clock hour and minute from the system clock.
pub fn local_time_of_day() -> (u32, u32) {
    let now = Local::now();
    (now.hour(), now.minute())
}

/// IANA timezone name from the system (`TZ` or `/etc/localtime`).
pub fn detect_system_timezone() -> Option<String> {
    if let Ok(tz) = std::env::var("TZ") {
        let trimmed = tz.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    timezone_from_localtime_link("/etc/localtime")
        .or_else(|| timezone_from_localtime_link("/var/db/zoneinfo/localtime"))
}

fn timezone_from_localtime_link(path: &str) -> Option<String> {
    let target = std::fs::read_link(path).ok()?;
    timezone_from_zoneinfo_path(&target)
}

fn timezone_from_zoneinfo_path(path: &Path) -> Option<String> {
    let raw = path.to_string_lossy();
    let marker = "zoneinfo/";
    let idx = raw.find(marker)?;
    let name = raw[idx + marker.len()..].trim_start_matches('/');
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zoneinfo_symlink_path() {
        assert_eq!(
            timezone_from_zoneinfo_path(Path::new("/usr/share/zoneinfo/Asia/Tehran")),
            Some("Asia/Tehran".into())
        );
        assert_eq!(
            timezone_from_zoneinfo_path(Path::new(
                "/usr/share/zoneinfo/America/New_York"
            )),
            Some("America/New_York".into())
        );
    }

    #[test]
    fn local_today_matches_local_now_date() {
        assert_eq!(local_today(), Local::now().date_naive());
    }
}
