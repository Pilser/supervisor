//! Built-in configs tools: get/set config.

use std::sync::Arc;

use crate::handler::configs::{ConfigOp, ConfigToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the configs tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("get_config")
            .description("Get a configuration value.")
            .with_param("key", "string", "Config key", true)
            .build(Arc::new(ConfigToolHandler::new(ConfigOp::Get, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("set_config")
            .description("Set a configuration value.")
            .with_param("key", "string", "Config key", true)
            .with_param("value", "string", "Config value (JSON)", true)
            .build(Arc::new(ConfigToolHandler::new(ConfigOp::Set, Arc::clone(&proxy)))),
    ))?;

    Ok(())
}