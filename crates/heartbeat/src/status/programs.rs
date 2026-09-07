//! Program status section of the heartbeat.

/// Render `(name, running)` pairs as JSON for the heartbeat payload.
pub fn programs_status(entries: &[(String, bool)]) -> serde_json::Value {
    let programs: Vec<serde_json::Value> = entries
        .iter()
        .map(|(name, running)| serde_json::json!({ "name": name, "running": running }))
        .collect();
    serde_json::json!({ "programs": programs })
}
