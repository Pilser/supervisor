//! Invoke a registered inter-plugin API endpoint.

use serde_json::Value;

use crate::api::ApiRegistry;
use crate::SupervisorError;

impl ApiRegistry {
    /// Call the endpoint `name` with `args`, returning its JSON response.
    pub fn call(&self, name: &str, args: Value) -> crate::Result<Value> {
        let entry = self
            .lookup(name)
            .ok_or_else(|| SupervisorError::NotFound(format!("plugin API `{name}`")))?;
        (entry.func)(&args)
    }

    /// Call an endpoint with a JSON object literal built from `key`/`value`.
    pub fn call_json(&self, name: &str, request: &str) -> crate::Result<Value> {
        let args: Value = serde_json::from_str(request).map_err(SupervisorError::from)?;
        self.call(name, args)
    }
}