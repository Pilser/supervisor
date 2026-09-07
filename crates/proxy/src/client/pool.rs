//! Connection pool to the wrapper.
//!
//! [`ConnectionPool`] tracks the pooled connections (per host) so higher levels
//! can decide whether the wrapper is still reachable. It is intentionally a
//! simple bookkeeping structure: requests open short-lived TCP connections per
//! send, and the pool records endpoint health.

use std::sync::Mutex;

/// A single pooled connection handle.
#[derive(Debug, Clone)]
pub struct PooledConn {
    /// Unique id of this pooled connection.
    pub id: String,
    /// Endpoint this connection targets.
    pub endpoint: String,
    /// Whether the connection is currently considered healthy.
    pub healthy: bool,
    /// When this connection was established.
    pub established: chrono::DateTime<chrono::Utc>,
}

impl PooledConn {
    /// Create a healthy pooled connection for `endpoint`.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            endpoint: endpoint.into(),
            healthy: true,
            established: chrono::Utc::now(),
        }
    }
}

/// A small bookkeeping pool of wrapper connections.
#[derive(Debug)]
pub struct ConnectionPool {
    conns: Mutex<Vec<PooledConn>>,
    max: usize,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(8)
    }
}

impl ConnectionPool {
    /// Create a pool with at most `max` tracked connections.
    pub fn new(max: usize) -> Self {
        Self {
            conns: Mutex::new(Vec::new()),
            max: max.max(1),
        }
    }

    /// The maximum number of tracked connections.
    pub fn max(&self) -> usize {
        self.max
    }

    /// Number of tracked connections.
    pub fn len(&self) -> usize {
        self.conns.lock().unwrap().len()
    }

    /// Whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Record a new pooled connection for `endpoint` (no-op if at capacity).
    pub fn add(&self, endpoint: impl Into<String>) -> PooledConn {
        let conn = PooledConn::new(endpoint);
        if let Ok(mut guard) = self.conns.lock() {
            if guard.len() < self.max {
                guard.push(conn.clone());
            }
        }
        conn
    }

    /// True if at least one tracked connection is healthy.
    pub fn healthy(&self) -> bool {
        self.conns
            .lock()
            .unwrap()
            .iter()
            .any(|c: &PooledConn| c.healthy)
    }

    /// Mark a connection unhealthy by id.
    pub fn mark_unhealthy(&self, id: &str) {
        if let Ok(mut guard) = self.conns.lock() {
            if let Some(c) = guard.iter_mut().find(|c| c.id == id) {
                c.healthy = false;
            }
        }
    }

    /// Mark a connection healthy by id.
    pub fn mark_healthy(&self, id: &str) {
        if let Ok(mut guard) = self.conns.lock() {
            if let Some(c) = guard.iter_mut().find(|c| c.id == id) {
                c.healthy = true;
            }
        }
    }
}