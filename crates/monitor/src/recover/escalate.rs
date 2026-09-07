//! Escalate to the wrapper when local recovery fails.

use shared::Result;
use serde::{Deserialize, Serialize};

/// Escalation notice queued for the wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub program: String,
    pub reason: String,
    pub at_unix_millis: u64,
}

/// Report an unrecoverable program failure for wrapper-side handling.
pub fn escalate_to_wrapper(
    program: &str,
    reason: &str,
    telemetry: &telemetry::Telemetry,
) -> Result<Escalation> {
    let escalation = Escalation {
        program: program.to_string(),
        reason: reason.to_string(),
        at_unix_millis: shared::now_unix_millis(),
    };
    telemetry
        .builder()
        .level(telemetry::Level::Error)
        .kind("monitor.escalate")
        .category("monitor")
        .message(format!("escalating {program}: {reason}"))
        .emit();
    Ok(escalation)
}
