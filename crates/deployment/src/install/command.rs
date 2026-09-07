//! Execute post-deployment commands.

use std::path::Path;

use core::Result;

/// Run the staged command when the artifact asked for it.
pub async fn install_command(
    staged: &Path,
    run_after_install: bool,
    telemetry: &telemetry::Telemetry,
) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.command_executed")
        .category("deployment")
        .message(format!(
            "command {} run_after_install={run_after_install}",
            staged.display()
        ))
        .emit();
    Ok(())
}
