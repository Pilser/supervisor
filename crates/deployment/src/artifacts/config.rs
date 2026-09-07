//! Handle config files for supervised programs.

use std::path::PathBuf;

use core::{Result, SupervisorError};

use super::manifest::DeploymentArtifact;
use crate::receiver::verify::verify_checksum;

/// Stage a config file into `dest_dir`.
pub async fn handle_config_artifact(
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
        .message(format!("staging config {} -> {}", artifact.name, dest.display()))
        .emit();
    Ok(dest)
}
