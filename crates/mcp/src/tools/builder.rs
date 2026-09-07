//! Fluent builder used by plugin code to define a tool.

use std::sync::Arc;

use serde_json::{json, Value};

use super::handler::ToolHandler;
use super::registry::ToolRegistry;
use super::Tool;

/// Fluently assemble a `Tool` (name, description, JSON-schema params) and
/// attach a handler.
pub struct ToolBuilder {
    name: String,
    description: String,
    properties: serde_json::Map<String, Value>,
    required: Vec<String>,
}

impl ToolBuilder {
    /// Start building a tool with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            properties: serde_json::Map::new(),
            required: Vec::new(),
        }
    }

    /// Set the human-readable description advertised in `tools/list`.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a JSON-schema property to `params`.
    ///
    /// `param_type` is a JSON Schema type such as `"string"`, `"number"` or
    /// `"boolean"`. When `required` is true the key is added to the schema's
    /// `required` array.
    pub fn with_param(
        mut self,
        pname: &str,
        param_type: &str,
        description: &str,
        required: bool,
    ) -> Self {
        self.properties.insert(
            pname.to_string(),
            json!({ "type": param_type, "description": description }),
        );
        if required {
            self.required.push(pname.to_string());
        }
        self
    }

    /// Turn the builder into a `Tool` bound to `handler`.
    pub fn build(self, handler: Arc<dyn ToolHandler>) -> Tool {
        Tool {
            name: self.name,
            description: self.description,
            params: json!({
                "type": "object",
                "properties": self.properties,
                "required": self.required,
            }),
            handler,
        }
    }

    /// Alias of [`ToolBuilder::build`] for plugin ergonomics.
    pub fn with_handler(self, handler: Arc<dyn ToolHandler>) -> Tool {
        self.build(handler)
    }

    /// Build the tool and register it immediately with `registry`.
    pub fn register(
        self,
        handler: Arc<dyn ToolHandler>,
        registry: &ToolRegistry,
    ) -> anyhow::Result<()> {
        let tool = self.build(handler);
        registry.register(Arc::new(tool))
    }
}