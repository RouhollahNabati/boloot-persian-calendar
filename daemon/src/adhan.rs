use boloot_cal_core::username_for_uid;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tracing::{debug, warn};

struct PlaybackState {
    uid: u32,
    username: String,
    path: PathBuf,
    child: Child,
}

static PLAYBACK: std::sync::OnceLock<Arc<Mutex<Option<PlaybackState>>>> = std::sync::OnceLock::new();

fn playback_slot() -> &'static Arc<Mutex<Option<PlaybackState>>> {
    PLAYBACK.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn spawn_as_user(username: &str, runtime: &str, program: &str, args: &[&str]) -> Option<Child> {
    let dbus_addr = format!("unix:path={runtime}/bus");
    Command::new("runuser")
        .arg("-u")
        .arg(username)
        .arg("--")
        .arg("env")
        .arg(format!("XDG_RUNTIME_DIR={runtime}"))
        .arg(format!("DBUS_SESSION_BUS_ADDRESS={dbus_addr}"))
        .arg("setsid")
        .arg(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn spawn_still_running(child: &mut Child) -> bool {
    thread::sleep(Duration::from_millis(150));
    match child.try_wait() {
        Ok(Some(_)) => false,
        Ok(None) => true,
        Err(_) => false,
    }
}

fn kill_playback_state(state: &mut PlaybackState) {
    let _ = state.child.kill();
    let _ = state.child.wait();
    let path = state.path.to_string_lossy();
    for player in ["pw-play", "paplay", "gst-play-1.0"] {
        let _ = Command::new("runuser")
            .arg("-u")
            .arg(&state.username)
            .arg("--")
            .arg("pkill")
            .arg("-x")
            .arg(player)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = Command::new("runuser")
            .arg("-u")
            .arg(&state.username)
            .arg("--")
            .arg("pkill")
            .arg("-f")
            .arg(format!("{player} {path}"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn child_still_running(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}

/// Returns true when adhan is playing for `uid`.
pub fn is_adhan_playing_for_uid(uid: u32) -> bool {
    let mut slot = playback_slot().lock().unwrap_or_else(|e| e.into_inner());
    let Some(state) = slot.as_mut() else {
        return false;
    };
    if state.uid != uid {
        return false;
    }
    if child_still_running(&mut state.child) {
        return true;
    }
    *slot = None;
    false
}

/// Stop adhan playback for `uid`. Returns true if playback was stopped.
pub fn stop_adhan_for_uid(uid: u32) -> bool {
    let mut slot = playback_slot().lock().unwrap_or_else(|e| e.into_inner());
    let Some(mut state) = slot.take() else {
        return false;
    };
    if state.uid != uid {
        *slot = Some(state);
        return false;
    }
    kill_playback_state(&mut state);
    debug!("adhan playback stopped for uid {uid}");
    true
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
    if let Some(state) = slot.as_mut() {
        if state.uid == uid && child_still_running(&mut state.child) {
            debug!("adhan already playing for uid {uid}, skipping");
            return false;
        }
        if child_still_running(&mut state.child) {
            let mut old = slot.take().unwrap();
            kill_playback_state(&mut old);
        } else {
            *slot = None;
        }
    }

    let volume_pct = volume.min(100);
    let paplay_volume = (volume_pct as u32) * 65536 / 100;
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return false,
    };

    let attempts: [(&str, Vec<String>); 3] = [
        ("pw-play", vec![path_str.to_string()]),
        (
            "gst-play-1.0",
            vec!["--no-interactive".into(), path_str.to_string()],
        ),
        (
            "paplay",
            vec![
                "--volume".into(),
                paplay_volume.to_string(),
                path_str.to_string(),
            ],
        ),
    ];

    for (player, args) in attempts {
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let Some(mut child) = spawn_as_user(&username, &runtime, player, &arg_refs) else {
            continue;
        };
        if !spawn_still_running(&mut child) {
            continue;
        }

        debug!("adhan playback started for uid {uid} via {player}: {}", path.display());
        *slot = Some(PlaybackState {
            uid,
            username,
            path: path.to_path_buf(),
            child,
        });
        return true;
    }

    warn!("no audio player available for adhan (uid {uid})");
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_slot_does_not_panic() {
        let _ = playback_slot();
    }

    #[test]
    fn stop_adhan_when_idle_returns_false() {
        let mut slot = playback_slot().lock().unwrap();
        *slot = None;
        drop(slot);
        assert!(!stop_adhan_for_uid(1000));
    }

    #[test]
    fn is_adhan_playing_when_idle_returns_false() {
        let mut slot = playback_slot().lock().unwrap();
        *slot = None;
        drop(slot);
        assert!(!is_adhan_playing_for_uid(1000));
    }
}
