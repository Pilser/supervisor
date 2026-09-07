//! The central tool registry: registration, lookup and dispatch.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail};
use serde_json::Value;

use super::validate::validate_params;
use super::Tool;

/// Holds every registered `Tool`, keyed by name. Safe to share across the
/// server: registration is a mutex-protected hash map, dispatch is via the
/// tool's `Arc<dyn ToolHandler>`.
pub struct ToolRegistry {
    tools: Mutex<HashMap<String, Arc<Tool>>>,
}

impl ToolRegistry {
    /// A new, empty registry.
    pub fn new() -> Self {
        Self {
            tools: Mutex::new(HashMap::new()),
        }
    }

    /// Register a tool. Returns an error when the name is empty or already in
    /// use.
    pub fn register(&self, tool: Arc<Tool>) -> anyhow::Result<()> {
        if tool.name.trim().is_empty() {
            bail!("tool name must not be empty");
        }
        let mut guard = self
            .tools
            .lock()
            .map_err(|_| anyhow!("tool registry lock poisoned"))?;
        if guard.contains_key(&tool.name) {
            bail!("tool already registered: {}", tool.name);
        }
        guard.insert(tool.name.clone(), tool);
        Ok(())
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<Tool>> {
        self.tools
            .lock()
            .ok()?
            .get(name)
            .cloned()
    }

    /// List all registered tools sorted by name.
    pub fn list(&self) -> Vec<Arc<Tool>> {
        let mut tools: Vec<Arc<Tool>> = self
            .tools
            .lock()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default();
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        tools
    }

    /// Validate `params` against the tool's schema and dispatch to its handler.
    pub async fn call(&self, name: &str, params: Value) -> anyhow::Result<Value> {
        let tool = self
            .get(name)
            .ok_or_else(|| anyhow!("unknown tool: {name}"))?;
        validate_params(&tool.params, &params)?;
        tool.handler.call(params).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}