//! Report-status command.

use shared::Result;

use crate::executor::result::CommandResult;

/// Return a snapshot of supervisor status.
pub async fn report_status(_args: &serde_json::Value) -> Result<CommandResult> {
    Ok(CommandResult::ok_with(
        "status ok",
        serde_json::json!({ "status": "running" }),
    ))
}
