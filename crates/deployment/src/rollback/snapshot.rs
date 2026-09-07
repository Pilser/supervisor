//! Snapshot current state before a deployment.

use std::path::{Path, PathBuf};

use shared::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// A snapshot of pre-deployment files that can be reverted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub dir: PathBuf,
    pub files: Vec<PathBuf>,
}

/// Copy `paths` into a fresh directory under `snapshot_dir`.
pub async fn take_snapshot(paths: &[PathBuf], snapshot_dir: &Path) -> Result<Snapshot> {
    let id = uuid::Uuid::new_v4().to_string();
    let dir = snapshot_dir.join(&id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("mkdir snapshot: {e}")))?;
    let mut files = Vec::new();
    for path in paths {
        if !path.exists() {
            continue;
        }
        let name = path
            .file_name()
            .ok_or_else(|| SupervisorError::Deployment("bad snapshot path".into()))?;
        let dest = dir.join(name);
        tokio::fs::copy(path, &dest)
            .await
            .map_err(|e| SupervisorError::Deployment(format!("snapshot copy: {e}")))?;
        files.push(dest);
    }
    Ok(Snapshot { id, dir, files })
}
