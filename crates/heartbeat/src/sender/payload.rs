//! Heartbeat payload construction.

use shared::Result;
use serde::{Deserialize, Serialize};

/// Payload posted to the wrapper on every heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub vm_id: String,
    pub tenant_id: String,
    pub timestamp_unix_millis: u64,
    pub programs: serde_json::Value,
    pub resources: serde_json::Value,
    pub custom: serde_json::Value,
}

/// Assemble a heartbeat payload from its parts.
pub fn build_heartbeat(
    vm_id: &str,
    tenant_id: &str,
    programs: serde_json::Value,
    resources: serde_json::Value,
    custom: serde_json::Value,
) -> Result<HeartbeatPayload> {
    Ok(HeartbeatPayload {
        vm_id: vm_id.to_string(),
        tenant_id: tenant_id.to_string(),
        timestamp_unix_millis: shared::now_unix_millis(),
        programs,
        resources,
        custom,
    })
}
