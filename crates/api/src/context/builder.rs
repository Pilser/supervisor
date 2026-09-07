//! Builder for [`SupervisorContext`].

use super::SupervisorContext;

/// Fluent builder for the shared plugin context.
#[derive(Debug, Default)]
pub struct SupervisorContextBuilder {
    vm_id: Option<String>,
    tenant_id: Option<String>,
    socket_path: Option<String>,
    wrapper_address: Option<String>,
    wrapper_port: Option<u16>,
}

impl SupervisorContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vm_id(mut self, v: impl Into<String>) -> Self {
        self.vm_id = Some(v.into());
        self
    }

    pub fn tenant_id(mut self, v: impl Into<String>) -> Self {
        self.tenant_id = Some(v.into());
        self
    }

    pub fn socket_path(mut self, v: impl Into<String>) -> Self {
        self.socket_path = Some(v.into());
        self
    }

    pub fn wrapper_address(mut self, v: impl Into<String>) -> Self {
        self.wrapper_address = Some(v.into());
        self
    }

    pub fn wrapper_port(mut self, v: u16) -> Self {
        self.wrapper_port = Some(v);
        self
    }

    pub fn build(self) -> core::Result<SupervisorContext> {
        Ok(SupervisorContext {
            vm_id: self.vm_id.unwrap_or_else(|| "vm_001".to_string()),
            tenant_id: self.tenant_id.unwrap_or_default(),
            socket_path: self
                .socket_path
                .unwrap_or_else(|| "/tmp/supervisor.sock".to_string()),
            wrapper_address: self.wrapper_address.unwrap_or_else(|| "172.16.0.1".to_string()),
            wrapper_port: self.wrapper_port.unwrap_or(9090),
        })
    }
}
