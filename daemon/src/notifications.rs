use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use boloot_cal_core::{
    active_session_uids, is_adhan_enabled_for, local_time_of_day, local_today,
    resolve_adhan_path, should_trigger_adhan, username_for_uid, BolootService, UiStrings, APP_NAME,
};
use chrono::Datelike;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{debug, warn};

use crate::adhan;
use crate::registry::ServiceRegistry;

pub fn spawn_notification_loop(registry: Arc<ServiceRegistry>) {
    tokio::spawn(async move {
        let mut sent: HashMap<u32, HashSet<String>> = HashMap::new();
        let mut last_day: u32 = 0;
        let mut poll_secs: u64 = 60;
        let mut ticker = interval(Duration::from_secs(poll_secs));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            let day = local_today().ordinal();
            if day != last_day {
                sent.clear();
                last_day = day;
            }

            let mut min_poll = poll_secs;
            for uid in active_session_uids() {
                let Ok(service) = registry.get(uid).await else {
                    continue;
                };
                let svc = service.read().await;
                let user_sent = sent.entry(uid).or_default();
                check_prayer_reminders(uid, &svc, user_sent);
                check_adhan(uid, &svc, user_sent, poll_secs);
                check_holiday_notifications(uid, &svc, user_sent);
                min_poll = min_poll.min(adaptive_poll_interval(&svc));
            }

            if min_poll != poll_secs {
                poll_secs = min_poll;
                ticker = interval(Duration::from_secs(poll_secs));
                ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            }
        }
    });
}

fn adaptive_poll_interval(service: &BolootService) -> u64 {
    let config = service.config();
    if !config.prayer.enabled || !config.prayer.adhan_enabled {
        return 60;
    }
    let Ok(schedule) = service.prayer_today() else {
        return 60;
    };
    let min_secs = schedule
        .times
        .min_seconds_until_enabled(|p| is_adhan_enabled_for(p, config));
    match min_secs {
        Some(s) if s <= 120 => 1,
        _ => 60,
    }
}

fn should_notify_prayer(remaining_seconds: i64, threshold_seconds: i64) -> bool {
    remaining_seconds <= threshold_seconds
}

fn check_prayer_reminders(uid: u32, service: &BolootService, sent: &mut HashSet<String>) {
    let config = service.config();
    if !config.prayer.enabled || config.prayer.notification_minutes.is_empty() {
        return;
    }

    let Ok(schedule) = service.prayer_today() else {
        return;
    };
    let Some(next) = schedule.next.as_ref() else {
        return;
    };

    for minutes in &config.prayer.notification_minutes {
        let threshold = (*minutes as i64) * 60;
        if should_notify_prayer(next.remaining_seconds, threshold) {
            let key = format!("prayer:{}:{}", next.label, minutes);
            if sent.insert(key) {
                let ui = UiStrings::for_language(config.calendar.language);
                let body = ui.prayer_notification_body(&next.label, &next.time);
                let summary = ui.prayer_notification_summary(&next.label, *minutes);
                send_notification(uid, APP_NAME, &summary, &body);
            }
        }
    }
}

fn check_adhan(uid: u32, service: &BolootService, sent: &mut HashSet<String>, poll_interval_secs: u64) {
    let config = service.config();
    if !config.prayer.enabled || !config.prayer.adhan_enabled {
        return;
    }

    let Ok(schedule) = service.prayer_today() else {
        return;
    };

    let day = local_today().ordinal();
    let path = match resolve_adhan_path(config) {
        Ok(p) => p,
        Err(e) => {
            warn!("adhan path resolution failed: {e}");
            return;
        }
    };

    for entry in &schedule.times.entries {
        if !is_adhan_enabled_for(entry.name, config) {
            continue;
        }
        let Some(delta) = schedule.times.seconds_until_entry(entry) else {
            continue;
        };
        if !should_trigger_adhan(delta, poll_interval_secs) {
            continue;
        }

        let key = format!("adhan:{:?}:{day}", entry.name);
        if sent.contains(&key) {
            continue;
        }

        if !adhan::play_adhan_for_uid(uid, &path, config.prayer.adhan_volume) {
            continue;
        }
        sent.insert(key);
    }
}

fn should_send_holiday_notification() -> bool {
    let hour = local_time_of_day().0;
    (17..=21).contains(&hour)
}

