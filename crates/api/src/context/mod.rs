//! Shared plugin context: identity plus subsystem access.

pub mod access;
pub mod builder;

pub use builder::SupervisorContextBuilder;

use serde::{Deserialize, Serialize};

/// Handle handed to every plugin: who this VM serves plus telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorContext {
    pub vm_id: String,
    pub tenant_id: String,
    pub socket_path: String,
    pub wrapper_address: String,
    pub wrapper_port: u16,
}
