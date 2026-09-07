//! TCP client connecting the supervisor to the wrapper.
//!
//! [`ProxyClient`] is the facade plugins and other supervisor crates talk to.
//! It dials `address:port` per request, writes one NDJSON envelope line
//! (`{ "type", "tenant_id", "timestamp", "payload" }`) and reads one NDJSON
//! response line back, plus the shared telemetry/cache/offline-queue handles.
//! `base_url` is retained as a display helper only — no HTTP is sent on the
//! wire. All fallible operations return `anyhow::Result<_>` and error classes
//! are exposed as [`ProxyError`].

pub mod connect;
pub mod pool;
pub mod retry;
pub mod tls;

pub use pool::{ConnectionPool, PooledConn};
pub use retry::RetryPolicy;
pub use tls::TlsConfig;

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use telemetry::{Level, Telemetry};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex as AsyncMutex;

use crate::cache::{CacheKey, CacheStore};
use crate::queue::{flush_queue, OfflineQueue};
use crate::request::{sign, Request, RequestBuilder, RequestType};
use crate::response::{
    error::ProxyError, parse::parse_response, transform::transform_response, Response,
};

/// Timeout for establishing the TCP connection to the wrapper.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Timeout for writing the request line and reading the response line.
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// The facade for sending wrapper requests.
#[derive(Clone)]
pub struct ProxyClient {
    /// Shared HTTP client handle (retained for API compatibility; the wrapper
    /// wire transport is plain TCP, see [`ProxyClient::send_json`]).
    pub client: reqwest::Client,
    /// Display URL of the wrapper endpoint, e.g. `http://127.0.0.1:9090`.
    /// Derived from `address`/`port`; the wire transport is TCP, not HTTP.
    pub base_url: String,
    /// Wrapper host each TCP stream connects to.
    pub address: String,
    /// Wrapper port each TCP stream connects to.
    pub port: u16,
    /// Current supervisor bearer token, mutable so auth can rotate it.
    pub token: Arc<AsyncMutex<String>>,
    /// Connection pool statistics to the wrapper.
    pub pool: Arc<ConnectionPool>,
    /// Cached wrapper responses to reduce round trips.
    pub cache: Arc<AsyncMutex<CacheStore>>,
    /// Buffer for requests issued while the wrapper is unreachable.
    pub queue: Arc<OfflineQueue>,
    /// Telemetry handle.
    pub telemetry: Telemetry,
    /// Whether TLS is enabled for the wrapper endpoint.
    pub tls_enabled: bool,
}

impl ProxyClient {
    /// Build a `ProxyClient` against the wrapper at `address:port` with no TLS
    /// and a no-op telemetry sink. Telemetry can be attached via
    /// [`ProxyClient::connect_with`].
    pub fn new(address: &str, port: u16, token: &str) -> anyhow::Result<Self> {
        Self::connect(address, port, token)
    }

    /// Return a copy of the current token.
    pub async fn token(&self) -> String {
        self.token.lock().await.clone()
    }

    /// Replace the current token (used by auth rotation).
    pub async fn set_token(&self, token: &str) {
        *self.token.lock().await = token.to_string();
    }

