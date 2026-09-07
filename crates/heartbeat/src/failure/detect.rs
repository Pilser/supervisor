//! Detect heartbeat send failures.

/// True when consecutive failures reach the threshold.
pub fn detect_failure(
    consecutive_failures: u32,
    threshold: u32,
    telemetry: &telemetry::Telemetry,
) -> bool {
    let failed = consecutive_failures >= threshold.max(1);
    if failed {
        telemetry
            .builder()
            .level(telemetry::Level::Error)
            .kind("heartbeat.failed")
            .category("heartbeat")
            .message(format!("heartbeat failing: {consecutive_failures} consecutive errors"))
            .emit();
    }
    failed
}
