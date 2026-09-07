//! Watch CPU and memory usage via `sysinfo`.

use shared::Result;
use serde::{Deserialize, Serialize};

/// Point-in-time resource snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub pid: Option<u32>,
    pub cpu_usage: f32,
    pub process_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
}

/// Read resource usage, scoped to `pid` when given, else host-wide.
pub fn read_usage(pid: Option<u32>, telemetry: &telemetry::Telemetry) -> Result<ResourceUsage> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    let (cpu_usage, process_memory_bytes) = match pid {
        Some(p) => match sys.process(sysinfo::Pid::from_u32(p)) {
            Some(proc) => (proc.cpu_usage(), proc.memory()),
            None => (0.0, 0),
        },
        None => (sys.global_cpu_usage(), 0),
    };
    let usage = ResourceUsage {
        pid,
        cpu_usage,
        process_memory_bytes,
        total_memory_bytes: sys.total_memory(),
        used_memory_bytes: sys.used_memory(),
    };
    telemetry
        .builder()
        .level(telemetry::Level::Debug)
        .kind("monitor.resources")
        .category("monitor")
        .message("resource sample collected")
        .data("cpu_usage", usage.cpu_usage)
        .data("used_memory_bytes", usage.used_memory_bytes)
        .emit();
    Ok(usage)
}
