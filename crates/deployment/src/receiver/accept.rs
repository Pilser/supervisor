//! Accept an incoming deployment package.

use shared::{Result, SupervisorError};

use crate::artifacts::manifest::DeploymentManifest;

/// Parse and validate a deployment package received from the wrapper.
pub fn accept_deployment(raw: &str, telemetry: &telemetry::Telemetry) -> Result<DeploymentManifest> {
    let manifest: DeploymentManifest =
        serde_json::from_str(raw).map_err(|e| SupervisorError::Deployment(format!("parse: {e}")))?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.received")
        .category("deployment")
        .message(format!(
            "deployment for tenant {} category {} ({} artifacts)",
            manifest.tenant_id,
            manifest.category,
            manifest.artifacts.len()
        ))
        .emit();
    Ok(manifest)
}
