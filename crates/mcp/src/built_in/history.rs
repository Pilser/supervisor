//! Built-in history tools: log/get history.

use std::sync::Arc;

use crate::handler::history::{HistoryOp, HistoryToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the history tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("log_history")
            .description("Log a tenant activity history entry.")
            .with_param("action", "string", "Action type", false)
            .with_param("summary", "string", "Human-readable summary", false)
            .with_param("domain", "string", "Optional domain", false)
            .with_param("duration_seconds", "number", "Duration in seconds", false)
            .with_param("session_id", "string", "Optional session id", false)
            .build(Arc::new(HistoryToolHandler::new(HistoryOp::Log, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("get_history")
            .description("Retrieve tenant activity history.")
            .with_param("domain", "string", "Optional domain filter", false)
            .with_param("limit", "number", "Max entries", false)
            .with_param("since", "string", "RFC3339 since timestamp", false)
            .build(Arc::new(HistoryToolHandler::new(HistoryOp::Get, Arc::clone(&proxy)))),
    ))?;

    Ok(())
}