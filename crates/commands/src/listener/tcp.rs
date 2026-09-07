//! Listen for commands from the wrapper.

use core::{Result, SupervisorError};

/// Bind the command port. The accept loop is driven by the caller.
pub async fn start_listener(port: u16, telemetry: &telemetry::Telemetry) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let _listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| SupervisorError::Command(format!("bind {addr}: {e}")))?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("commands.listen")
        .category("commands")
        .message(format!("listening for commands on {addr}"))
        .emit();
    Ok(())
}
