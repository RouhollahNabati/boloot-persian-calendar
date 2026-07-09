use serde::Serialize;
use std::io::Write;

const LOG_PATH: &str = "/home/dispro/Projects/Boloot Calendar/.cursor/debug-0383bf.log";
const SESSION_ID: &str = "0383bf";

#[derive(Serialize)]
struct AgentLog<'a, T: Serialize> {
    sessionId: &'a str,
    runId: &'a str,
    hypothesisId: &'a str,
    location: &'a str,
    message: &'a str,
    data: T,
    timestamp: i64,
}

pub fn agent_log(
    hypothesis_id: &str,
    location: &str,
    message: &str,
    data: impl Serialize,
    run_id: &str,
) {
    let payload = AgentLog {
        sessionId: SESSION_ID,
        runId: run_id,
        hypothesisId: hypothesis_id,
        location,
        message,
        data,
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    if let Ok(line) = serde_json::to_string(&payload) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_PATH)
        {
            let _ = writeln!(file, "{line}");
        }
    }
}
