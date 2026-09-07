//! Monitoring events log.

use serde::{Deserialize, Serialize};

/// Bounded in-memory log of monitoring events.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EventLog {
    pub events: Vec<String>,
    pub capacity: usize,
}

impl EventLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, event: String) {
        if self.events.len() >= self.capacity {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn list(&self) -> &[String] {
        &self.events
    }
}