fn check_holiday_notifications(uid: u32, service: &BolootService, sent: &mut HashSet<String>) {
    if !service.config().calendar.holiday_notifications {
        return;
    }
    if !should_send_holiday_notification() {
        return;
    }

    let Ok(tomorrow) = service.holidays_tomorrow() else {
        return;
    };
    if tomorrow.is_empty() {
        return;
    }

    let key = format!(
        "holiday:{}",
        tomorrow
            .iter()
            .map(|h| h.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    if sent.insert(key) {
        let ui = UiStrings::for_language(service.config().calendar.language);
        let names = tomorrow
            .iter()
            .map(|h| h.name.as_str())
            .collect::<Vec<_>>()
            .join("، ");
        send_notification(
            uid,
            APP_NAME,
            &ui.holiday_notification_title,
            &ui.holiday_notification_body(&names),
        );
    }
}

fn user_runtime_dir(uid: u32) -> Option<String> {
    let dir = format!("/run/user/{uid}");
    if Path::new(&dir).is_dir() {
        Some(dir)
    } else {
        None
    }
}

fn notifications_suppressed_for_uid(uid: u32) -> bool {
    let Some(username) = username_for_uid(uid) else {
        return true;
    };
    let Some(runtime) = user_runtime_dir(uid) else {
        return true;
    };

    if gsettings_bool_for_user(&username, &runtime, "org.gnome.desktop.notifications", "disable-notifications")
    {
        return true;
    }
    if gsettings_is_false_for_user(
        &username,
        &runtime,
        "org.gnome.desktop.notifications",
        "show-banners",
    ) {
        return true;
    }
    if kreadconfig_bool_for_user(&username, &runtime, "knotificationsrc", "DoNotDisturb", "Enabled") {
        return true;
    }
    false
}

fn run_as_user(uid: u32, args: &[&str]) -> Option<std::process::Output> {
    let username = username_for_uid(uid)?;
    let runtime = user_runtime_dir(uid)?;
    let dbus_addr = format!("unix:path={runtime}/bus");
    Command::new("runuser")
        .arg("-u")
        .arg(&username)
        .arg("--")
        .arg("env")
        .arg(format!("XDG_RUNTIME_DIR={runtime}"))
        .arg(format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"))
        .args(args)
        .output()
        .ok()
}

fn gsettings_bool_for_user(username: &str, runtime: &str, schema: &str, key: &str) -> bool {
    let dbus_addr = format!("unix:path={runtime}/bus");
    Command::new("runuser")
        .args([
            "-u",
            username,
            "--",
            "env",
            &format!("XDG_RUNTIME_DIR={runtime}"),
            &format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"),
            "gsettings",
            "get",
            schema,
            key,
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "true")
        .unwrap_or(false)
}

fn gsettings_is_false_for_user(username: &str, runtime: &str, schema: &str, key: &str) -> bool {
    let dbus_addr = format!("unix:path={runtime}/bus");
    Command::new("runuser")
        .args([
            "-u",
            username,
            "--",
            "env",
            &format!("XDG_RUNTIME_DIR={runtime}"),
            &format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"),
            "gsettings",
            "get",
            schema,
            key,
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "false")
        .unwrap_or(false)
}

fn kreadconfig_bool_for_user(
    username: &str,
    runtime: &str,
    file: &str,
    group: &str,
    key: &str,
) -> bool {
    let dbus_addr = format!("unix:path={runtime}/bus");
    Command::new("runuser")
        .args([
            "-u",
            username,
            "--",
            "env",
            &format!("XDG_RUNTIME_DIR={runtime}"),
            &format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"),
            "kreadconfig5",
            "--file",
            file,
            "--group",
            group,
            "--key",
            key,
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "true")
        .unwrap_or(false)
}

fn send_notification(uid: u32, app: &str, summary: &str, body: &str) {
    if notifications_suppressed_for_uid(uid) {
        debug!("notification suppressed (DND) for uid {uid}: {summary}");
        return;
    }

    let result = run_as_user(
        uid,
        &[
            "notify-send",
            "-a",
            app,
            "-i",
            "preferences-system-time",
            summary,
            body,
        ],
    );

    match result {
        Some(output) if output.status.success() => {
            debug!("notification sent for uid {uid}: {summary}");
        }
        Some(output) => {
            warn!(
                "notify-send failed for uid {uid}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        None => {
            debug!("notify-send unavailable for uid {uid}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boloot_cal_core::{AdhanPrayerToggles, BolootConfig, PrayerName};

    #[test]
    fn prayer_notification_fires_when_remaining_crosses_threshold() {
        assert!(should_notify_prayer(540, 600));
        assert!(should_notify_prayer(600, 600));
        assert!(!should_notify_prayer(601, 600));
    }

    #[test]
    fn dnd_check_does_not_panic_without_desktop() {
        let _ = notifications_suppressed_for_uid(999_999);
    }

    #[test]
    fn adhan_independent_of_empty_notification_minutes() {
        let config = BolootConfig::default();
        assert!(!config.prayer.notification_minutes.is_empty() || config.prayer.adhan_enabled);
        let mut cfg = BolootConfig::default();
        cfg.prayer.notification_minutes.clear();
        cfg.prayer.adhan_enabled = true;
        assert!(cfg.prayer.adhan_enabled);
        assert!(cfg.prayer.notification_minutes.is_empty());
    }

    #[test]
    fn adhan_trigger_uses_entry_delta_not_next() {
        assert!(should_trigger_adhan(0, 1));
        assert!(should_trigger_adhan(1, 60));
        assert!(!should_trigger_adhan(61, 60));
    }

    #[test]
    fn default_adhan_prayers_excludes_sunrise() {
        let toggles = AdhanPrayerToggles::default();
        assert!(!toggles.is_enabled(PrayerName::Sunrise));
        assert!(toggles.is_enabled(PrayerName::Fajr));
    }
}
