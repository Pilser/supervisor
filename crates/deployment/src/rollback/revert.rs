//! Revert to the previous state on failure.

use std::path::PathBuf;

use shared::{Result, SupervisorError};

use super::snapshot::Snapshot;

/// Restore snapshotted files next to their originals.
/// Stub: copies each snapshot file back alongside its source name into
/// `restore_dir`.
pub async fn revert_snapshot(snapshot: &Snapshot, restore_dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    tokio::fs::create_dir_all(restore_dir)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("mkdir restore: {e}")))?;
    let mut restored = Vec::new();
    for file in &snapshot.files {
        let name = file
            .file_name()
            .ok_or_else(|| SupervisorError::Deployment("bad snapshot entry".into()))?;
        let dest = restore_dir.join(name);
        tokio::fs::copy(file, &dest)
            .await
            .map_err(|e| SupervisorError::Deployment(format!("revert copy: {e}")))?;
        restored.push(dest);
    }
    Ok(restored)
}
