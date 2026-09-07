//! Update binary command.

use core::Result;

use crate::executor::result::CommandResult;

/// Stage a binary update. The deployment crate performs the real install.
pub async fn update_binary(args: &serde_json::Value) -> Result<CommandResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
    Ok(CommandResult::ok(format!("update staged for {name}")))
}
