//! CPU/memory section of the heartbeat.

use shared::Result;

/// Host-wide CPU and memory snapshot as JSON.
pub fn resources_status(telemetry: &telemetry::Telemetry) -> Result<serde_json::Value> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    let value = serde_json::json!({
        "cpu_usage": sys.global_cpu_usage(),
        "total_memory_bytes": sys.total_memory(),
        "used_memory_bytes": sys.used_memory(),
    });
    telemetry
        .builder()
        .level(telemetry::Level::Debug)
        .kind("heartbeat.resources")
        .category("heartbeat")
        .message("heartbeat resource section collected")
        .emit();
    Ok(value)
}
