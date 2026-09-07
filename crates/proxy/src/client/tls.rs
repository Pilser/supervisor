//! TLS configuration for the wrapper connection.
//!
//! The wrapper is reached over plain HTTP by default; TLS can be enabled here.

/// TLS settings for the wrapper endpoint.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Whether HTTPS is used for the wrapper endpoint.
    pub enabled: bool,
    /// Whether to skip certificate verification (dev only).
    pub insecure: bool,
    /// Max pooled/idle connections to the wrapper.
    pub max_connections: usize,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            insecure: false,
            max_connections: 8,
        }
    }
}

impl TlsConfig {
    /// True if TLS is enabled for the wrapper endpoint.
    pub fn tls_enabled(&self) -> bool {
        self.enabled
    }

    /// True if certificate verification should be skipped.
    pub fn insecure(&self) -> bool {
        self.insecure
    }

    /// Max pooled connections.
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    /// Enable/disable TLS.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Read the `tls_enabled` flag from a config.
pub fn tls_enabled(cfg: &TlsConfig) -> bool {
    cfg.tls_enabled()
}