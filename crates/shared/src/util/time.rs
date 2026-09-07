//! Time helpers producing RFC3339 strings with nanosecond precision.

use chrono::{DateTime, SecondsFormat, Utc};

/// Current UTC time as RFC3339 with nanoseconds.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true)
}

fn to_millis(ms: i64) -> u64 {
    u64::try_from(ms).unwrap_or(0)
}

/// Current unix time in milliseconds.
pub fn now_unix_millis() -> u64 {
    to_millis(Utc::now().timestamp_millis())
}

/// Parse an RFC3339 timestamp into unix milliseconds.
pub fn timestamp_to_unix_millis(ts: &str) -> Option<u64> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|d| to_millis(d.with_timezone(&Utc).timestamp_millis()))
}