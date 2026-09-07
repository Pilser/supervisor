//! Heartbeat interval configuration.

use core::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// Seconds between heartbeats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatInterval {
    pub seconds: u64,
}

impl Default for HeartbeatInterval {
    fn default() -> Self {
        Self { seconds: 5 }
    }
}

impl HeartbeatInterval {
    pub fn new(seconds: u64) -> Result<Self> {
        if seconds == 0 {
            return Err(SupervisorError::Heartbeat("interval must be > 0".into()));
        }
        Ok(Self { seconds })
    }

    /// Sleep until the next heartbeat is due.
    pub async fn wait(&self) {
        tokio::time::sleep(std::time::Duration::from_secs(self.seconds.max(1))).await;
    }
}
