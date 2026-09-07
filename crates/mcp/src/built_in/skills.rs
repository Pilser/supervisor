//! Built-in skills tools: add/update/list/get skills.

use std::sync::Arc;

use crate::handler::skills::{SkillOp, SkillToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the skills tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("add_skill")
            .description("Record a skill for the learner agent.")
            .with_param("name", "string", "Skill name", true)
            .with_param("domain", "string", "Optional domain", false)
            .with_param("level", "string", "Skill level", false)
            .with_param("confidence", "number", "Confidence 0..1", false)
            .build(Arc::new(SkillToolHandler::new(SkillOp::Add, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("update_skill")
            .description("Update an existing skill.")
            .with_param("id", "string", "Skill id", true)
            .with_param("name", "string", "New name", false)
            .with_param("domain", "string", "New domain", false)
            .with_param("level", "string", "New level", false)
            .with_param("confidence", "number", "New confidence", false)
            .build(Arc::new(SkillToolHandler::new(SkillOp::Update, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("list_skills")
            .description("List learner skills.")
            .with_param("domain", "string", "Optional domain filter", false)
            .build(Arc::new(SkillToolHandler::new(SkillOp::List, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("get_skill")
            .description("Get a single skill by id.")
            .with_param("id", "string", "Skill id", true)
            .build(Arc::new(SkillToolHandler::new(SkillOp::Get, Arc::clone(&proxy)))),
    ))?;

    Ok(())
}