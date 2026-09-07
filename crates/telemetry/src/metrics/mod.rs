//! In-process metric registry.

pub mod counter;
pub mod gauge;
pub mod histogram;
pub mod latency;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use counter::Counter;
pub use gauge::Gauge;
pub use histogram::Histogram;
pub use latency::LatencyTracker;

#[derive(Clone, Default)]
pub struct Metrics {
    counts: Arc<Mutex<HashMap<String, u64>>>,
    latencies: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sample(&self, name: &str, value: f64) {
        let mut l = self.latencies.lock().unwrap();
        l.entry(name.to_string()).or_default().push(value);
    }

    pub fn record_event(&self, kind: String) {
        let category = kind.split('.').next().unwrap_or("system").to_string();
        let mut c = self.counts.lock().unwrap();
        *c.entry(format!("event.{category}")).or_insert(0) += 1;
    }
}

impl std::fmt::Debug for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metrics({} series)", self.latencies.lock().unwrap().len())
    }
}