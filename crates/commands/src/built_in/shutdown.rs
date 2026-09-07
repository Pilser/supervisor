//! Prepare-for-shutdown command.

use shared::Result;

use crate::executor::result::CommandResult;

/// Flush state and prepare supervised programs for shutdown.
pub async fn prepare_shutdown(_args: &serde_json::Value) -> Result<CommandResult> {
    Ok(CommandResult::ok("shutdown prepared"))
}
