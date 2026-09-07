//! Request construction for the wrapper protocol.

pub mod builder;
pub mod sign;
pub mod tenant;
pub mod types;

pub use builder::RequestBuilder;
pub use types::RequestType;

use shared::TenantContext;
use serde_json::Value;

/// A fully-specified wrapper request: type, operation, tenant and data.
///
/// `tenant` is mandatory: the wire envelope always carries `tenant_id`.
#[derive(Debug, Clone)]
pub struct Request {
    /// Request category (`db_read`, `db_write`, `heartbeat`, ...).
    pub rtype: RequestType,
    /// The wrapper operation name, e.g. `memory.save` (sent as `payload.op`).
    pub operation: String,
    /// The tenant context attached to the request (sent as `tenant_id`).
    pub tenant: TenantContext,
    /// The operation payload (sent as `payload.data`).
    pub data: Value,
}

impl Request {
    /// Start building a request with type [`RequestType::DbRead`].
    pub fn builder() -> RequestBuilder {
        RequestBuilder::new(RequestType::DbRead)
    }

    /// Build a request with the mandatory tenant already attached.
    pub fn new(
        rtype: RequestType,
        operation: impl Into<String>,
        tenant: TenantContext,
        data: Value,
    ) -> Self {
        Self {
            rtype,
            operation: operation.into(),
            tenant,
            data,
        }
    }

    /// Convert this request into the canonical wrapper envelope
    /// `{ "type", "tenant_id", "timestamp", "payload": { "op", "data" } }`.
    pub fn into_json(self) -> anyhow::Result<Value> {
        RequestBuilder::new(self.rtype)
            .operation(self.operation)
            .tenant(self.tenant)
            .data(self.data)
            .build()
    }
}