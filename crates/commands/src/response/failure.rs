//! Send failure responses.

/// Envelope for a failed command.
pub fn failure_response(id: &str, error: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "status": "error",
        "message": error,
    })
}
