//! Read the agent's own health state (state/daemon_state.json).

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// One component's health as reported by the agent.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentComponent {
    pub status: String,
    #[serde(default)]
    pub last_ok: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub restart_count: u32,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl AgentComponent {
    pub fn is_healthy(&self) -> bool {
        self.status == "ok"
    }
}

/// Parsed agent state file.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentState {
    #[serde(default)]
    pub components: HashMap<String, AgentComponent>,
}

impl AgentState {
    /// True when every reported component is "ok".
    pub fn all_healthy(&self) -> bool {
        if self.components.is_empty() {
            return false;
        }
        self.components.values().all(|c| c.is_healthy())
    }

    /// Names of components that are not "ok".
    pub fn unhealthy_components(&self) -> Vec<(&String, &AgentComponent)> {
        self.components
            .iter()
            .filter(|(_, c)| !c.is_healthy())
            .collect()
    }
}

/// Read and parse the agent state file at `path`.
pub fn read_agent_state(path: &Path) -> Option<AgentState> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}
