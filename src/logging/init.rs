//! Initialize process logging.

/// Log severity used by the supervisor bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

// Reference every variant so the enum does not trigger dead-code warnings
// while only `Info` is constructed by the current bootstrap path.
#[allow(dead_code)]
const _ALL_LEVELS: [LogLevel; 4] = [
    LogLevel::Error,
    LogLevel::Warn,
    LogLevel::Info,
    LogLevel::Debug,
];

/// Initialize logging. Stub: structured logger wiring lands with the logging
/// backend; bootstrap messages go to stderr.
pub fn init_logging() {
    eprintln!("[supervisor] logging initialized");
}

/// Emit one bootstrap log line.
pub fn log(level: LogLevel, message: &str) {
    let tag = match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARN",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
    };
    eprintln!("[supervisor:{tag}] {message}");
}
