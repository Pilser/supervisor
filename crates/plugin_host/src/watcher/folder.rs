//! Watch a plugin folder for additions, changes and removals.

use std::path::{Path, PathBuf};

/// Polls `path` for `.so` changes.
#[derive(Debug, Clone)]
pub struct FolderWatcher {
    /// Directory being watched.
    pub path: PathBuf,
}

impl FolderWatcher {
    /// Watch `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The watched directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Snapshot the currently-present plugin files.
    pub fn scan(&self) -> crate::Result<Vec<PathBuf>> {
        super::scan::plugin_files(&self.path)
    }
}
