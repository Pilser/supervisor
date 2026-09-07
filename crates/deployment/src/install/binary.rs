//! Install binaries and set permissions.

use std::path::Path;

use shared::{Result, SupervisorError};

/// Copy the staged binary to `target` and mark it executable.
pub async fn install_binary(
    staged: &Path,
    target: &Path,
    telemetry: &telemetry::Telemetry,
) -> Result<()> {
    let bytes = tokio::fs::read(staged)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("read staged binary: {e}")))?;
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| SupervisorError::Deployment(format!("mkdir bin dir: {e}")))?;
    }
    tokio::fs::write(target, bytes)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("write binary: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perm = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(target, perm)
            .map_err(|e| SupervisorError::Deployment(format!("chmod +x: {e}")))?;
    }
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.binary_installed")
        .category("deployment")
        .message(format!("binary installed -> {}", target.display()))
        .emit();
    Ok(())
}
