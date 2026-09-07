//! Metric payload attached to `metric.*` events.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPayload {
    pub name: String,
    pub kind: String,
    pub value: f64,
    pub unit: String,
    pub labels: std::collections::HashMap<String, String>,
}