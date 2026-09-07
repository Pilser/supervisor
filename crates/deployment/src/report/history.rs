//! Deployment history log.

use serde::{Deserialize, Serialize};

/// Bounded log of past deployment outcomes.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeploymentHistory {
    pub entries: Vec<String>,
    pub capacity: usize,
}

impl DeploymentHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, entry: String) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn list(&self) -> &[String] {
        &self.entries
    }
}
