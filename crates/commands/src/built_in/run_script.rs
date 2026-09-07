//! Execute arbitrary script/command.

use core::Result;

use crate::executor::result::CommandResult;

/// Record a script execution request. Stub: no shell-out in the stub crate.
pub async fn run_script(args: &serde_json::Value) -> Result<CommandResult> {
    let script = args.get("script").and_then(|v| v.as_str()).unwrap_or("unknown");
    Ok(CommandResult::ok(format!("script staged: {script}")))
}
