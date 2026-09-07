//! Forwarding telemetry events to the wrapper.

pub mod batch;
pub mod client;
pub mod compress;
pub mod retry;

pub use batch::Batcher;
pub use client::ForwardClient;

use std::sync::Mutex;

use tokio::sync::mpsc;

use crate::config::ForwardSettings;
use crate::models::Event;

/// Background forwarder: queues events and posts them to the wrapper in
/// batches. Spawns a single worker task on `start()`.
pub struct Forwarder {
    client: ForwardClient,
    settings: ForwardSettings,
    tx: Mutex<Option<mpsc::UnboundedSender<Event>>>,
}

impl std::fmt::Debug for Forwarder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Forwarder(endpoint={})",
            self.client.endpoint()
        )
    }
}

impl Forwarder {
    pub fn new(settings: ForwardSettings) -> Self {
        let client = ForwardClient::new(&settings.wrapper_address, settings.wrapper_port)
            .with_tenant(settings.tenant_id.clone());
        Self {
            client,
            settings,
            tx: Mutex::new(None),
        }
    }

    /// A forwarder that never sends.
    pub fn disabled() -> Self {
        let settings = ForwardSettings::default();
        Self::new(settings)
    }

    /// Start the worker task (idempotent).
    pub fn start(&self) {
        let is_started = || self.tx.lock().unwrap().is_some();
        if is_started() {
            return;
        }
        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        *self.tx.lock().unwrap() = Some(tx.clone());
        let client = self.client.clone();
        let batch_size = self.settings.batch_size.max(1);
        let interval = self.settings.flush_interval_seconds.max(1);
        let retries = self.settings.retry_attempts;
        tokio::spawn(async move {
            Batcher::run(client.clone(), rx, batch_size, interval, retries).await;
        });
    }

    /// Enqueue an event to be forwarded.
    pub fn submit(&self, ev: Event) {
        if let Some(tx) = self.tx.lock().unwrap().as_ref() {
            let _ = tx.send(ev);
        }
    }
}