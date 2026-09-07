//! Registering and updating inter-plugin API endpoints.

use std::sync::Arc;

use crate::api::{ApiEntry, ApiFn, ApiRegistry};

impl ApiRegistry {
    /// Register (or replace) the endpoint `name` backed by `func`.
    pub fn register(&self, name: impl Into<String>, func: Arc<ApiFn>) -> crate::Result<()> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(crate::SupervisorError::InvalidInput(
                "cannot register an API with an empty name".into(),
            ));
        }
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.insert(name.clone(), Arc::new(ApiEntry { name, func }));
        Ok(())
    }

    /// Shortcut registering an endpoint built from a plain closure.
    pub fn register_fn<F>(&self, name: impl Into<String>, func: F) -> crate::Result<()>
    where
        F: Fn(&serde_json::Value) -> crate::Result<serde_json::Value> + Send + Sync + 'static,
    {
        self.register(name, Arc::new(func))
    }

    /// Remove an endpoint by name. Returns true if one existed.
    pub fn unregister(&self, name: &str) -> crate::Result<bool> {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        Ok(map.remove(name).is_some())
    }
}