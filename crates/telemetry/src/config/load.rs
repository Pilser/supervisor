//! Loading `[telemetry]` config from a TOML file / string.

use std::path::Path;

use super::TelemetrySettings;

pub fn load_from_file(path: &Path) -> anyhow::Result<Option<TelemetrySettings>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
    load_from_str(&text)
}

pub fn load_from_str(toml: &str) -> anyhow::Result<Option<TelemetrySettings>> {
    let value: toml::Value = toml::from_str(toml).map_err(|e| anyhow::anyhow!("invalid toml: {e}"))?;
    match value.get("telemetry") {
        None => Ok(None),
        Some(section) => {
            let settings: TelemetrySettings = section
                .clone()
                .try_into()
                .map_err(|e| anyhow::anyhow!("bad [telemetry] section: {e}"))?;
            Ok(Some(settings))
        }
    }
}