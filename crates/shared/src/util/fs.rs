//! Filesystem helpers.

use crate::SupervisorError;

/// Atomically write `bytes` to `path` via temp file + rename.
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> crate::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.as_os_str().to_owned();
    let mut tmp = std::path::PathBuf::from(tmp);
    let name = format!("{}.tmp", tmp.file_name().map(|n| n.to_string_lossy()).unwrap_or_default());
    tmp.set_file_name(name);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Read a JSON value from disk.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> crate::Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)?;
    let value = serde_json::from_slice(&bytes)?;
    Ok(Some(value))
}

/// Write a value as pretty JSON to disk (atomically).
pub fn write_json<T: serde::Serialize>(path: &std::path::Path, value: &T) -> crate::Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    write_atomic(path, &bytes)
}

/// Ensure a directory tree exists.
pub fn ensure_dir(path: &std::path::Path) -> crate::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Represent an arbitrary I/O error as an internal supervisor error.
pub fn io_err(e: std::io::Error) -> SupervisorError {
    SupervisorError::Internal(e.to_string())
}