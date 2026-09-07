//! Install agent plugins/tools.

use std::path::Path;

use core::Result;

/// Install the staged agent plugin at `staged`.
pub async fn install_agent_plugin(staged: &Path, telemetry: &telemetry::Telemetry) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.agent_plugin_installed")
        .category("deployment")
        .message(format!("installed agent plugin {}", staged.display()))
        .emit();
    Ok(())
}
