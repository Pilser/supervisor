//! Built-in reports tools: get/list reports.

use std::sync::Arc;

use crate::handler::reports::{ReportOp, ReportToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the reports tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("get_report")
            .description("Get a progress report for a period.")
            .with_param("period", "string", "daily|weekly|monthly", true)
            .with_param("id", "string", "Optional report id", false)
            .build(Arc::new(ReportToolHandler::new(ReportOp::Get, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("list_reports")
            .description("List available progress reports.")
            .with_param("period", "string", "Optional period filter", false)
            .with_param("limit", "number", "Max results", false)
            .build(Arc::new(ReportToolHandler::new(ReportOp::List, Arc::clone(&proxy)))),
    ))?;

    Ok(())
}