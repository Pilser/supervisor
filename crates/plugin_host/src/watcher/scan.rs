//! Scan the plugin folder for loadable `.so` files.

use std::path::{Path, PathBuf};

/// Return every `.so` file directly under `dir`, sorted for determinism.
///
/// Missing directories yield an empty list; I/O errors surface as
/// `SupervisorError::Io`.
pub fn plugin_files(dir: &Path) -> crate::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("so") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
