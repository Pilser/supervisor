//! Built-in projects tools: create/update/list/get projects.

use std::sync::Arc;

use crate::handler::projects::{ProjectOp, ProjectToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the projects tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("create_project")
            .description("Create a project for the tenant.")
            .with_param("name", "string", "Project name", true)
            .with_param("domain", "string", "Optional domain", false)
            .with_param("description", "string", "Optional description", false)
            .build(Arc::new(ProjectToolHandler::new(ProjectOp::Create, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("update_project")
            .description("Update an existing project.")
            .with_param("id", "string", "Project id", true)
            .with_param("name", "string", "New name", false)
            .with_param("status", "string", "New status", false)
            .with_param("description", "string", "New description", false)
            .build(Arc::new(ProjectToolHandler::new(ProjectOp::Update, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("list_projects")
            .description("List tenant projects.")
            .with_param("status", "string", "Optional status filter", false)
            .with_param("domain", "string", "Optional domain filter", false)
            .build(Arc::new(ProjectToolHandler::new(ProjectOp::List, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("get_project")
            .description("Get a single project by id.")
            .with_param("id", "string", "Project id", true)
            .build(Arc::new(ProjectToolHandler::new(ProjectOp::Get, Arc::clone(&proxy)))),
    ))?;

    Ok(())
}