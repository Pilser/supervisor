//! Proxy handle: custom wrapper requests for plugins.

/// Plugin-facing handle for wrapper-bound requests.
#[derive(Debug, Clone)]
pub struct ProxyApi {
    pub wrapper_address: String,
    pub wrapper_port: u16,
}

impl ProxyApi {
    pub fn new(wrapper_address: &str, wrapper_port: u16) -> Self {
        Self {
            wrapper_address: wrapper_address.to_string(),
            wrapper_port,
        }
    }

    /// Build a wrapper envelope without depending on the `proxy` crate.
    pub fn envelope(
        &self,
        request_type: &str,
        tenant_id: &str,
        operation: &str,
        data: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "type": request_type,
            "tenant_id": tenant_id,
            "operation": operation,
            "data": data,
        })
    }
}
