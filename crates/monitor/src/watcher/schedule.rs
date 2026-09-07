//! Monitoring interval schedule.

use shared::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// How often the monitor polls watched processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSchedule {
    pub interval_secs: u64,
}

impl Default for MonitorSchedule {
    fn default() -> Self {
        Self { interval_secs: 5 }
    }
}

impl MonitorSchedule {
    pub fn from_interval_secs(interval_secs: u64) -> Result<Self> {
        if interval_secs == 0 {
            return Err(SupervisorError::Monitor("interval must be > 0".into()));
        }
        Ok(Self { interval_secs })
    }

    /// Sleep until the next poll is due.
    pub async fn tick(&self) {
        tokio::time::sleep(std::time::Duration::from_secs(self.interval_secs.max(1))).await;
    }
}
