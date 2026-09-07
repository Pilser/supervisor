//! Detect process exit.

use shared::Result;

use crate::watcher::process::is_running;

/// True when the process has exited.
pub fn detect_crash(pid: u32, name: &str, telemetry: &telemetry::Telemetry) -> Result<bool> {
    let crashed = !is_running(pid);
    if crashed {
        telemetry
            .builder()
            .level(telemetry::Level::Error)
            .kind("monitor.crash")
            .category("monitor")
            .message(format!("process {name} pid={pid} exited"))
            .emit();
    }
    Ok(crashed)
}
