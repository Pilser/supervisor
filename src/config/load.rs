//! Load `config.toml` from disk or a string.

use super::SupervisorConfig;

/// Load and parse the config file at `path`.
pub fn load_from_file(path: &str) -> supervisor_core::Result<SupervisorConfig> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| supervisor_core::SupervisorError::Config(format!("read {path}: {e}")))?;
    load_from_str(&raw)
}

/// Parse TOML config text.
pub fn load_from_str(raw: &str) -> supervisor_core::Result<SupervisorConfig> {
    toml::from_str(raw).map_err(|e| supervisor_core::SupervisorError::Config(format!("parse: {e}")))
}
