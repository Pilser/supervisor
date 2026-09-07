//! Handler layer: thin `DataProxy` abstraction plus typed tool handlers.
//!
//! Handlers accept typed arguments, translate them into the canonical wrapper
//! envelope (`{ "type", "tenant_id", "timestamp", "payload": { "op", "data" } }`)
//! via the `proxy` crate's [`RequestBuilder`](proxy::RequestBuilder), and
//! return a serializable result. `tenant_id` is required on every envelope;
//! tool params carrying `tenant_id` use it, otherwise `"local"` is stamped.

pub mod configs;
pub mod history;
pub mod memory;
pub mod profile;
pub mod projects;
pub mod reports;
pub mod skills;
pub mod tenant;

use std::sync::Arc;

use serde_json::Value;

use crate::tools::handler::BoxFuture;

/// A boxed future used by `DataProxy` implementations.
pub type ProxyFuture<'a> = BoxFuture<'a, anyhow::Result<Value>>;

/// Shared handle to a data proxy.
pub type SharedProxy = Arc<dyn DataProxy>;

/// Thin abstraction over the wrapper: a single `send` that takes the canonical
/// envelope JSON and returns the JSON result. Kept intentionally minimal — no
/// real network is needed to compile.
pub trait DataProxy: Send + Sync {
    /// Send a canonical envelope and await the JSON response.
    fn send(&self, json: Value) -> ProxyFuture<'static>;
}

/// A `DataProxy` that echoes its input, used when no wrapper is attached.
pub struct NoopProxy;

impl DataProxy for NoopProxy {
    fn send(&self, json: Value) -> ProxyFuture<'static> {
        Box::pin(async move { Ok(json) })
    }
}

/// Build a shared no-op proxy.
pub fn noop_proxy() -> SharedProxy {
    Arc::new(NoopProxy)
}

/// Tenant id stamped when tool params carry no explicit `tenant_id`.
pub const LOCAL_TENANT_ID: &str = "local";

/// Extract the tenant id from tool params, defaulting to `"local"`.
pub fn tenant_from_params(params: &Value) -> String {
    params
        .get("tenant_id")
        .and_then(Value::as_str)
        .unwrap_or(LOCAL_TENANT_ID)
        .to_string()
}

/// Build the canonical wrapper envelope for a tool call:
/// `{ "type", "tenant_id", "timestamp", "payload": { "op", "data" } }`.
///
/// `request_type` is the wrapper request-type wire string (`"db_read"` /
/// `"db_write"`), `operation` the dotted method name (e.g. `"memory.save"`),
/// and `data` the tool arguments.
pub fn proxy_request(
    request_type: &str,
    tenant_id: &str,
    operation: &str,
    data: Value,
) -> anyhow::Result<Value> {
    let rtype = proxy::RequestType::from_str(request_type)
        .ok_or_else(|| anyhow::anyhow!("unknown wrapper request type: {request_type}"))?;
    let tenant = shared::TenantContext::new(tenant_id, "", "", "");
    proxy::RequestBuilder::new(rtype)
        .operation(operation)
        .tenant(tenant)
        .data(data)
        .build()
}

/// Dispatch a request through `proxy` wrapped in the canonical envelope.
pub async fn proxy_call(
    proxy: &dyn DataProxy,
    request_type: &str,
    tenant_id: &str,
    operation: &str,
    params: Value,
) -> anyhow::Result<Value> {
    proxy
        .send(proxy_request(request_type, tenant_id, operation, params)?)
        .await
}
