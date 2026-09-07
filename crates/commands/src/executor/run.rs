//! Execute a parsed command via the built-in handlers.

use shared::{Result, SupervisorError};

use super::result::CommandResult;
use crate::listener::parse::IncomingCommand;

/// Dispatch `cmd` to its built-in handler.
pub async fn execute(cmd: &IncomingCommand, telemetry: &telemetry::Telemetry) -> Result<CommandResult> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("commands.received")
        .category("commands")
        .message(format!("executing {} id={}", cmd.name, cmd.id))
        .emit();
    let result = match cmd.name.as_str() {
        "install" => crate::built_in::install::install_plugin(&cmd.args).await?,
        "configure" => crate::built_in::configure::update_config(&cmd.args).await?,
        "restart" => crate::built_in::restart::restart_program(&cmd.args).await?,
        "shutdown" => crate::built_in::shutdown::prepare_shutdown(&cmd.args).await?,
        "update" => crate::built_in::update::update_binary(&cmd.args).await?,
        "run_script" => crate::built_in::run_script::run_script(&cmd.args).await?,
        "status" => crate::built_in::status::report_status(&cmd.args).await?,
        other => {
            return Err(SupervisorError::Command(format!("unknown command {other}")));
        }
    };
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("commands.executed")
        .category("commands")
        .message(format!("executed {} success={}", cmd.name, result.success))
        .emit();
    Ok(result)
}
