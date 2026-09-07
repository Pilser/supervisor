//! Telemetry configuration.

pub mod load;
pub mod settings;

pub use settings::{
    DiskSettings, ForwardSettings, MetricsSettings, TelemetrySettings,
};