//! Handed to a plugin at register/unregister time.

use serde_json::Value;

use super::{PluginApi, PluginInfo};

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub info: PluginInfo,
    pub api: PluginApi,
    pub config: Value,
}

impl PluginContext {
    pub fn new(info: PluginInfo, api: PluginApi) -> Self {
        Self {
            info,
            api,
            config: Value::Object(Default::default()),
        }
    }

    pub fn with_config(mut self, config: Value) -> Self {
        self.config = config;
        self
    }
}