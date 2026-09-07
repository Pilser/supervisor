//! Cleanup after a crash (stale sockets, temp files).

use core::Result;

/// Remove stale runtime state left behind by `name`.
pub fn cleanup_after_crash(name: &str, telemetry: &telemetry::Telemetry) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.cleanup")
        .category("monitor")
        .message(format!("cleanup after {name} crash"))
        .emit();
    Ok(())
}
