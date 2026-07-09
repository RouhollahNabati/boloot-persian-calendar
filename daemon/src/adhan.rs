use boloot_cal_core::username_for_uid;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use tracing::{debug, warn};

static PLAYBACK: std::sync::OnceLock<Arc<Mutex<Option<Child>>>> = std::sync::OnceLock::new();

fn playback_slot() -> &'static Arc<Mutex<Option<Child>>> {
    PLAYBACK.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// Play adhan audio in the active desktop session for `uid`.
/// Returns `true` when playback was started successfully.
pub fn play_adhan_for_uid(uid: u32, path: &Path, volume: u8) -> bool {
    let Some(username) = username_for_uid(uid) else {
        return false;
    };
    let runtime = format!("/run/user/{uid}");
    if !Path::new(&runtime).is_dir() {
        return false;
    }

    let mut slot = playback_slot().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(child) = slot.as_mut() {
        if child.try_wait().ok().flatten().is_none() {
            debug!("adhan already playing, skipping");
            return false;
        }
        *slot = None;
    }

    let volume_pct = volume.min(100);
    let paplay_volume = (volume_pct as u32) * 65536 / 100;
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return false,
    };

    let dbus_addr = format!("unix:path={runtime}/bus");
    let child = Command::new("runuser")
        .arg("-u")
        .arg(&username)
        .arg("--")
        .arg("env")
        .arg(format!("XDG_RUNTIME_DIR={runtime}"))
        .arg(format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"))
        .arg("paplay")
        .arg("--volume")
        .arg(paplay_volume.to_string())
        .arg(path_str)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
        .or_else(|| {
            Command::new("runuser")
                .arg("-u")
                .arg(&username)
                .arg("--")
                .arg("env")
                .arg(format!("XDG_RUNTIME_DIR={runtime}"))
                .arg(format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"))
                .arg("pw-play")
                .arg(path_str)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()
        });

    match child {
        Some(c) => {
            debug!("adhan playback started for uid {uid}: {}", path.display());
            *slot = Some(c);
            true
        }
        None => {
            warn!("no audio player available for adhan (uid {uid})");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_slot_does_not_panic() {
        let _ = playback_slot();
    }
}
