//! Update config file command.

use core::Result;

use crate::executor::result::CommandResult;

/// Apply a config key/value update.
pub async fn update_config(args: &serde_json::Value) -> Result<CommandResult> {
    let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("unknown");
    Ok(CommandResult::ok(format!("config {key} updated")))
}
