//! Establishing the connection to the wrapper.
//!
//! [`ProxyClient::connect`] records the `address:port` endpoint (the wire
//! transport is NDJSON over TCP) and assembles the shared client handles.
//! [`build_base_url`] derives the `http://<addr>:<port>` display URL from the
//! same endpoint configuration.

use std::sync::Arc;

use telemetry::Telemetry;
use tokio::sync::Mutex as AsyncMutex;

use crate::cache::CacheStore;
use crate::client::pool::ConnectionPool;
use crate::queue::OfflineQueue;

use super::tls::TlsConfig;
use super::ProxyClient;

/// Build the base URL for the wrapper endpoint from a host/port config.
pub fn build_base_url(host: &str, port: u16, tls: &TlsConfig) -> String {
    let scheme = if tls.tls_enabled() { "https" } else { "http" };
    format!("{scheme}://{host}:{port}")
}

impl ProxyClient {
    /// Establish a connection to the wrapper at `address:port` with default
    /// settings (no TLS, no-op telemetry).
    pub fn connect(address: &str, port: u16, token: &str) -> anyhow::Result<Self> {
        Self::connect_with(
            address,
            port,
            token,
            TlsConfig::default(),
            Telemetry::noop(),
        )
    }

    /// Establish a connection to the wrapper with full configuration.
    pub fn connect_with(
        address: &str,
        port: u16,
        token: &str,
        tls: TlsConfig,
        telemetry: Telemetry,
    ) -> anyhow::Result<Self> {
        let base_url = build_base_url(address, port, &tls);

        let mut builder = reqwest::Client::builder()
            .pool_max_idle_per_host(tls.max_connections())
            .timeout(std::time::Duration::from_secs(30));
        if tls.insecure() {
            builder = builder.danger_accept_invalid_certs(true);
        }
        let client = builder.build()?;

        let pool = Arc::new(ConnectionPool::new(tls.max_connections()));
        // Seed the pool so `healthy()` reflects the configured endpoint.
        pool.add(base_url.clone());

        let cache = CacheStore::new(crate::cache::CacheTtl::default());

        Ok(Self {
            client,
            base_url,
            address: address.to_string(),
            port,
            token: Arc::new(AsyncMutex::new(token.to_string())),
            pool,
            cache: Arc::new(AsyncMutex::new(cache)),
            queue: Arc::new(OfflineQueue::new()),
            telemetry,
            tls_enabled: tls.tls_enabled(),
        })
    }
}