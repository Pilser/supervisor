//! Install supervisor plugins (hot-reload via `plugin_host`).

use std::path::Path;

use core::Result;

/// Install the staged plugin at `staged`. Stub: hot-reload wiring lands with
/// the plugin pipeline.
pub async fn install_plugin(staged: &Path, telemetry: &telemetry::Telemetry) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.plugin_installed")
        .category("deployment")
        .message(format!("installed plugin {}", staged.display()))
        .emit();
    Ok(())
}
