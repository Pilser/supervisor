//! Custom status fields (plugin hook surface).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Extra key/value fields merged into the heartbeat payload.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomFields {
    pub fields: HashMap<String, String>,
}

impl CustomFields {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.fields.insert(key, value);
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.fields).unwrap_or(serde_json::Value::Null)
    }
}
