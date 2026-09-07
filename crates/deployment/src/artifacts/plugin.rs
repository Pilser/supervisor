//! Handle `.so` supervisor plugin files.

use std::path::PathBuf;

use shared::{Result, SupervisorError};

use super::manifest::DeploymentArtifact;
use crate::receiver::verify::verify_checksum;

/// Stage a supervisor plugin into `dest_dir`. Hot-reload is done by
/// `plugin_host` after install.
pub async fn handle_plugin_artifact(
    artifact: &DeploymentArtifact,
    dest_dir: &str,
    telemetry: &telemetry::Telemetry,
) -> Result<PathBuf> {
    let data = artifact
        .content
        .as_ref()
        .ok_or_else(|| SupervisorError::Deployment(format!("no bytes for {}", artifact.name)))?;
    verify_checksum(data, &artifact.checksum)?;
    let dest = PathBuf::from(dest_dir).join(&artifact.name);
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.artifact")
        .category("deployment")
        .message(format!("staging plugin {} -> {}", artifact.name, dest.display()))
        .emit();
    Ok(dest)
}
