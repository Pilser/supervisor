//! Detect unresponsive (frozen) processes via last-seen timestamps.

use std::collections::HashMap;

use core::Result;
use serde::{Deserialize, Serialize};

/// Tracks last-seen pings per PID; stale entries count as frozen.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FreezeDetector {
    pub timeout_secs: u64,
    pub last_seen: HashMap<u32, u64>,
}

impl FreezeDetector {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout_secs: timeout_secs.max(1),
            last_seen: HashMap::new(),
        }
    }

    /// Record a successful ping/response from `pid`.
    pub fn mark_seen(&mut self, pid: u32) {
        self.last_seen.insert(pid, core::now_unix_millis());
    }

    /// True when `pid` has not been seen within the timeout.
    pub fn is_frozen(&self, pid: u32, telemetry: &telemetry::Telemetry) -> Result<bool> {
        let now = core::now_unix_millis();
        let frozen = match self.last_seen.get(&pid) {
            Some(seen) => now - seen > self.timeout_secs * 1000,
            None => false,
        };
        if frozen {
            telemetry
                .builder()
                .level(telemetry::Level::Warn)
                .kind("monitor.freeze")
                .category("monitor")
                .message(format!("process pid={pid} appears frozen"))
                .emit();
        }
        Ok(frozen)
    }
}
