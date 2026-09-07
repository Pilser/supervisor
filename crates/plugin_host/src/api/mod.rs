//! Inter-plugin API registry.
//!
//! Plugins expose named callable endpoints that other plugins can look up and
//! invoke at runtime (RPC-style). The registry is a simple name → function
//! map guarded by a mutex.

pub mod call;
pub mod lookup;
pub mod register;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::Result;

/// An inter-plugin API endpoint: maps a JSON request to a JSON response.
pub type ApiFn = dyn Fn(&serde_json::Value) -> Result<serde_json::Value> + Send + Sync;

/// A registered inter-plugin API endpoint.
pub struct ApiEntry {
    pub name: String,
    pub func: Arc<ApiFn>,
}

impl ApiEntry {
    /// Convenience accessor for the endpoint name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// The inter-plugin API registry held by the host.
#[derive(Clone)]
pub struct ApiRegistry {
    inner: Arc<Mutex<HashMap<String, Arc<ApiEntry>>>>,
}

impl ApiRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for ApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ApiRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiRegistry").finish_non_exhaustive()
    }
}