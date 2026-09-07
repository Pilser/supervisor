//! Built-in memory tools: save/get/list/search/delete memory.

use std::sync::Arc;

use crate::handler::memory::{MemoryOp, MemoryToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

/// Register the memory tool family.
pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    registry.register(Arc::new(
        ToolBuilder::new("save_memory")
            .description("Persist a memory entry for the tenant agent.")
            .with_param("key", "string", "Memory key", true)
            .with_param("value", "string", "Memory value", true)
            .with_param("category", "string", "Optional memory category", false)
            .with_param("importance", "number", "Importance 0..1", false)
            .build(Arc::new(MemoryToolHandler::new(MemoryOp::Save, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("get_memory")
            .description("Fetch a single memory entry by key.")
            .with_param("key", "string", "Memory key", true)
            .build(Arc::new(MemoryToolHandler::new(MemoryOp::Get, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("list_memory")
            .description("List memory entries, optionally filtered by category.")
            .with_param("category", "string", "Optional category filter", false)
            .with_param("limit", "number", "Max entries", false)
            .build(Arc::new(MemoryToolHandler::new(MemoryOp::List, Arc::clone(&proxy)))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("search_memory")
            .description("Semantic search over memory entries.")
            .with_param("query", "string", "Search query", true)
            .with_param("limit", "number", "Max results", false)
            .build(Arc::new(MemoryToolHandler::new(
                MemoryOp::Search,
                Arc::clone(&proxy),
            ))),
    ))?;

    registry.register(Arc::new(
        ToolBuilder::new("delete_memory")
            .description("Delete a memory entry by key.")
            .with_param("key", "string", "Memory key", true)
            .build(Arc::new(MemoryToolHandler::new(
                MemoryOp::Delete,
                Arc::clone(&proxy),
            ))),
    ))?;

    Ok(())
}