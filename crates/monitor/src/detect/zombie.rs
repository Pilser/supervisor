//! Detect zombie processes.

use shared::Result;

use crate::watcher::process::is_running;

/// True when the PID exists but no longer responds to health probes.
/// Stub: full zombie-state inspection is platform specific.
pub fn detect_zombie(pid: u32, telemetry: &telemetry::Telemetry) -> Result<bool> {
    let _ = is_running(pid);
    telemetry
        .builder()
        .level(telemetry::Level::Debug)
        .kind("monitor.zombie_check")
        .category("monitor")
        .message(format!("zombie check pid={pid}"))
        .emit();
    Ok(false)
}
