//! Heartbeat send history.

use serde::{Deserialize, Serialize};

/// Counts plus a bounded log of recent heartbeat outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatHistory {
    pub sent: u64,
    pub failed: u64,
    pub entries: Vec<String>,
    pub capacity: usize,
}

impl HeartbeatHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            sent: 0,
            failed: 0,
            entries: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    fn push(&mut self, entry: String) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn record_sent(&mut self) {
        self.sent += 1;
        self.push("sent".to_string());
    }

    pub fn record_failed(&mut self) {
        self.failed += 1;
        self.push("failed".to_string());
    }
}