    /// Send one NDJSON envelope line to the wrapper over TCP and return the
    /// parsed response envelope.
    ///
    /// Opens a `TcpStream` to `address:port`, writes the envelope plus `\n`,
    /// then reads a single `\n`-terminated response line back. Connect
    /// failures surface as [`ProxyError::Unreachable`], I/O timeouts as
    /// [`ProxyError::Timeout`], and a response payload reporting an auth
    /// failure as [`ProxyError::Unauthorized`].
    pub async fn send_json(&self, msg: Value) -> anyhow::Result<Value> {
        let rtype = msg
            .get("type")
            .and_then(Value::as_str)
            .and_then(RequestType::from_str);
        let operation = msg
            .get("payload")
            .and_then(|p| p.get("op"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let tenant_id = msg.get("tenant_id").and_then(Value::as_str).map(String::from);

        self.emit_sent(rtype, &operation, tenant_id.as_deref());

        let mut line = serde_json::to_string(&msg)
            .map_err(|e| ProxyError::Parse(format!("envelope is not JSON: {e}")))?;
        line.push('\n');

        let start = Instant::now();
        let addr = format!("{}:{}", self.address, self.port);
        let stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
            .await
            .map_err(|_| ProxyError::Timeout(format!("connect to wrapper {addr} timed out")))?
            .map_err(|e| ProxyError::Unreachable(format!("wrapper unreachable at {addr}: {e}")))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        tokio::time::timeout(IO_TIMEOUT, writer.write_all(line.as_bytes()))
            .await
            .map_err(|_| ProxyError::Timeout("write to wrapper timed out".to_string()))?
            .map_err(|e| ProxyError::Network(format!("write to wrapper failed: {e}")))?;

        let mut response_line = String::new();
        tokio::time::timeout(IO_TIMEOUT, reader.read_line(&mut response_line))
            .await
            .map_err(|_| ProxyError::Timeout("read from wrapper timed out".to_string()))?
            .map_err(|e| ProxyError::Network(format!("read from wrapper failed: {e}")))?;

        if response_line.trim().is_empty() {
            return Err(ProxyError::Transport(
                "wrapper closed the connection without a response line".to_string(),
            )
            .into());
        }

        let value: Value = serde_json::from_str(response_line.trim())
            .map_err(|e| ProxyError::Parse(format!("wrapper response is not NDJSON: {e}")))?;

        self.emit_response(rtype, tenant_id.as_deref(), start.elapsed());

        if is_unauthorized(&value) {
            return Err(ProxyError::Unauthorized("wrapper rejected token".to_string()).into());
        }

        Ok(value)
    }

    /// Pipeline entry point taking a pre-built [`RequestBuilder`].
    pub async fn request(&self, builder: RequestBuilder) -> anyhow::Result<Response> {
        let req = builder.into_request()?;
        self.call(req).await
    }

    /// The full facade path: build the envelope, attach tenant, embed the
    /// token into `payload`, send over TCP, parse, transform and cache. On a
    /// retryable failure it enqueues the request into the [`OfflineQueue`]
    /// and returns the (retryable) error.
    pub async fn call(&self, req: Request) -> anyhow::Result<Response> {
        let rtype = req.rtype;
        let operation = req.operation.clone();
        let tenant_id = req.tenant.tenant_id.clone();

        let key = CacheKey::scoped(&tenant_id, &rtype, &operation);

        // Cache hit short-circuits reads.
        if rtype.is_read() {
            if let Some(cached) = self.cache.lock().await.get(&key) {
                self.emit_cache_hit(&key, &operation);
                return Ok(Response::from_cache(cached));
            }
            self.emit_cache_miss(&key, &operation);
        }

        // Build the canonical envelope and embed the supervisor token into
        // `payload` (there are no HTTP headers on the TCP wire).
        let mut envelope = req.into_json()?;
        {
            let token = self.token.lock().await;
            sign::embed_token_in_payload(&mut envelope, &token)?;
        }

        let result = self.send_json(envelope.clone()).await;
        let response = match result {
            Ok(raw) => {
                let mut resp = parse_response(Some(rtype), raw);
                transform_response(&mut resp);
                resp
            }
            Err(err) => {
                if let Some(proxy) = err.downcast_ref::<ProxyError>() {
                    if proxy.is_retryable() {
                        self.emit_unreachable(proxy.kind());
                        self.queue.enqueue(envelope).await;
                        self.emit_queued();
                    }
                }
                return Err(err);
            }
        };

        // Refresh / invalidate the local cache.
        if response.ok {
            let mut cache = self.cache.lock().await;
            if rtype.is_read() {
                let ttl = cache.ttl();
                cache.put(key, response.data.clone(), Some(ttl.for_request(rtype)));
            } else {
                crate::cache::invalidate::invalidate_key(&mut cache, &key);
            }
        }

        Ok(response)
    }

    /// Drain any locally queued offline requests through this client.
    pub async fn flush(&self) -> anyhow::Result<usize> {
        let flushed = flush_queue(&self.queue, self).await;
        self.emit_reconnected();
        Ok(flushed)
    }

    fn emit_sent(&self, rtype: Option<RequestType>, operation: &str, tenant: Option<&str>) {
        let mut b = self
            .telemetry
            .builder()
            .level(Level::Debug)
            .kind("request.to_wrapper.send")
            .tenant_id(tenant.map(String::from))
            .data("transport", "tcp")
            .message("request sent to wrapper");
        if let Some(r) = rtype {
            b = b.data("type", r.as_str());
        }
        b.data("operation", operation).emit();
    }

    fn emit_response(&self, rtype: Option<RequestType>, tenant: Option<&str>, dur: std::time::Duration) {
        let mut b = self
            .telemetry
            .builder()
            .level(Level::Debug)
            .kind("response from wrapper")
            .tenant_id(tenant.map(String::from))
            .data("transport", "tcp")
            .data("duration_ms", dur.as_millis() as u64)
            .message("response received from wrapper");
        if let Some(r) = rtype {
            b = b.data("type", r.as_str());
        }
        b.emit();
    }

    fn emit_unreachable(&self, kind: &'static str) {
        self.telemetry
            .builder()
            .level(Level::Warn)
            .kind("wrapper.unreachable")
            .data("kind", kind)
            .message("wrapper unreachable; request queued")
            .emit();
    }

    fn emit_reconnected(&self) {
        self.telemetry
            .builder()
            .level(Level::Info)
            .kind("wrapper.reconnected")
            .message("wrapper reconnected; offline queue flushed")
            .emit();
    }

    fn emit_queued(&self) {
        self.telemetry
            .builder()
            .level(Level::Debug)
            .kind("request.queued")
            .message("request enqueued into offline queue")
            .emit();
    }

    fn emit_cache_hit(&self, ops: &CacheKey, _op: &str) {
        self.telemetry
            .builder()
            .level(Level::Debug)
            .kind("cache.hit")
            .data("key", ops.cache_index().to_string())
            .message("cache hit")
            .emit();
    }

    fn emit_cache_miss(&self, ops: &CacheKey, _op: &str) {
        self.telemetry
            .builder()
            .level(Level::Debug)
            .kind("cache.miss")
            .data("key", ops.cache_index().to_string())
            .message("cache miss")
            .emit();
    }
}

/// True when a wrapper response envelope reports an auth failure.
///
/// The TCP wire has no status codes, so auth rejections are detected from the
/// response `payload` (`status_code` 401/403 or an error/message mentioning
/// unauthorized/forbidden).
fn is_unauthorized(value: &Value) -> bool {
    let payload = value.get("payload").unwrap_or(value);
    if let Some(code) = payload.get("status_code").and_then(Value::as_u64) {
        if code == 401 || code == 403 {
            return true;
        }
    }
    for key in ["error", "message", "status"] {
        if let Some(text) = payload.get(key).and_then(Value::as_str) {
            let lower = text.to_ascii_lowercase();
            if lower.contains("unauthorized") || lower.contains("forbidden") {
                return true;
            }
        }
    }
    false
}
