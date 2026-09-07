//! `telemetry` — structured events written to local disk and forwarded to the
//! wrapper for central monitoring. Mirrors the wrapper's event schema exactly.

pub mod config;
pub mod event;
pub mod forward;
pub mod macros;
pub mod metrics;
pub mod models;
pub mod writer;

pub use config::TelemetrySettings;
pub use event::{emit_event, EventBuilder, Level};
pub use forward::{ForwardClient, Forwarder};
pub use metrics::{Counter, Gauge, Histogram, Metrics};
pub use models::{
    ErrorPayload, MetricPayload, RequestPayload, SpanPayload, SpanStatus, StatePayload,
};
pub use models::Event;

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::writer::FileWriter;

/// The central supervisor telemetry handle.
#[derive(Clone)]
pub struct Telemetry {
    pub config: Arc<TelemetrySettings>,
    pub metrics: Arc<Metrics>,
    enabled: bool,
    file_writer: Arc<RwLock<Option<FileWriter>>>,
    forwarder: Arc<Forwarder>,
}

impl std::fmt::Debug for Telemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Telemetry")
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl Telemetry {
    pub fn init(settings: TelemetrySettings) -> Self {
        let enabled = settings.enabled;
        let write_disk = settings.disk.enabled && output_writes_disk(&settings.output);
        let forward_enabled = settings.forward.enabled && output_forwards(&settings.output);
        let file_writer = if write_disk {
            Arc::new(RwLock::new(Some(FileWriter::new(settings.disk.clone()))))
        } else {
            Arc::new(RwLock::new(None))
        };
        let forwarder = Forwarder::new(settings.forward.clone());
        if forward_enabled {
            forwarder.start();
        }
        Self {
            config: Arc::new(settings),
            metrics: Arc::new(Metrics::new()),
            enabled,
            file_writer,
            forwarder: Arc::new(forwarder),
        }
    }

    pub fn noop() -> Self {
        let settings = TelemetrySettings::default();
        Self {
            enabled: false,
            config: Arc::new(settings),
            metrics: Arc::new(Metrics::new()),
            file_writer: Arc::new(RwLock::new(None)),
            forwarder: Arc::new(Forwarder::disabled()),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn builder(&self) -> EventBuilder {
        EventBuilder::new(self.clone())
    }

    pub fn emit(&self, event: Event) {
        if !self.enabled {
            return;
        }
        let kind = event.kind.clone();
        let file_writer = Arc::clone(&self.file_writer);
        let forwarder = Arc::clone(&self.forwarder);
        let metrics = Arc::clone(&self.metrics);
        tokio::task::spawn(async move {
            {
                let mut guard = file_writer.write().await;
                if let Some(w) = guard.as_mut() {
                    let _ = w.write(&event).await;
                }
            }
            forwarder.submit(event);
            metrics.record_event(kind);
        });
    }

    pub async fn flush(&self) {
        let mut guard = self.file_writer.write().await;
        if let Some(w) = guard.as_mut() {
            let _ = w.flush().await;
        }
    }
}

fn output_writes_disk(output: &str) -> bool {
    matches!(output, "disk" | "both")
}

fn output_forwards(output: &str) -> bool {
    matches!(output, "forward" | "both")
}

/// Current RFC3339 timestamp with nanosecond precision.
pub fn now_string() -> String {
    core::now_rfc3339()
}