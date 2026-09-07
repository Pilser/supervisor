//! Plugin handle: inter-plugin API surface.

use serde::{Deserialize, Serialize};

/// Plugin-facing handle describing this plugin to the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginApi {
    pub name: String,
    pub version: String,
}

impl PluginApi {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}
