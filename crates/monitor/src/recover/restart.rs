//! Restart a crashed process.

use core::Result;

use crate::programs::ProgramSpec;

/// Restart `spec` by spawning its binary. Stub: records intent via telemetry.
pub async fn restart_program(spec: &ProgramSpec, telemetry: &telemetry::Telemetry) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.recovery_started")
        .category("monitor")
        .message(format!("restarting {} ({})", spec.name, spec.binary))
        .emit();
    let mut cmd = tokio::process::Command::new(&spec.binary);
    cmd.args(&spec.args);
    cmd.spawn().map_err(|e| {
        core::SupervisorError::Monitor(format!("failed to restart {}: {e}", spec.name))
    })?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.restarted")
        .category("monitor")
        .message(format!("restarted {}", spec.name))
        .emit();
    Ok(())
}
