//! Deployment package types received from the wrapper.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    SupervisorPlugin,
    AgentPlugin,
    Config,
    Binary,
    Command,
}

/// One artifact in a deployment package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentArtifact {
    pub artifact_type: ArtifactType,
    pub name: String,
    pub path: String,
    pub checksum: String,
    #[serde(default)]
    pub content: Option<Vec<u8>>,
    #[serde(default)]
    pub run_after_install: bool,
}

/// The deployment package pushed to a supervisor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentManifest {
    pub tenant_id: String,
    pub category: String,
    pub artifacts: Vec<DeploymentArtifact>,
}