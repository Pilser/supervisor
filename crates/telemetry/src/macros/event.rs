//! `telemetry_event!` — inline event emission.

#[macro_export]
macro_rules! telemetry_event {
    ($telemetry:expr, $level:expr, $kind:expr, $message:expr) => {
        $crate::event::emit_event($telemetry, $level, $kind, $message)
    };
}