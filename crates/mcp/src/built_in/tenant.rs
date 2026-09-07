//! Built-in tenant tools: get_tenant_info, get_category_info.

use std::sync::Arc;

use crate::handler::tenant::{TenantOp, TenantToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the tenant tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("get_tenant_info")
            .description("Get the current tenant identity and context.")
            .build(Arc::new(TenantToolHandler::new(
                TenantOp::GetInfo,
                Arc::clone(&proxy),
            ))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("get_category_info")
            .description("Get deployment-relevant category info.")
            .with_param("category_id", "string", "Category id", true)
            .build(Arc::new(TenantToolHandler::new(
                TenantOp::GetCategory,
                Arc::clone(&proxy),
            ))),
    ))?;

    Ok(())
}