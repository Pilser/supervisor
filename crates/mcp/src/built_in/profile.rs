//! Built-in profile tools: get/update profile.

use std::sync::Arc;

use crate::handler::profile::{ProfileOp, ProfileToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the profile tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("get_profile")
            .description("Get the current tenant agent profile.")
            .with_param("username", "string", "Optional username", false)
            .build(Arc::new(ProfileToolHandler::new(ProfileOp::Get, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("update_profile")
            .description("Update profile fields for the tenant agent.")
            .with_param("username", "string", "Optional username", false)
            .with_param("updates", "object", "Profile fields to update", true)
            .build(Arc::new(ProfileToolHandler::new(ProfileOp::Update, Arc::clone(&proxy)))),
    ))?;

    Ok(())
}