//! Install plugin/tool command.

use core::Result;

use crate::executor::result::CommandResult;

/// Stage a plugin install. The deployment crate performs the real install.
pub async fn install_plugin(args: &serde_json::Value) -> Result<CommandResult> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
    Ok(CommandResult::ok(format!("install staged for {name}")))
}
