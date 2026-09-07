//! Place config files in their target locations.

use std::path::Path;

use shared::{Result, SupervisorError};

/// Copy the staged config file to `target`.
pub async fn install_config(
    staged: &Path,
    target: &Path,
    telemetry: &telemetry::Telemetry,
) -> Result<()> {
    let bytes = tokio::fs::read(staged)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("read staged config: {e}")))?;
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| SupervisorError::Deployment(format!("mkdir config dir: {e}")))?;
    }
    tokio::fs::write(target, bytes)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("write config: {e}")))?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.config_updated")
        .category("deployment")
        .message(format!("config installed -> {}", target.display()))
        .emit();
    Ok(())
}
