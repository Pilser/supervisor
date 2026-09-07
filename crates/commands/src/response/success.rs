//! Send success responses.

use crate::executor::result::CommandResult;

/// Envelope for a successful command result.
pub fn success_response(id: &str, result: &CommandResult) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "status": "ok",
        "message": result.message,
        "data": result.data,
    })
}
