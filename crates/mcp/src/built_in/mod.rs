//! Built-in tool sets defined by the architecture document.
//!
//! Each submodule registers a family of tools (memory, profile, history,
//! projects, skills, reports, configs, tenant) into the shared registry using a
//! single `DataProxy`.

pub mod configs;
pub mod history;
pub mod memory;
pub mod profile;
pub mod projects;
pub mod reports;
pub mod skills;
pub mod tenant;

use std::sync::Arc;

use crate::handler::{SharedProxy, noop_proxy};
use crate::tools::ToolRegistry;

/// Register every built-in tool family using a shared no-op proxy.
pub fn register_all(registry: &ToolRegistry) -> anyhow::Result<()> {
    let proxy = noop_proxy();
    register_with_proxy(registry, proxy)
}

/// Register every built-in tool family, routing calls through `proxy`.
pub fn register_with_proxy(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    memory::register(registry, Arc::clone(&proxy))?;
    profile::register(registry, Arc::clone(&proxy))?;
    history::register(registry, Arc::clone(&proxy))?;
    projects::register(registry, Arc::clone(&proxy))?;
    skills::register(registry, Arc::clone(&proxy))?;
    reports::register(registry, Arc::clone(&proxy))?;
    configs::register(registry, Arc::clone(&proxy))?;
    tenant::register(registry, Arc::clone(&proxy))?;
    Ok(())
}