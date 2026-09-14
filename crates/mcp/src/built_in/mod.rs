//! Built-in portfolio tools — single family `portfolio.*`.

pub mod portfolio;

use std::sync::Arc;

use crate::handler::{SharedProxy, noop_proxy};
use crate::tools::ToolRegistry;

/// Register portfolio tools using a shared no-op proxy.
pub fn register_all(registry: &ToolRegistry) -> anyhow::Result<()> {
    let proxy = noop_proxy();
    register_with_proxy(registry, proxy)
}

/// Register portfolio tools, routing calls through `proxy`.
pub fn register_with_proxy(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    portfolio::register(registry, Arc::clone(&proxy))?;
    Ok(())
}