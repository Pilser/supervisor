//! Heartbeat handle: payload and interval customization for plugins.

/// Plugin-facing handle for heartbeat customization.
#[derive(Debug, Clone)]
pub struct HeartbeatApi {
    pub interval: heartbeat::HeartbeatInterval,
    pub custom: heartbeat::CustomFields,
}

impl HeartbeatApi {
    pub fn new(interval_secs: u64) -> core::Result<Self> {
        Ok(Self {
            interval: heartbeat::HeartbeatInterval::new(interval_secs)?,
            custom: heartbeat::CustomFields::new(),
        })
    }

    /// Assemble a heartbeat payload with the given status sections.
    pub fn payload(
        &self,
        vm_id: &str,
        tenant_id: &str,
        programs: serde_json::Value,
        resources: serde_json::Value,
    ) -> core::Result<heartbeat::HeartbeatPayload> {
        heartbeat::build_heartbeat(vm_id, tenant_id, programs, resources, self.custom.to_json())
    }
}
