//! Attaching tenant identity to every wrapper request.

use core::TenantContext;
use serde_json::Value;

/// Extract the `tenant_id` string from a [`TenantContext`].
pub fn tenant_id(tenant: &TenantContext) -> String {
    tenant.tenant_id.clone()
}

/// Attach the `tenant_id` from a [`TenantContext`] into a wrapper request
/// envelope, e.g. `{ "type": "...", "tenant_id": "...", ... }`.
pub fn attach_tenant(envelope: &mut Value, tenant: &TenantContext) {
    match envelope {
        Value::Object(map) => {
            map.insert("tenant_id".to_string(), Value::String(tenant.tenant_id.clone()));
        }
        other => {
            *other = serde_json::json!({
                "tenant_id": tenant.tenant_id,
                "inner": other.take(),
            });
        }
    }
}

/// Return the `tenant_id` string, failing when no tenant was supplied.
///
/// `tenant_id` is mandatory on the wrapper wire protocol: every envelope must
/// carry it top-level. Builders use this to fail fast instead of emitting an
/// envelope without one.
pub fn require_tenant_id(tenant: Option<&TenantContext>) -> anyhow::Result<String> {
    tenant
        .map(|t| t.tenant_id.clone())
        .ok_or_else(|| anyhow::anyhow!("tenant_id is required on every wrapper request"))
}

/// Attach the full tenant context (all identifying fields) to a request payload.
pub fn attach_context(envelope: &mut Value, tenant: &TenantContext) {
    if let Value::Object(map) = envelope {
        map.insert("tenant_id".to_string(), Value::String(tenant.tenant_id.clone()));
        map.insert("vm_id".to_string(), Value::String(tenant.vm_id.clone()));
        map.insert("category_id".to_string(), Value::String(tenant.category_id.clone()));
        map.insert("username".to_string(), Value::String(tenant.username.clone()));
    }
}