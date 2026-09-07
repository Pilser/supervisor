//! [`RequestBuilder`] — a fluent helper that produces a `Request` or directly
//! the wrapper-protocol JSON envelope.
//!
//! The wrapper wire protocol is NDJSON over TCP. Every envelope carries
//! `{ "type", "tenant_id", "timestamp", "payload": { "op", "data" } }` where
//! `tenant_id` is always present and `timestamp` is RFC3339.

use core::TenantContext;
use serde_json::{json, Value};

use super::tenant;
use super::{Request, RequestType};

/// Fluent builder for wrapper requests.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    rtype: RequestType,
    operation: String,
    tenant: Option<TenantContext>,
    data: Value,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new(RequestType::DbRead)
    }
}

impl RequestBuilder {
    /// Begin building a request of the given type.
    pub fn new(rtype: RequestType) -> Self {
        Self {
            rtype,
            operation: String::new(),
            tenant: None,
            data: json!({}),
        }
    }

    /// Set the wrapper operation name (stored as `payload.op` on the wire).
    pub fn operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = operation.into();
        self
    }

    /// Set the tenant context (emitted as top-level `tenant_id`; required).
    pub fn tenant(mut self, tenant: TenantContext) -> Self {
        self.tenant = Some(tenant);
        self
    }

    /// Set the operation payload (stored as `payload.data` on the wire).
    pub fn data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Materialise the builder into a [`Request`].
    ///
    /// Fails when no tenant was attached: `tenant_id` is required on the wire.
    pub fn into_request(self) -> anyhow::Result<Request> {
        tenant::require_tenant_id(self.tenant.as_ref())?;
        // Checked present above.
        let tenant = self.tenant.expect("tenant checked by require_tenant_id");
        Ok(Request {
            rtype: self.rtype,
            operation: self.operation,
            tenant,
            data: self.data,
        })
    }

    /// Build the canonical wrapper envelope:
    /// `{ "type", "tenant_id", "timestamp", "payload": { "op", "data" } }`.
    ///
    /// Fails when no tenant was attached.
    pub fn build(self) -> anyhow::Result<Value> {
        let tenant_id = tenant::require_tenant_id(self.tenant.as_ref())?;
        Ok(json!({
            "type": self.rtype.as_str(),
            "tenant_id": tenant_id,
            "timestamp": core::now_rfc3339(),
            "payload": {
                "op": self.operation,
                "data": self.data,
            },
        }))
    }
}
