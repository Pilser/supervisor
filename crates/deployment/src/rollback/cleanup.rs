//! Cleanup failed deployment artifacts.

use shared::{Result, SupervisorError};

use super::snapshot::Snapshot;

/// Remove the snapshot directory after a successful deploy or revert.
pub async fn cleanup_snapshot(snapshot: &Snapshot) -> Result<()> {
    if snapshot.dir.exists() {
        tokio::fs::remove_dir_all(&snapshot.dir)
            .await
            .map_err(|e| SupervisorError::Deployment(format!("cleanup snapshot: {e}")))?;
    }
    Ok(())
}
