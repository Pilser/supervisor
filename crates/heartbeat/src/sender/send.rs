//! Send a heartbeat to the wrapper over TCP (NDJSON envelope).

use core::{Result, SupervisorError};

use super::payload::HeartbeatPayload;

/// Forward one heartbeat: wrap the status payload in the canonical
/// `heartbeat` envelope and send it through the proxy client, which writes
/// one NDJSON line to `wrapper_address:wrapper_port` and reads the ack line
/// back.
pub async fn send_heartbeat(
    payload: &HeartbeatPayload,
    wrapper_address: &str,
    wrapper_port: u16,
    token: &str,
    telemetry: &telemetry::Telemetry,
) -> Result<()> {
    let client = proxy::ProxyClient::new(wrapper_address, wrapper_port, token)
        .map_err(|e| SupervisorError::Network(format!("proxy client: {e}")))?;
    let tenant =
        core::TenantContext::new(payload.tenant_id.clone(), payload.vm_id.clone(), "", "");
    let data = serde_json::to_value(payload)
        .map_err(|e| SupervisorError::Heartbeat(format!("heartbeat payload: {e}")))?;
    let builder = proxy::RequestBuilder::new(proxy::RequestType::Heartbeat)
        .operation("heartbeat")
        .tenant(tenant)
        .data(data);
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("heartbeat.send")
        .category("heartbeat")
        .message(format!("heartbeat to {wrapper_address}:{wrapper_port}"))
        .data("vm_id", payload.vm_id.clone())
        .data("tenant_id", payload.tenant_id.clone())
        .emit();
    let response = client
        .request(builder)
        .await
        .map_err(|e| SupervisorError::Network(format!("heartbeat send: {e}")))?;
    if !response.is_ok() {
        return Err(SupervisorError::Heartbeat(format!(
            "wrapper rejected heartbeat: {}",
            response.error.as_deref().unwrap_or("unknown error")
        )));
    }
    telemetry
        .builder()
        .level(telemetry::Level::Debug)
        .kind("heartbeat.ack")
        .category("heartbeat")
        .message("heartbeat acknowledged by wrapper")
        .data("vm_id", payload.vm_id.clone())
        .emit();
    Ok(())
}
