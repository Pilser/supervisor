//! Retry failed heartbeats with bounded attempts.

use serde::{Deserialize, Serialize};

/// Tracks consecutive failures against a retry budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    pub attempts: u32,
    pub max_attempts: u32,
}

impl RetryState {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            attempts: 0,
            max_attempts: max_attempts.max(1),
        }
    }

    pub fn should_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    pub fn record_failure(&mut self, telemetry: &telemetry::Telemetry) {
        self.attempts += 1;
        telemetry
            .builder()
            .level(telemetry::Level::Warn)
            .kind("heartbeat.retry")
            .category("heartbeat")
            .message(format!("heartbeat retry {}/{}", self.attempts, self.max_attempts))
            .emit();
    }

    pub fn record_success(&mut self) {
        self.attempts = 0;
    }
}
