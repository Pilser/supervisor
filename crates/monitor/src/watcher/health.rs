//! Health check via liveness probe.

use core::Result;
use serde::{Deserialize, Serialize};

use super::process::is_running;

/// Result of a single health probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Alive,
    Unknown,
    Unresponsive,
}

/// Probe a watched process; unresponsive when the PID is gone.
pub fn health_check(pid: u32, name: &str, telemetry: &telemetry::Telemetry) -> Result<HealthStatus> {
    let status = if is_running(pid) {
        HealthStatus::Alive
    } else {
        HealthStatus::Unresponsive
    };
    if status != HealthStatus::Alive {
        telemetry
            .builder()
            .level(telemetry::Level::Warn)
            .kind("monitor.health_failed")
            .category("monitor")
            .message(format!("health check failed for {name} pid={pid}"))
            .emit();
    }
    Ok(status)
}
