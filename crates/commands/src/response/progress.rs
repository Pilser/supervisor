//! Send progress for long-running commands.

/// Envelope reporting 0–100 progress for a running command.
pub fn progress_response(id: &str, percent: u8) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "status": "progress",
        "percent": percent.min(100),
    })
}
