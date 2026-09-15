//! Restart a crashed process.

use shared::Result;

use crate::programs::ProgramSpec;

/// Restart `spec` by spawning its binary. Returns the child handle: the
/// caller must keep it (and poll `try_wait()`) — a dropped handle leaks a
/// zombie that defeats liveness checks.
pub async fn restart_program(
    spec: &ProgramSpec,
    telemetry: &telemetry::Telemetry,
) -> Result<tokio::process::Child> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.recovery_started")
        .category("monitor")
        .message(format!("restarting {} ({})", spec.name, spec.binary))
        .emit();
    let mut cmd = tokio::process::Command::new(&spec.binary);
    cmd.args(&spec.args);
    let child = cmd.spawn().map_err(|e| {
        shared::SupervisorError::Monitor(format!("failed to restart {}: {e}", spec.name))
    })?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.restarted")
        .category("monitor")
        .message(format!("restarted {}", spec.name))
        .emit();
    Ok(child)
}
