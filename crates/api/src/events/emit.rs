//! Emit API-level events to telemetry.

/// Emit a namespaced API event.
pub fn emit_api_event(
    telemetry: &telemetry::Telemetry,
    kind: &str,
    message: &str,
) {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind(format!("api.{kind}"))
        .category("api")
        .message(message.to_string())
        .emit();
}
