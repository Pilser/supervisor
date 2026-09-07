//! VM uptime tracking.

use serde::{Deserialize, Serialize};

/// Uptime measured from tracker creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeTracker {
    pub boot_unix_millis: u64,
}

impl Default for UptimeTracker {
    fn default() -> Self {
        Self {
            boot_unix_millis: shared::now_unix_millis(),
        }
    }
}

impl UptimeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn uptime_secs(&self) -> u64 {
        shared::now_unix_millis().saturating_sub(self.boot_unix_millis) / 1000
    }
}
