//! MCP server: connection handling over TCP/UNIX transports plus handshake,
//! dispatch and cooperative shutdown.
//!
//! Transports (`tcp`, `unix`) read newline-delimited JSON-RPC messages and
//! dispatch them through `handshake::handle_message` against the shared
//! [`McpServer`] registry.

pub mod handshake;
pub mod shutdown;
pub mod tcp;
pub mod unix;

use std::sync::Arc;

use serde_json::Value;

use crate::tools::ToolRegistry;

pub use handshake::{handle_message, initialize_result};
pub use shutdown::{Shutdown, ShutdownSignal};

/// Shared MCP server state: the tool registry plus lightweight lifecycle hooks.
///
/// Emit methods are currently no-ops; telemetry/hook wiring is owned by other
/// agents and can replace these bodies without changing call sites.
#[derive(Clone)]
pub struct McpServer {
    pub registry: Arc<ToolRegistry>,
}

impl McpServer {
    /// A server backed by a fresh, empty registry.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ToolRegistry::new()),
        }
    }

    /// A server sharing an existing registry.
    pub fn with_registry(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    /// Hook: a client connected.
    pub fn emit_connected(&self) {}

    /// Hook: a client disconnected.
    pub fn emit_disconnected(&self) {}

    /// Hook: a transport-level error worth logging.
    pub fn emit_log_error(&self, _message: &str) {}

    /// Hook: a tool lifecycle event (`called` / `completed`).
    pub fn emit_tool(&self, _event: &str, _tool: &str, _args: &Value) {}

    /// Hook: a tool failure.
    pub fn emit_tool_error(&self, _event: &str, _tool: &str, _error: &str) {}
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
