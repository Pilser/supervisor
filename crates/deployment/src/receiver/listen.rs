//! Listen for deployment pushes from the wrapper.

use shared::{Result, SupervisorError};

/// Bind the deployment port. The accept loop is driven by the caller.
pub async fn listen_for_deployment(port: u16, telemetry: &telemetry::Telemetry) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let _listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| SupervisorError::Deployment(format!("bind {addr}: {e}")))?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.listen")
        .category("deployment")
        .message(format!("listening for deployments on {addr}"))
        .emit();
    Ok(())
}
