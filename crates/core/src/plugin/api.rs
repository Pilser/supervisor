//! Plugin-facing API token.

#[derive(Debug, Clone, Default)]
pub struct PluginApi {
    pub plugin_id: String,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
}

impl PluginApi {
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            extensions: Default::default(),
        }
    }

    pub fn extend(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.extensions.insert(key.into(), value);
    }
}