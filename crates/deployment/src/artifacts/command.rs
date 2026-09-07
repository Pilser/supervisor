//! Handle command/script files.

use std::path::PathBuf;

use shared::{Result, SupervisorError};

use super::manifest::DeploymentArtifact;
use crate::receiver::verify::verify_checksum;

/// Stage a command/script file into `dest_dir`.
pub async fn handle_command_artifact(
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
        .message(format!(
            "staging command {} -> {} run_after_install={}",
            artifact.name,
            dest.display(),
            artifact.run_after_install
        ))
        .emit();
    Ok(dest)
}
