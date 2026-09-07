//! Restart-a-program command.

use shared::Result;

use crate::executor::result::CommandResult;

/// Restart the named supervised program.
pub async fn restart_program(args: &serde_json::Value) -> Result<CommandResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
    Ok(CommandResult::ok(format!("restart requested for {name}")))
}
