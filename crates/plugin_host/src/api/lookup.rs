//! Looking up registered inter-plugin API endpoints.

use std::sync::Arc;

use crate::api::{ApiEntry, ApiRegistry};

impl ApiRegistry {
    /// The endpoint registered under `name`, if present.
    pub fn lookup(&self, name: &str) -> Option<Arc<ApiEntry>> {
        let map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.get(name).cloned()
    }

    /// True if an endpoint named `name` is registered.
    pub fn contains(&self, name: &str) -> bool {
        let map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.contains_key(name)
    }

    /// Names of all registered endpoints.
    pub fn names(&self) -> Vec<String> {
        let map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.keys().cloned().collect()
    }
}