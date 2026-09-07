//! Category info the supervisor uses to drive deployment.

use serde::{Deserialize, Serialize};

/// Details about the tenant's category that affect what gets deployed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryInfo {
    pub category_id: String,
    pub name: String,
    pub vm_template_id: String,
    /// Supervisor plugin artifact names.
    pub supervisor_plugins: Vec<String>,
    /// Agent plugin / tool names.
    pub agent_plugins: Vec<String>,
    #[serde(default)]
    pub config_files: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
}