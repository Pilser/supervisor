//! Tool abstraction: metadata + handler, builder, registry and validation.

pub mod builder;
pub mod handler;
pub mod registry;
pub mod validate;

use std::sync::Arc;

use serde_json::Value;

pub use builder::ToolBuilder;
pub use handler::{BoxFuture, ToolHandler};
pub use registry::ToolRegistry;

/// A single MCP tool: its advertised metadata plus the handler that executes
/// calls.
pub struct Tool {
    pub name: String,
    pub description: String,
    /// JSON schema describing `params` (a JSON Schema `object`).
    pub params: Value,
    pub handler: Arc<dyn ToolHandler>,
}