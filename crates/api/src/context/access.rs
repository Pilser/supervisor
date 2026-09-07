//! Read access to the shared context.

use super::SupervisorContext;

impl SupervisorContext {
    pub fn vm_id(&self) -> &str {
        &self.vm_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    pub fn wrapper_endpoint(&self) -> String {
        format!("{}:{}", self.wrapper_address, self.wrapper_port)
    }
}
