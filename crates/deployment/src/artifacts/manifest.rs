//! Parse the deployment manifest sent by the wrapper.

use shared::{Result, SupervisorError};

pub use shared::{DeploymentArtifact, DeploymentManifest};

/// Parse a JSON deployment manifest.
pub fn parse_manifest(raw: &str) -> Result<DeploymentManifest> {
    serde_json::from_str(raw).map_err(|e| SupervisorError::Deployment(format!("manifest: {e}")))
}
