//! Uptime, restart count, crash count.

use core::Result;
use serde::{Deserialize, Serialize};

/// Aggregate monitor statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStats {
    pub started_unix_millis: u64,
    pub restarts: u64,
    pub crashes: u64,
}

impl Default for MonitorStats {
    fn default() -> Self {
        Self {
            started_unix_millis: core::now_unix_millis(),
            restarts: 0,
            crashes: 0,
        }
    }
}

impl MonitorStats {
    pub fn record_restart(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        self.restarts += 1;
        telemetry
            .builder()
            .level(telemetry::Level::Info)
            .kind("monitor.stats")
            .category("monitor")
            .message(format!("restart recorded total={}", self.restarts))
            .emit();
        Ok(())
    }

    pub fn record_crash(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        self.crashes += 1;
        telemetry
            .builder()
            .level(telemetry::Level::Error)
            .kind("monitor.stats")
            .category("monitor")
            .message(format!("crash recorded total={}", self.crashes))
            .emit();
        Ok(())
    }

    pub fn uptime_secs(&self) -> u64 {
        core::now_unix_millis().saturating_sub(self.started_unix_millis) / 1000
    }
}
