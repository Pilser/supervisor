//! Watch a process by PID.

use shared::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// A process currently under supervision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedProcess {
    pub pid: u32,
    pub name: String,
    pub started_unix_millis: u64,
}

/// Register a PID for monitoring.
pub fn watch_process(
    pid: u32,
    name: &str,
    telemetry: &telemetry::Telemetry,
) -> Result<WatchedProcess> {
    if pid == 0 {
        return Err(SupervisorError::Monitor("refusing to watch pid 0".into()));
    }
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.watch")
        .category("monitor")
        .message(format!("watching {name} pid={pid}"))
        .emit();
    Ok(WatchedProcess {
        pid,
        name: name.to_string(),
        started_unix_millis: shared::now_unix_millis(),
    })
}

/// True when the PID exists AND is not a zombie. Zombies keep their PID
/// in the table until the parent reaps them; without this check a dead
/// (unreaped) child reads as alive forever and is never restarted.
pub fn is_running(pid: u32) -> bool {
    let sys = sysinfo::System::new_all();
    match sys.process(sysinfo::Pid::from_u32(pid)) {
        Some(p) => p.status() != sysinfo::ProcessStatus::Zombie,
        None => false,
    }
}
