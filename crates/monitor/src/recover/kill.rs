//! Kill a frozen process.

use shared::Result;

/// Kill `pid`. Returns true when a process was actually terminated.
pub fn kill_process(pid: u32, telemetry: &telemetry::Telemetry) -> Result<bool> {
    let mut sys = sysinfo::System::new_all();
    let killed = match sys.process(sysinfo::Pid::from_u32(pid)) {
        Some(proc) => proc.kill(),
        None => false,
    };
    let _ = &mut sys;
    telemetry
        .builder()
        .level(telemetry::Level::Warn)
        .kind("monitor.kill")
        .category("monitor")
        .message(format!("kill pid={pid} killed={killed}"))
        .emit();
    Ok(killed)
}
