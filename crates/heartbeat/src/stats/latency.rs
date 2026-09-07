//! Round-trip latency tracking.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// Rolling window of heartbeat round-trip samples (milliseconds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTracker {
    pub samples: VecDeque<f64>,
    pub capacity: usize,
}

impl LatencyTracker {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn record(&mut self, millis: f64) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(millis);
    }

    pub fn average(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }
}
