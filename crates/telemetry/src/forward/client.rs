//! TCP client that forwards event batches to the wrapper as NDJSON lines.

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use crate::models::Event;

/// Timeout for establishing the TCP connection to the wrapper.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Timeout for writing the NDJSON batch.
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// A thin TCP client pointed at the wrapper. Each event is wrapped in the
/// canonical envelope `{ "type": "telemetry.event", "tenant_id", "timestamp",
/// "payload": <event> }` and sent as one NDJSON line.
#[derive(Clone)]
pub struct ForwardClient {
    address: String,
    port: u16,
    tenant_id: String,
    endpoint: String,
}

impl std::fmt::Debug for ForwardClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForwardClient")
            .field("endpoint", &self.endpoint)
            .field("tenant_id", &self.tenant_id)
            .finish()
    }
}

impl ForwardClient {
    pub fn new(address: &str, port: u16) -> Self {
        Self {
            address: address.to_string(),
            port,
            tenant_id: "supervisor".to_string(),
            endpoint: format!("tcp://{address}:{port}"),
        }
    }

    /// Override the tenant id stamped on envelopes for events that carry none.
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = tenant_id.into();
        self
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Send a batch of events: one NDJSON envelope line per event.
    pub async fn send_batch(&self, events: &[Event]) -> anyhow::Result<()> {
        let mut out = String::new();
        for event in events {
            let payload = serde_json::to_value(event)?;
            let envelope = serde_json::json!({
                "type": "telemetry.event",
                "tenant_id": event.tenant_id.clone().unwrap_or_else(|| self.tenant_id.clone()),
                "timestamp": shared::now_rfc3339(),
                "payload": payload,
            });
            out.push_str(&serde_json::to_string(&envelope)?);
            out.push('\n');
        }
        if out.is_empty() {
            return Ok(());
        }
        let addr = format!("{}:{}", self.address, self.port);
        let mut stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
            .await
            .map_err(|_| anyhow::anyhow!("telemetry connect to {addr} timed out"))?
            .map_err(|e| anyhow::anyhow!("telemetry connect to {addr}: {e}"))?;
        timeout(WRITE_TIMEOUT, stream.write_all(out.as_bytes()))
            .await
            .map_err(|_| anyhow::anyhow!("telemetry write to {addr} timed out"))?
            .map_err(|e| anyhow::anyhow!("telemetry write to {addr}: {e}"))?;
        // Fire-and-forget: the bytes are delivered; a missing wrapper ack
        // must not fail the forward.
        Ok(())
    }
}
