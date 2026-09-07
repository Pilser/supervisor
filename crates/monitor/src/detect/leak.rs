//! Detect memory leaks via monotonic RSS growth.

use core::Result;
use serde::{Deserialize, Serialize};

/// Flags a leak when consecutive samples keep growing past a threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakDetector {
    pub growth_threshold_bytes: u64,
    pub min_samples: usize,
}

impl LeakDetector {
    pub fn new(growth_threshold_bytes: u64) -> Self {
        Self {
            growth_threshold_bytes,
            min_samples: 3,
        }
    }

    /// True when every sample grows and total growth exceeds the threshold.
    pub fn check_growth(
        &self,
        samples: &[u64],
        telemetry: &telemetry::Telemetry,
    ) -> Result<bool> {
        if samples.len() < self.min_samples.max(2) {
            return Ok(false);
        }
        let growing = samples.windows(2).all(|w| w[1] >= w[0]);
        let growth = samples.last().unwrap_or(&0).saturating_sub(samples[0]);
        let leak = growing && growth >= self.growth_threshold_bytes;
        if leak {
            telemetry
                .builder()
                .level(telemetry::Level::Warn)
                .kind("monitor.leak")
                .category("monitor")
                .message(format!("possible memory leak: +{growth} bytes"))
                .emit();
        }
        Ok(leak)
    }
}
