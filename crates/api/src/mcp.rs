//! MCP handle: tool registry access for plugins.

use serde::{Deserialize, Serialize};

/// Plugin-facing handle for the MCP server (socket + tool calls).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpApi {
    pub socket_path: String,
    pub max_connections: u32,
}

impl McpApi {
    pub fn new(socket_path: &str, max_connections: u32) -> Self {
        Self {
            socket_path: socket_path.to_string(),
            max_connections,
        }
    }

    /// Describe a tool call request without depending on the `mcp` crate.
    pub fn tool_call(&self, tool: &str, args: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": { "name": tool, "arguments": args },
        })
    }
}
