//! Strongly-typed telemetry config for the supervisor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    pub enabled: bool,
    /// `"disk"`, `"forward"`, `"both"` or `"none"`.
    pub output: String,
    pub disk: DiskSettings,
    pub forward: ForwardSettings,
    pub metrics: MetricsSettings,
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            output: "both".to_string(),
            disk: DiskSettings::default(),
            forward: ForwardSettings::default(),
            metrics: MetricsSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSettings {
    pub enabled: bool,
    pub path: String,
    pub filename_pattern: String,
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
    pub retention_days: u32,
}

impl Default for DiskSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/var/log/supervisor/telemetry".to_string(),
            filename_pattern: "telemetry_{date}.json".to_string(),
            buffer_size: 500,
            flush_interval_seconds: 5,
            retention_days: 7,
        }
    }
}

/// Settings for forwarding events to the wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardSettings {
    pub enabled: bool,
    pub wrapper_address: String,
    pub wrapper_port: u16,
    /// Tenant stamped on forwarded envelopes when the event carries none.
    #[serde(default = "default_forward_tenant_id")]
    pub tenant_id: String,
    pub batch_size: usize,
    pub flush_interval_seconds: u64,
    pub retry_attempts: u32,
    pub compress: bool,
}

/// Default tenant for forwarded telemetry envelopes.
fn default_forward_tenant_id() -> String {
    "supervisor".to_string()
}

impl Default for ForwardSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            wrapper_address: "172.16.0.1".to_string(),
            wrapper_port: 9090,
            tenant_id: default_forward_tenant_id(),
            batch_size: 50,
            flush_interval_seconds: 10,
            retry_attempts: 3,
            compress: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSettings {
    pub enabled: bool,
    pub collection_interval_seconds: u64,
}

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_seconds: 10,
        }
    }
}